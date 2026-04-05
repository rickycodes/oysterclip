use arboard::Clipboard;
use chrono::{Datelike, Local, TimeZone, Utc};
use dioxus::prelude::*;
use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::config::source::ClipboardSource;
use crate::data::entry::{CachedEntries, ClipboardEntry};
use crate::data::history::{clear_history, delete_entries, delete_entry};

const STATUS_TIMEOUT_SECS: u64 = 5;
const STATUS_TIMEOUT: Duration = Duration::from_secs(STATUS_TIMEOUT_SECS);

/// Holds mutable state signals used by confirmation dialogs and actions.
pub struct DeleteActionState {
    pub entries: Signal<Vec<ClipboardEntry>>,
    pub selected_id: Signal<Option<i64>>,
    pub selected_ids: Signal<HashSet<i64>>,
    pub error: Signal<Option<String>>,
    pub action_status: Signal<Option<String>>,
}

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
    if !filters.is_empty() && !apply_filters(entry, &filters) {
        return false;
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
        ClipboardEntry::Image { path, .. } => path
            .as_deref()
            .map(|value| value.to_lowercase().contains(&search))
            .unwrap_or(false),
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
                        filter.value == "text"
                            || filter.value == "pass"
                            || filter.value == "password"
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
/// Supported: `Nm` (N minutes), `Nh` (N hours), `Nd` (N days), `Nw` (N weeks), `today`, `yesterday`.
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

    if let Some(n) = value.strip_suffix('m') {
        let minutes: i64 = n.parse().ok()?;
        return Some((now - chrono::Duration::minutes(minutes)).timestamp() as u64);
    }

    if let Some(n) = value.strip_suffix('h') {
        let hours: i64 = n.parse().ok()?;
        return Some((now - chrono::Duration::hours(hours)).timestamp() as u64);
    }

    if let Some(n) = value.strip_suffix('d') {
        let days: i64 = n.parse().ok()?;
        return Some((now - chrono::Duration::days(days)).timestamp() as u64);
    }

    if let Some(n) = value.strip_suffix('w') {
        let weeks: i64 = n.parse().ok()?;
        return Some((now - chrono::Duration::weeks(weeks)).timestamp() as u64);
    }

    None
}

fn is_password(content: &str) -> bool {
    crate::data::format::is_password(content)
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
    mut state: DeleteActionState,
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
            state.entries.set(Vec::new());
            state.selected_id.set(None);
            state.selected_ids.set(HashSet::new());
            state.error.set(None);
            set_status(state.action_status, "History cleared");
        }
        Err(err) => {
            state.error.set(Some(err));
            set_status(state.action_status, "Clear failed");
        }
    }
}

pub fn confirm_and_delete_entry(
    source: Arc<ClipboardSource>,
    cache: Arc<Mutex<Option<CachedEntries>>>,
    mut state: DeleteActionState,
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
            let mut next_entries = (state.entries)();
            next_entries.retain(|entry| match entry {
                ClipboardEntry::Text { id: entry_id, .. }
                | ClipboardEntry::Image { id: entry_id, .. } => entry_id != &id,
            });
            state.entries.set(next_entries);
            state.selected_id.set(None);
            state.error.set(None);
            set_status(state.action_status, "Entry deleted");
        }
        Err(err) => {
            state.error.set(Some(err));
            set_status(state.action_status, "Delete failed");
        }
    }
}

pub fn confirm_and_delete_entries(
    source: Arc<ClipboardSource>,
    cache: Arc<Mutex<Option<CachedEntries>>>,
    mut state: DeleteActionState,
    ids: Vec<i64>,
) {
    let count = ids.len();
    let noun = if count == 1 { "entry" } else { "entries" };
    let confirmed = MessageDialog::new()
        .set_level(MessageLevel::Warning)
        .set_title(format!("Delete {count} {noun}?"))
        .set_description(format!(
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
            let mut next_entries = (state.entries)();
            next_entries.retain(|e| !id_set.contains(&entry_id(e)));
            state.entries.set(next_entries);
            state.selected_id.set(None);
            state.selected_ids.set(HashSet::new());
            state.error.set(None);
            set_status(state.action_status, format!("{count} {noun} deleted"));
        }
        Err(err) => {
            state.error.set(Some(err));
            set_status(state.action_status, "Delete failed");
        }
    }
}

pub fn copy_text_to_clipboard(
    mut copy_status: Signal<Option<(i64, String)>>,
    entry_id: i64,
    text: String,
    label: &str,
) {
    let result = Clipboard::new().and_then(|mut cb| cb.set_text(text));
    let message = match result {
        Ok(_) => format!("Copied {}", label),
        Err(_) => "Copy failed".to_string(),
    };
    copy_status.set(Some((entry_id, message.clone())));

    spawn(async move {
        tokio::time::sleep(STATUS_TIMEOUT).await;
        if copy_status().as_ref().map(|(_, m)| m.as_str()) == Some(message.as_str()) {
            copy_status.set(None);
        }
    });
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

/// Aggregate multiple selected clipboard entries and send to a target app (e.g., notepad).
///
/// Config example:
/// ```toml
/// [bulk_actions.handlers]
/// send_to_notepad = {
///     handler_type = "aggregate_command",
///     app = "notepad",
///     separator = "\n---\n",
///     template = "[{timestamp}] {text}"
/// }
/// ```
pub fn aggregate_to_app(
    entries: &[ClipboardEntry],
    separator: &str,
    template: Option<&str>,
    app: &str,
    action_status: Signal<Option<String>>,
) {
    if entries.is_empty() {
        set_status(action_status, "No entries selected");
        return;
    }

    let combined = entries
        .iter()
        .filter_map(|entry| {
            if let ClipboardEntry::Text { content, id, .. } = entry {
                let formatted = if let Some(tmpl) = template {
                    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                    tmpl.replace("{timestamp}", &now)
                        .replace("{text}", content)
                        .replace("{id}", &id.to_string())
                } else {
                    content.clone()
                };
                Some(formatted)
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join(separator);

    // Handle special "editor" app name to use system default editor
    let app_to_use = if app == "editor" {
        if cfg!(target_os = "windows") {
            "notepad"
        } else {
            "nano"
        }
    } else {
        app
    };

    // Cross-platform app launching
    let result = if cfg!(target_os = "windows") {
        let temp_file = std::env::temp_dir().join("clipboard_bulk_temp.txt");
        std::fs::write(&temp_file, &combined).and_then(|_| {
            std::process::Command::new(app_to_use)
                .arg(&temp_file)
                .spawn()
                .map(|_| ())
        })
    } else {
        // macOS and Linux: write to temp file and open
        let temp_file = std::env::temp_dir().join("clipboard_bulk_temp.txt");
        std::fs::write(&temp_file, &combined).and_then(|_| {
            let opener = if cfg!(target_os = "macos") {
                "open"
            } else {
                "xdg-open"
            };
            std::process::Command::new(opener)
                .arg(&temp_file)
                .spawn()
                .map(|_| ())
        })
    };

    match result {
        Ok(_) => set_status(
            action_status,
            format!(
                "Sent {} entries to {}",
                entries.len(),
                if app == "editor" { "editor" } else { app }
            ),
        ),
        Err(e) => set_status(
            action_status,
            format!("Failed to open {}: {}", app_to_use, e),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_since_minutes() {
        let cutoff = parse_since_cutoff("30m");
        assert!(cutoff.is_some());
        let cutoff = cutoff.unwrap();
        let expected = (Utc::now() - chrono::Duration::minutes(30)).timestamp() as u64;
        // Allow 1 second of variance due to test execution time
        assert!((cutoff as i64 - expected as i64).abs() <= 1);
    }

    #[test]
    fn test_parse_since_hours() {
        let cutoff = parse_since_cutoff("2h");
        assert!(cutoff.is_some());
        let cutoff = cutoff.unwrap();
        let expected = (Utc::now() - chrono::Duration::hours(2)).timestamp() as u64;
        assert!((cutoff as i64 - expected as i64).abs() <= 1);
    }

    #[test]
    fn test_parse_since_days() {
        let cutoff = parse_since_cutoff("7d");
        assert!(cutoff.is_some());
        let cutoff = cutoff.unwrap();
        let expected = (Utc::now() - chrono::Duration::days(7)).timestamp() as u64;
        assert!((cutoff as i64 - expected as i64).abs() <= 1);
    }

    #[test]
    fn test_parse_since_weeks() {
        let cutoff = parse_since_cutoff("2w");
        assert!(cutoff.is_some());
        let cutoff = cutoff.unwrap();
        let expected = (Utc::now() - chrono::Duration::weeks(2)).timestamp() as u64;
        assert!((cutoff as i64 - expected as i64).abs() <= 1);
    }

    #[test]
    fn test_parse_since_today() {
        let cutoff = parse_since_cutoff("today");
        assert!(cutoff.is_some());
        let cutoff = cutoff.unwrap();
        let local = Local::now();
        let start = Local
            .with_ymd_and_hms(local.year(), local.month(), local.day(), 0, 0, 0)
            .single()
            .unwrap();
        let expected = start.with_timezone(&Utc).timestamp() as u64;
        assert_eq!(cutoff, expected);
    }

    #[test]
    fn test_parse_since_yesterday() {
        let cutoff = parse_since_cutoff("yesterday");
        assert!(cutoff.is_some());
        let cutoff = cutoff.unwrap();
        let local = Local::now() - chrono::Duration::days(1);
        let start = Local
            .with_ymd_and_hms(local.year(), local.month(), local.day(), 0, 0, 0)
            .single()
            .unwrap();
        let expected = start.with_timezone(&Utc).timestamp() as u64;
        assert_eq!(cutoff, expected);
    }

    #[test]
    fn test_parse_since_invalid() {
        assert!(parse_since_cutoff("invalid").is_none());
        assert!(parse_since_cutoff("abc").is_none());
        assert!(parse_since_cutoff("").is_none());
    }

    #[test]
    fn test_parse_since_one_minute() {
        let cutoff = parse_since_cutoff("1m");
        assert!(cutoff.is_some());
        let cutoff = cutoff.unwrap();
        let expected = (Utc::now() - chrono::Duration::minutes(1)).timestamp() as u64;
        assert!((cutoff as i64 - expected as i64).abs() <= 1);
    }

    #[test]
    fn test_parse_since_one_week() {
        let cutoff = parse_since_cutoff("1w");
        assert!(cutoff.is_some());
        let cutoff = cutoff.unwrap();
        let expected = (Utc::now() - chrono::Duration::weeks(1)).timestamp() as u64;
        assert!((cutoff as i64 - expected as i64).abs() <= 1);
    }
}
