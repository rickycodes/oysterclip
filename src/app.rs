use arboard::Clipboard;
use dioxus::prelude::*;
use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::components::{DetailPane, DetailState, Sidebar};
use crate::entry::{CachedEntries, ClipboardEntry, ClipboardPayload};
use crate::history::{clear_history, delete_entry, get_clipboard_entries};
use crate::source::ClipboardSource;

const APP_STYLE: &str = include_str!("../styles.css");
const STATUS_TIMEOUT_SECS: u64 = 3;
const STATUS_TIMEOUT: Duration = Duration::from_secs(STATUS_TIMEOUT_SECS);

#[component]
pub fn App() -> Element {
    let source = use_hook(|| Arc::new(ClipboardSource::from_env()));
    let cache = use_hook(|| Arc::new(Mutex::new(None::<CachedEntries>)));
    let mut entries = use_signal(Vec::<ClipboardEntry>::new);
    let mut selected_id = use_signal(|| None::<i64>);
    let mut query = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);
    let mut copy_status = use_signal(|| None::<String>);
    let action_status = use_signal(|| None::<String>);

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
    let selected_text = current_selected_id.and_then(|id| {
        filtered_entries.iter().find_map(|entry| match entry {
            ClipboardEntry::Text { id: entry_id, content, .. } if *entry_id == id => Some(content.clone()),
            _ => None,
        })
    });
    let detail_state = if let Some(message) = error() {
        DetailState::Error(message)
    } else if current_entries.is_empty() {
        DetailState::EmptyHistory
    } else if filtered_entries.is_empty() {
        DetailState::EmptySearch(current_query.clone())
    } else if let Some(entry) = detail {
        DetailState::Entry(entry)
    } else {
        DetailState::Unselected
    };

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
                set_temporary_status(action_status, "History cleared");
            }
            Err(err) => {
                error.set(Some(err));
                set_temporary_status(action_status, "Clear failed");
            }
        }
    };

    let handle_copy_text = {
        let copy_status_signal = copy_status;
        move |text: String| {
            copy_text_to_clipboard(copy_status_signal, text);
        }
    };

    let source_for_delete = source.clone();
    let cache_for_delete = cache.clone();
    let source_for_delete_keys = source_for_delete.clone();
    let cache_for_delete_keys = cache_for_delete.clone();
    let handle_delete = {
        let delete_entries = entries;
        let delete_selected_id = selected_id;
        let delete_error = error;
        let delete_action_status = action_status;
        move |id: i64| {
            confirm_and_delete_entry(
                source_for_delete.clone(),
                cache_for_delete.clone(),
                delete_entries,
                delete_selected_id,
                delete_error,
                delete_action_status,
                id,
            );
        }
    };

    let handle_keydown = {
        let keyboard_entries = filtered_entries.clone();
        let selected_text_for_enter = selected_text.clone();
        let copy_status_for_enter = copy_status;
        move |event: KeyboardEvent| {
            match event.code() {
                Code::ArrowDown => {
                    event.prevent_default();
                    if let Some(id) = adjacent_entry_id(&keyboard_entries, current_selected_id, 1) {
                        selected_id.set(Some(id));
                        if copy_status().is_some() {
                            copy_status.set(None);
                        }
                    }
                }
                Code::ArrowUp => {
                    event.prevent_default();
                    if let Some(id) = adjacent_entry_id(&keyboard_entries, current_selected_id, -1) {
                        selected_id.set(Some(id));
                        if copy_status().is_some() {
                            copy_status.set(None);
                        }
                    }
                }
                Code::Home => {
                    event.prevent_default();
                    if let Some(id) = keyboard_entries.first().map(entry_id) {
                        selected_id.set(Some(id));
                        if copy_status().is_some() {
                            copy_status.set(None);
                        }
                    }
                }
                Code::End => {
                    event.prevent_default();
                    if let Some(id) = keyboard_entries.last().map(entry_id) {
                        selected_id.set(Some(id));
                        if copy_status().is_some() {
                            copy_status.set(None);
                        }
                    }
                }
                Code::Enter => {
                    if let Some(text) = selected_text_for_enter.clone() {
                        event.prevent_default();
                        copy_text_to_clipboard(copy_status_for_enter, text);
                    }
                }
                Code::Delete | Code::Backspace => {
                    if let Some(id) = current_selected_id {
                        event.prevent_default();
                        confirm_and_delete_entry(
                            source_for_delete_keys.clone(),
                            cache_for_delete_keys.clone(),
                            entries,
                            selected_id,
                            error,
                            action_status,
                            id,
                        );
                    }
                }
                _ => {}
            }
        }
    };

    rsx! {
        style { "{APP_STYLE}" }
        main {
            class: "app",
            tabindex: 0,
            onkeydown: handle_keydown,
            Sidebar {
                entries: filtered_entries.clone(),
                total_entries: current_entries.len(),
                selected_id: current_selected_id,
                query: current_query,
                error: error(),
                action_status: action_status(),
                on_select: handle_select,
                on_query_input: handle_query_input,
                on_clear: handle_clear,
            }
            DetailPane {
                state: detail_state,
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


fn adjacent_entry_id(entries: &[ClipboardEntry], selected_id: Option<i64>, direction: isize) -> Option<i64> {
    if entries.is_empty() {
        return None;
    }

    let current_index = selected_id.and_then(|id| entries.iter().position(|entry| entry_id(entry) == id));
    let next_index = match (current_index, direction) {
        (Some(index), step) if step > 0 => (index + 1).min(entries.len() - 1),
        (Some(index), _) => index.saturating_sub(1),
        (None, step) if step > 0 => 0,
        (None, _) => entries.len() - 1,
    };

    entries.get(next_index).map(entry_id)
}


fn confirm_and_delete_entry(
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


fn set_temporary_status(mut status: Signal<Option<String>>, message: impl Into<String>) {
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


fn copy_text_to_clipboard(copy_status: Signal<Option<String>>, text: String) {
    let result = Clipboard::new().and_then(|mut cb| cb.set_text(text));
    match result {
        Ok(_) => set_temporary_status(copy_status, "Copied"),
        Err(_) => set_temporary_status(copy_status, "Copy failed"),
    }
}
