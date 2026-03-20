use arboard::Clipboard;
use dioxus::prelude::*;
use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::components::{DetailPane, Sidebar};
use crate::entry::{CachedEntries, ClipboardEntry, ClipboardPayload};
use crate::history::{clear_history, delete_entry, get_clipboard_entries};
use crate::source::ClipboardSource;

const APP_STYLE: &str = include_str!("../styles.css");

#[component]
pub fn App() -> Element {
    let source = use_hook(|| Arc::new(ClipboardSource::from_env()));
    let cache = use_hook(|| Arc::new(Mutex::new(None::<CachedEntries>)));
    let mut entries = use_signal(Vec::<ClipboardEntry>::new);
    let mut selected_id = use_signal(|| None::<i64>);
    let mut query = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);
    let mut copy_status = use_signal(|| None::<String>);
    let mut action_status = use_signal(|| None::<String>);

    let polling_source = source.clone();
    let polling_cache = cache.clone();

    use_future(move || {
        let source = polling_source.clone();
        let cache = polling_cache.clone();
        async move {
            loop {
                let payload = {
                    if let Ok(mut cache_guard) = cache.lock() {
                        get_clipboard_entries(&source, &mut cache_guard)
                    } else {
                        ClipboardPayload {
                            entries: Vec::new(),
                            error: Some("Failed to acquire cache lock.".to_string()),
                        }
                    }
                };

                if let Some(err) = payload.error {
                    if error() != Some(err.clone()) {
                        error.set(Some(err));
                    }
                } else {
                    if error().is_some() {
                        error.set(None);
                    }

                    let new_entries = payload.entries;
                    let new_len = new_entries.len();
                    let selected_still_exists = selected_id()
                        .map(|id| new_entries.iter().any(|entry| entry_id(entry) == id))
                        .unwrap_or(false);
                    if entries() != new_entries {
                        entries.set(new_entries);
                    }

                    if !selected_still_exists || new_len == 0 {
                        selected_id.set(None);
                    }
                }

                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    });

    let current_entries = entries();
    let current_query = query();
    let filtered_entries: Vec<ClipboardEntry> = current_entries
        .iter()
        .filter(|entry| matches_query(entry, &current_query))
        .cloned()
        .collect();
    let current_selected_id = selected_id();
    let detail = current_selected_id.and_then(|id| {
        filtered_entries
            .iter()
            .find(|entry| entry_id(entry) == id)
            .cloned()
    });

    let handle_select = move |id: i64| {
        selected_id.set(Some(id));
        if copy_status().is_some() {
            copy_status.set(None);
        }
    };

    let handle_query_input = move |value: String| {
        query.set(value);
    };

    let source_for_clear = source.clone();
    let cache_for_clear = cache.clone();
    let handle_clear = move |_| {
        let confirmed = MessageDialog::new()
            .set_level(MessageLevel::Warning)
            .set_title("Clear clipboard history?")
            .set_description("This will permanently delete all clipboard history entries.")
            .set_buttons(MessageButtons::OkCancel)
            .show();

        if !matches!(confirmed, MessageDialogResult::Ok) {
            return;
        }

        match clear_history(&source_for_clear) {
            Ok(_) => {
                if let Ok(mut cache_guard) = cache_for_clear.lock() {
                    *cache_guard = None;
                }
                entries.set(Vec::new());
                selected_id.set(None);
                error.set(None);
                action_status.set(Some("History cleared".to_string()));
            }
            Err(err) => {
                error.set(Some(err));
                action_status.set(Some("Clear failed".to_string()));
            }
        }
    };

    let handle_copy_text = move |text: String| {
        let result = Clipboard::new().and_then(|mut cb| cb.set_text(text));
        match result {
            Ok(_) => copy_status.set(Some("Copied".to_string())),
            Err(_) => copy_status.set(Some("Copy failed".to_string())),
        }
    };

    let source_for_delete = source.clone();
    let cache_for_delete = cache.clone();
    let handle_delete = move |id: i64| {
        let confirmed = MessageDialog::new()
            .set_level(MessageLevel::Warning)
            .set_title("Delete clipboard entry?")
            .set_description("This will permanently delete the selected clipboard entry.")
            .set_buttons(MessageButtons::OkCancel)
            .show();

        if !matches!(confirmed, MessageDialogResult::Ok) {
            return;
        }

        match delete_entry(&source_for_delete, id) {
            Ok(_) => {
                if let Ok(mut cache_guard) = cache_for_delete.lock() {
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
                action_status.set(Some("Entry deleted".to_string()));
            }
            Err(err) => {
                error.set(Some(err));
                action_status.set(Some("Delete failed".to_string()));
            }
        }
    };

    rsx! {
        style { "{APP_STYLE}" }
        main { class: "app",
            Sidebar {
                entries: filtered_entries.clone(),
                selected_id: current_selected_id,
                query: current_query,
                error: error(),
                action_status: action_status(),
                on_select: handle_select,
                on_query_input: handle_query_input,
                on_clear: handle_clear,
            }
            DetailPane {
                detail: detail,
                copy_status: copy_status(),
                on_copy_text: handle_copy_text,
                on_delete: handle_delete,
            }
        }
    }
}

fn entry_id(entry: &ClipboardEntry) -> i64 {
    match entry {
        ClipboardEntry::Text { id, .. } | ClipboardEntry::Image { id, .. } => *id,
    }
}

fn matches_query(entry: &ClipboardEntry, query: &str) -> bool {
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
