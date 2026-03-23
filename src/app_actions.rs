use arboard::Clipboard;
use dioxus::prelude::*;
use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::entry::{CachedEntries, ClipboardEntry};
use crate::history::{clear_history, delete_entry};
use crate::source::ClipboardSource;

const STATUS_TIMEOUT_SECS: u64 = 3;
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

    let query = trimmed.to_lowercase();
    match entry {
        ClipboardEntry::Text { content, kind, .. } => {
            content.to_lowercase().contains(&query)
                || kind
                    .as_deref()
                    .map(|kind| kind.to_lowercase().contains(&query))
                    .unwrap_or(false)
                || "text".contains(&query)
        }
        ClipboardEntry::Image { path, .. } => {
            path.to_lowercase().contains(&query) || "image".contains(&query)
        }
    }
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
            error.set(None);
            set_temporary_status(action_status, "History cleared");
        }
        Err(err) => {
            error.set(Some(err));
            set_temporary_status(action_status, "Clear failed");
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
            set_temporary_status(action_status, "Entry deleted");
        }
        Err(err) => {
            error.set(Some(err));
            set_temporary_status(action_status, "Delete failed");
        }
    }
}

pub fn copy_text_to_clipboard(copy_status: Signal<Option<String>>, text: String) {
    let result = Clipboard::new().and_then(|mut cb| cb.set_text(text));
    match result {
        Ok(_) => set_temporary_status(copy_status, "Copied"),
        Err(_) => set_temporary_status(copy_status, "Copy failed"),
    }
}

pub fn set_temporary_status(mut status: Signal<Option<String>>, message: impl Into<String>) {
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
