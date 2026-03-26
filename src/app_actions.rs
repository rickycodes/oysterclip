use arboard::Clipboard;
use chrono::{Datelike, Local, TimeZone, Utc};
use dioxus::prelude::*;
use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::entry::{CachedEntries, ClipboardEntry};
use crate::history::{clear_history, delete_entries, delete_entry};
use crate::source::ClipboardSource;

const STATUS_TIMEOUT_SECS: u64 = 5;
const STATUS_TIMEOUT: Duration = Duration::from_secs(STATUS_TIMEOUT_SECS);

pub fn entry_id(entry: &ClipboardEntry) -> i64 {
    match entry {
        ClipboardEntry::Text { id, .. } | ClipboardEntry::Image { id, .. } => *id,
    }
}

pub fn matches_query(entry: &ClipboardEntry, query: &str) -> bool {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return true;
    }

    // Parse filters from query (e.g., "type:image kind:url search text")
    let (filters, search_text) = parse_query_filters(trimmed);

    // Check type and kind filters first
    if !filters.is_empty() {
        if !apply_filters(entry, &filters) {
            return false;
        }
    }

    // If no search text remains, we're done (filters alone matched)
    if search_text.is_empty() {
        return true;
    }

    // Apply text search on content and kind
    let search = search_text.to_lowercase();
    match entry {
        ClipboardEntry::Text { content, kind, .. } => {
            content.to_lowercase().contains(&search)
                || kind
                    .as_deref()
                    .map(|kind| kind.to_lowercase().contains(&search))
                    .unwrap_or(false)
        }
        ClipboardEntry::Image { path, .. } => {
            path.as_deref()
                .map(|value| value.to_lowercase().contains(&search))
                .unwrap_or(false)
        }
    }
}

#[derive(Debug, Clone)]
struct QueryFilter {
    key: String,
    value: String,
}

fn parse_query_filters(query: &str) -> (Vec<QueryFilter>, String) {
    let mut filters = Vec::new();
    let mut search_parts = Vec::new();

    for part in query.split_whitespace() {
        if let Some((key, value)) = part.split_once(':') {
            if matches!(key, "type" | "kind" | "since") && !value.is_empty() {
                filters.push(QueryFilter {
                    key: key.to_lowercase(),
                    value: value.to_lowercase(),
                });
            } else {
                search_parts.push(part);
            }
        } else {
            search_parts.push(part);
        }
    }

    (filters, search_parts.join(" "))
}

fn apply_filters(entry: &ClipboardEntry, filters: &[QueryFilter]) -> bool {
    for filter in filters {
        match filter.key.as_str() {
            "type" => {
                let type_matches = match entry {
                    ClipboardEntry::Text { .. } => {
                        filter.value == "text" || filter.value == "pass" || filter.value == "password"
                    }
                    ClipboardEntry::Image { .. } => filter.value == "image",
                };
                if !type_matches {
                    return false;
                }
            }
            "kind" => {
                let kind_matches = match entry {
                    ClipboardEntry::Text { kind, content, .. } => {
                        let entry_kind = if is_password(content) {
                            "password"
                        } else if let Some(k) = kind {
                            k.as_str()
                        } else {
                            "text"
                        };
                        entry_kind.to_lowercase().contains(&filter.value)
                    }
                    ClipboardEntry::Image { .. } => false,
                };
                if !kind_matches {
                    return false;
                }
            }
            "since" => {
                if let Some(cutoff) = parse_since_cutoff(&filter.value) {
                    let ts = match entry {
                        ClipboardEntry::Text { timestamp, .. }
                        | ClipboardEntry::Image { timestamp, .. } => *timestamp,
                    };
                    if ts < cutoff {
                        return false;
                    }
                }
            }
            _ => {}
        }
    }
    true
}

/// Parse a `since:` value into a UTC unix timestamp cutoff.
/// Supported: `1h`, `Nh` (N hours), `Nd` (N days), `today`, `yesterday`.
fn parse_since_cutoff(value: &str) -> Option<u64> {
    let now = Utc::now();

    if value == "today" {
        let local = Local::now();
        let start = Local
            .with_ymd_and_hms(local.year(), local.month(), local.day(), 0, 0, 0)
            .single()?;
        return Some(start.with_timezone(&Utc).timestamp() as u64);
    }

    if value == "yesterday" {
        let local = Local::now() - chrono::Duration::days(1);
        let start = Local
            .with_ymd_and_hms(local.year(), local.month(), local.day(), 0, 0, 0)
            .single()?;
        return Some(start.with_timezone(&Utc).timestamp() as u64);
    }

    if let Some(n) = value.strip_suffix('h') {
        let hours: i64 = n.parse().ok()?;
        return Some((now - chrono::Duration::hours(hours)).timestamp() as u64);
    }

    if let Some(n) = value.strip_suffix('d') {
        let days: i64 = n.parse().ok()?;
        return Some((now - chrono::Duration::days(days)).timestamp() as u64);
    }

    None
}

fn is_password(content: &str) -> bool {
    crate::format::is_password(content)
}

pub fn adjacent_entry_id(
    entries: &[ClipboardEntry],
    selected_id: Option<i64>,
    direction: isize,
) -> Option<i64> {
    if entries.is_empty() {
        return None;
    }

    let current_index =
        selected_id.and_then(|id| entries.iter().position(|entry| entry_id(entry) == id));
    let next_index = match (current_index, direction) {
        (Some(index), step) if step > 0 => (index + 1).min(entries.len() - 1),
        (Some(index), _) => index.saturating_sub(1),
        (None, step) if step > 0 => 0,
        (None, _) => entries.len() - 1,
    };

    entries.get(next_index).map(entry_id)
}

pub fn confirm_and_clear_history(
    source: Arc<ClipboardSource>,
    cache: Arc<Mutex<Option<CachedEntries>>>,
    mut entries: Signal<Vec<ClipboardEntry>>,
    mut selected_id: Signal<Option<i64>>,
    mut selected_ids: Signal<HashSet<i64>>,
    mut error: Signal<Option<String>>,
    action_status: Signal<Option<String>>,
) {
    let confirmed = MessageDialog::new()
        .set_level(MessageLevel::Warning)
        .set_title("Clear clipboard history?")
        .set_description("This will permanently delete all clipboard history entries.")
        .set_buttons(MessageButtons::OkCancel)
        .show();

    if !matches!(confirmed, MessageDialogResult::Ok) {
        return;
    }

    match clear_history(&source) {
        Ok(_) => {
            if let Ok(mut cache_guard) = cache.lock() {
                *cache_guard = None;
            }
            entries.set(Vec::new());
            selected_id.set(None);
            selected_ids.set(HashSet::new());
            error.set(None);
            set_status(action_status, "History cleared");
        }
        Err(err) => {
            error.set(Some(err));
            set_status(action_status, "Clear failed");
        }
    }
}

pub fn confirm_and_delete_entry(
    source: Arc<ClipboardSource>,
    cache: Arc<Mutex<Option<CachedEntries>>>,
    mut entries: Signal<Vec<ClipboardEntry>>,
    mut selected_id: Signal<Option<i64>>,
    mut error: Signal<Option<String>>,
    action_status: Signal<Option<String>>,
    id: i64,
) {
    let confirmed = MessageDialog::new()
        .set_level(MessageLevel::Warning)
        .set_title("Delete clipboard entry?")
        .set_description("This will permanently delete the selected clipboard entry.")
        .set_buttons(MessageButtons::OkCancel)
        .show();

    if !matches!(confirmed, MessageDialogResult::Ok) {
        return;
    }

    match delete_entry(&source, id) {
        Ok(_) => {
            if let Ok(mut cache_guard) = cache.lock() {
                *cache_guard = None;
            }
            let mut next_entries = entries();
            next_entries.retain(|entry| match entry {
                ClipboardEntry::Text { id: entry_id, .. }
                | ClipboardEntry::Image { id: entry_id, .. } => *entry_id != id,
            });
            entries.set(next_entries);
            selected_id.set(None);
            error.set(None);
            set_status(action_status, "Entry deleted");
        }
        Err(err) => {
            error.set(Some(err));
            set_status(action_status, "Delete failed");
        }
    }
}

pub fn confirm_and_delete_entries(
    source: Arc<ClipboardSource>,
    cache: Arc<Mutex<Option<CachedEntries>>>,
    mut entries: Signal<Vec<ClipboardEntry>>,
    mut selected_id: Signal<Option<i64>>,
    mut selected_ids: Signal<HashSet<i64>>,
    mut error: Signal<Option<String>>,
    action_status: Signal<Option<String>>,
    ids: Vec<i64>,
) {
    let count = ids.len();
    let noun = if count == 1 { "entry" } else { "entries" };
    let confirmed = MessageDialog::new()
        .set_level(MessageLevel::Warning)
        .set_title(&format!("Delete {count} {noun}?"))
        .set_description(&format!(
            "This will permanently delete {count} selected clipboard {noun}."
        ))
        .set_buttons(MessageButtons::OkCancel)
        .show();

    if !matches!(confirmed, MessageDialogResult::Ok) {
        return;
    }

    match delete_entries(&source, &ids) {
        Ok(_) => {
            if let Ok(mut cache_guard) = cache.lock() {
                *cache_guard = None;
            }
            let id_set: HashSet<i64> = ids.iter().cloned().collect();
            let mut next_entries = entries();
            next_entries.retain(|e| !id_set.contains(&entry_id(e)));
            entries.set(next_entries);
            selected_id.set(None);
            selected_ids.set(HashSet::new());
            error.set(None);
            set_status(action_status, format!("{count} {noun} deleted"));
        }
        Err(err) => {
            error.set(Some(err));
            set_status(action_status, "Delete failed");
        }
    }
}

pub fn copy_text_to_clipboard(copy_status: Signal<Option<String>>, text: String) {
    let result = Clipboard::new().and_then(|mut cb| cb.set_text(text));
    match result {
        Ok(_) => set_status(copy_status, "Copied"),
        Err(_) => set_status(copy_status, "Copy failed"),
    }
}

pub fn set_status(mut status: Signal<Option<String>>, message: impl Into<String>) {
    let message = message.into();
    status.set(Some(message.clone()));

    spawn({
        let message = message.clone();
        async move {
            tokio::time::sleep(STATUS_TIMEOUT).await;

            if status() == Some(message.clone()) {
                status.set(None);
            }
        }
    });
}

pub fn open_url(url: &str) {
    let cmd = if cfg!(target_os = "windows") {
        format!("start {}", url)
    } else if cfg!(target_os = "macos") {
        format!("open {}", url)
    } else {
        format!("xdg-open {}", url)
    };

    let _ = std::process::Command::new("sh").arg("-c").arg(cmd).spawn();
}
