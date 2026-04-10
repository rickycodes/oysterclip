use dioxus::prelude::*;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;

use super::actions::{
    adjacent_entry_id, confirm_and_delete_entries, confirm_and_delete_entry,
    copy_text_to_clipboard, set_status, DeleteActionState,
};
use crate::config::source::ClipboardSource;
use crate::data::entry::{CachedEntries, ClipboardEntry};
use crate::system::watcher_control::{self, WatcherStatus};
use common::{MSG_WATCHER_PAUSED, MSG_WATCHER_RESUMED};

pub struct CurrentState {
    pub selected_id: Option<i64>,
    pub query: String,
    pub watcher_status: WatcherStatus,
}

pub struct SelectionSignals {
    pub selected_ids: Signal<HashSet<i64>>,
    pub selected_id: Signal<Option<i64>>,
    pub selected_text: Option<String>,
    pub selected_label: &'static str,
}

pub struct UISignals {
    pub show_password: Signal<bool>,
    pub image_overlay_open: Signal<bool>,
    pub help_open: Signal<bool>,
    pub query: Signal<String>,
    pub focus_search: Signal<u32>,
    pub copy_status: Signal<Option<(i64, String)>>,
    pub error: Signal<Option<String>>,
    pub action_status: Signal<Option<String>>,
    pub watcher_status: Signal<WatcherStatus>,
    pub entries: Signal<Vec<ClipboardEntry>>,
}

pub struct Services {
    pub source_for_delete: Arc<ClipboardSource>,
    pub cache_for_delete: Arc<Mutex<Option<CachedEntries>>>,
    pub source_for_watcher: Arc<ClipboardSource>,
}

pub fn create_handler(
    filtered_entries: Vec<ClipboardEntry>,
    current: CurrentState,
    mut selection: SelectionSignals,
    mut ui: UISignals,
    services: Services,
) -> impl FnMut(KeyboardEvent) {
    move |event: KeyboardEvent| {
        let code = event.code();

        match code {
            // Navigation: Arrow keys (Shift+Arrow extends selection)
            Code::ArrowDown | Code::KeyJ => {
                event.prevent_default();
                if let Some(id) = adjacent_entry_id(&filtered_entries, current.selected_id, 1) {
                    if event.modifiers().shift() {
                        let mut set = (selection.selected_ids)();
                        if let Some(cur) = current.selected_id {
                            set.insert(cur);
                        }
                        set.insert(id);
                        selection.selected_ids.set(set);
                    }
                    selection.selected_id.set(Some(id));
                    ui.show_password.set(false);
                    ui.image_overlay_open.set(false);
                }
            }
            Code::ArrowUp | Code::KeyK => {
                event.prevent_default();
                if let Some(id) = adjacent_entry_id(&filtered_entries, current.selected_id, -1) {
                    if event.modifiers().shift() {
                        let mut set = (selection.selected_ids)();
                        if let Some(cur) = current.selected_id {
                            set.insert(cur);
                        }
                        set.insert(id);
                        selection.selected_ids.set(set);
                    }
                    selection.selected_id.set(Some(id));
                    ui.show_password.set(false);
                    ui.image_overlay_open.set(false);
                }
            }
            // Jump to first entry: Home
            Code::Home => {
                event.prevent_default();
                if let Some(id) = filtered_entries.first().map(|entry| entry.id()) {
                    selection.selected_id.set(Some(id));
                    ui.show_password.set(false);
                    ui.image_overlay_open.set(false);
                }
            }
            // Jump to last entry: End
            Code::End => {
                event.prevent_default();
                if let Some(id) = filtered_entries.last().map(|entry| entry.id()) {
                    selection.selected_id.set(Some(id));
                    ui.show_password.set(false);
                    ui.image_overlay_open.set(false);
                }
            }
            // Toggle selection: Space
            Code::Space => {
                if let Some(id) = current.selected_id {
                    event.prevent_default();
                    let mut set = (selection.selected_ids)();
                    if set.contains(&id) {
                        set.remove(&id);
                    } else {
                        set.insert(id);
                    }
                    selection.selected_ids.set(set);
                }
            }
            // Copy to clipboard: Enter, y (yank)
            Code::Enter | Code::KeyY => {
                if let Some(text) = selection.selected_text.clone() {
                    event.prevent_default();
                    if let Some(id) = current.selected_id {
                        copy_text_to_clipboard(ui.copy_status, id, text, selection.selected_label);
                    }
                }
            }
            // Toggle watcher pause/resume: p
            Code::KeyP => {
                event.prevent_default();
                let result = if current.watcher_status.paused {
                    watcher_control::resume(&services.source_for_watcher)
                } else {
                    watcher_control::pause(&services.source_for_watcher)
                };
                match result {
                    Ok(next_status) => {
                        let message = if next_status.paused {
                            MSG_WATCHER_PAUSED
                        } else {
                            MSG_WATCHER_RESUMED
                        };
                        ui.watcher_status.set(next_status);
                        set_status(ui.action_status, message);
                    }
                    Err(err) => {
                        ui.watcher_status.set(WatcherStatus::unavailable(err));
                        set_status(ui.action_status, "Watcher control failed");
                    }
                }
            }
            // Delete: Delete, Backspace, d — bulk if selection active, else single
            Code::Delete | Code::Backspace | Code::KeyD => {
                let ids: Vec<i64> = (selection.selected_ids)().into_iter().collect();
                if !ids.is_empty() {
                    event.prevent_default();
                    ui.image_overlay_open.set(false);
                    let state = DeleteActionState {
                        entries: ui.entries,
                        selected_id: selection.selected_id,
                        selected_ids: selection.selected_ids,
                        error: ui.error,
                        action_status: ui.action_status,
                    };
                    confirm_and_delete_entries(
                        services.source_for_delete.clone(),
                        services.cache_for_delete.clone(),
                        state,
                        ids,
                    );
                } else if let Some(id) = current.selected_id {
                    event.prevent_default();
                    ui.image_overlay_open.set(false);
                    let state = DeleteActionState {
                        entries: ui.entries,
                        selected_id: selection.selected_id,
                        selected_ids: selection.selected_ids,
                        error: ui.error,
                        action_status: ui.action_status,
                    };
                    confirm_and_delete_entry(
                        services.source_for_delete.clone(),
                        services.cache_for_delete.clone(),
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
                    ui.help_open.toggle();
                } else if event.key() == Key::Character("/".to_string()) {
                    event.prevent_default();
                    ui.focus_search.set((ui.focus_search)() + 1);
                }
            }
            // Focus search: Ctrl+F
            Code::KeyF => {
                if event.modifiers().ctrl() {
                    event.prevent_default();
                    ui.focus_search.set((ui.focus_search)() + 1);
                }
            }
            // Escape: close help/overlay → clear selection → clear search
            Code::Escape => {
                event.prevent_default();
                if (ui.help_open)() {
                    ui.help_open.set(false);
                } else if (ui.image_overlay_open)() {
                    ui.image_overlay_open.set(false);
                } else if !(selection.selected_ids)().is_empty() {
                    selection.selected_ids.set(HashSet::new());
                } else if !current.query.is_empty() {
                    ui.query.set(String::new());
                }
            }
            _ => {}
        }
    }
}
