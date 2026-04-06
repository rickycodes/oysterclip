use dioxus::prelude::*;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;

use crate::data::entry::{CachedEntries, ClipboardEntry};
use crate::system::watcher_control::{self, WatcherStatus};
use crate::config::source::ClipboardSource;
use common::{MSG_WATCHER_PAUSED, MSG_WATCHER_RESUMED};
use super::actions::{copy_text_to_clipboard, confirm_and_delete_entry, confirm_and_delete_entries, set_status, DeleteActionState, adjacent_entry_id};

/// Factory function that creates a keyboard event handler closure
/// Factory function that creates a keyboard event handler closure
pub fn create_handler(
    filtered_entries: Vec<ClipboardEntry>,
    current_selected_id: Option<i64>,
    mut selected_ids: Signal<HashSet<i64>>,
    mut selected_id: Signal<Option<i64>>,
    selected_text: Option<String>,
    selected_label: &'static str,
    current_query: String,
    current_watcher_status: WatcherStatus,
    mut show_password: Signal<bool>,
    mut image_overlay_open: Signal<bool>,
    copy_status: Signal<Option<(i64, String)>>,
    mut watcher_status: Signal<WatcherStatus>,
    error: Signal<Option<String>>,
    action_status: Signal<Option<String>>,
    mut help_open: Signal<bool>,
    mut query: Signal<String>,
    mut focus_search: Signal<u32>,
    entries: Signal<Vec<ClipboardEntry>>,
    source_for_delete_keys: Arc<ClipboardSource>,
    cache_for_delete_keys: Arc<Mutex<Option<CachedEntries>>>,
    source_for_watcher_key: Arc<ClipboardSource>,
) -> impl FnMut(KeyboardEvent) {
    move |event: KeyboardEvent| {
        let code = event.code();

        match code {
            // Navigation: Arrow keys (Shift+Arrow extends selection)
            Code::ArrowDown | Code::KeyJ => {
                event.prevent_default();
                if let Some(id) = adjacent_entry_id(&filtered_entries, current_selected_id, 1) {
                    if event.modifiers().shift() {
                        let mut set = selected_ids();
                        if let Some(cur) = current_selected_id {
                            set.insert(cur);
                        }
                        set.insert(id);
                        selected_ids.set(set);
                    }
                    selected_id.set(Some(id));
                    show_password.set(false);
                    image_overlay_open.set(false);
                }
            }
            Code::ArrowUp | Code::KeyK => {
                event.prevent_default();
                if let Some(id) = adjacent_entry_id(&filtered_entries, current_selected_id, -1) {
                    if event.modifiers().shift() {
                        let mut set = selected_ids();
                        if let Some(cur) = current_selected_id {
                            set.insert(cur);
                        }
                        set.insert(id);
                        selected_ids.set(set);
                    }
                    selected_id.set(Some(id));
                    show_password.set(false);
                    image_overlay_open.set(false);
                }
            }
            // Jump to first entry: Home
            Code::Home => {
                event.prevent_default();
                if let Some(id) = filtered_entries.first().map(|entry| entry.id()) {
                    selected_id.set(Some(id));
                    show_password.set(false);
                    image_overlay_open.set(false);
                }
            }
            // Jump to last entry: End
            Code::End => {
                event.prevent_default();
                if let Some(id) = filtered_entries.last().map(|entry| entry.id()) {
                    selected_id.set(Some(id));
                    show_password.set(false);
                    image_overlay_open.set(false);
                }
            }
            // Toggle selection: Space
            Code::Space => {
                if let Some(id) = current_selected_id {
                    event.prevent_default();
                    let mut set = selected_ids();
                    if set.contains(&id) {
                        set.remove(&id);
                    } else {
                        set.insert(id);
                    }
                    selected_ids.set(set);
                }
            }
            // Copy to clipboard: Enter, y (yank)
            Code::Enter | Code::KeyY => {
                if let Some(text) = selected_text.clone() {
                    event.prevent_default();
                    if let Some(id) = current_selected_id {
                        copy_text_to_clipboard(
                            copy_status,
                            id,
                            text,
                            selected_label,
                        );
                    }
                }
            }
            // Toggle watcher pause/resume: p
            Code::KeyP => {
                event.prevent_default();
                let result = if current_watcher_status.paused {
                    watcher_control::resume(&source_for_watcher_key)
                } else {
                    watcher_control::pause(&source_for_watcher_key)
                };
                match result {
                    Ok(next_status) => {
                        let message = if next_status.paused {
                            MSG_WATCHER_PAUSED
                        } else {
                            MSG_WATCHER_RESUMED
                        };
                        watcher_status.set(next_status);
                        set_status(action_status, message);
                    }
                    Err(err) => {
                        watcher_status.set(WatcherStatus::unavailable(err));
                        set_status(action_status, "Watcher control failed");
                    }
                }
            }
            // Delete: Delete, Backspace, d — bulk if selection active, else single
            Code::Delete | Code::Backspace | Code::KeyD => {
                let ids: Vec<i64> = selected_ids().into_iter().collect();
                if !ids.is_empty() {
                    event.prevent_default();
                    image_overlay_open.set(false);
                    let state = DeleteActionState {
                        entries,
                        selected_id,
                        selected_ids,
                        error,
                        action_status,
                    };
                    confirm_and_delete_entries(
                        source_for_delete_keys.clone(),
                        cache_for_delete_keys.clone(),
                        state,
                        ids,
                    );
                } else if let Some(id) = current_selected_id {
                    event.prevent_default();
                    image_overlay_open.set(false);
                    let state = DeleteActionState {
                        entries,
                        selected_id,
                        selected_ids,
                        error,
                        action_status,
                    };
                    confirm_and_delete_entry(
                        source_for_delete_keys.clone(),
                        cache_for_delete_keys.clone(),
                        state,
                        id,
                    );
                }
            }
            // Focus search: /
            // Show help: ? (Shift+/)
            Code::Slash => {
                // '/' focuses search; '?' (Shift+/) opens help
                if event.key() == Key::Character("?".to_string()) {
                    event.prevent_default();
                    help_open.toggle();
                } else if event.key() == Key::Character("/".to_string()) {
                    event.prevent_default();
                    focus_search.set(focus_search() + 1);
                }
            }
            // Focus search: Ctrl+F
            Code::KeyF => {
                if event.modifiers().ctrl() {
                    event.prevent_default();
                    focus_search.set(focus_search() + 1);
                }
            }
            // Escape: close help/overlay → clear selection → clear search
            Code::Escape => {
                event.prevent_default();
                if help_open() {
                    help_open.set(false);
                } else if image_overlay_open() {
                    image_overlay_open.set(false);
                } else if !selected_ids().is_empty() {
                    selected_ids.set(HashSet::new());
                } else if !current_query.is_empty() {
                    query.set(String::new());
                }
            }
            _ => {}
        }
    }
}
