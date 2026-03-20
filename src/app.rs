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
    let mut selected = use_signal(|| None::<usize>);
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

                    let new_len = payload.entries.len();
                    if entries() != payload.entries {
                        entries.set(payload.entries);
                    }

                    match selected() {
                        Some(idx) if idx < new_len => {}
                        _ if new_len == 0 => selected.set(None),
                        _ => {}
                    }
                }

                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    });

    let current_entries = entries();
    let current_selected = selected();
    let detail = current_selected.and_then(|idx| current_entries.get(idx).cloned());

    let handle_select = move |idx: usize| {
        selected.set(Some(idx));
        if copy_status().is_some() {
            copy_status.set(None);
        }
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
                selected.set(None);
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
                selected.set(None);
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
                entries: current_entries.clone(),
                selected: current_selected,
                error: error(),
                action_status: action_status(),
                on_select: handle_select,
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
