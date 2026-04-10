use dioxus::prelude::*;
use std::collections::HashSet;
use std::rc::Rc;

use crate::app::actions::{
    aggregate_to_app, confirm_and_clear_history, confirm_and_delete_entries,
    confirm_and_delete_entry, copy_text_to_clipboard, set_status, DeleteActionState,
};
use crate::app::state::use_app_state;
use crate::config::settings::AppConfig;
use crate::data::format::is_password;
use crate::system::watcher_control;
use crate::ui::help_modal::HelpModal;
use crate::ui::theme::{load_theme, save_theme};
use crate::ui::{DetailPane, ImageOverlay, Sidebar};
use common::{MSG_WATCHER_PAUSED, MSG_WATCHER_RESUMED};

const APP_STYLE: &str = include_str!("../../styles.css");

#[component]
pub fn App() -> Element {
    let state = use_app_state();

    // Load config once at app startup
    let config = AppConfig::load();
    let notepad_handler = config.get_handler("notepad");

    let source = state.source.clone();
    let cache = state.cache.clone();
    let entries = state.entries;
    let mut selected_id = state.selected_id;
    let mut selected_ids = state.selected_ids;
    let mut query = state.query;
    let error = state.error;
    let copy_status = state.copy_status;
    let action_status = state.action_status;
    let mut show_password = state.show_password;
    let mut image_overlay_open = use_signal(|| false);
    let mut help_open = use_signal(|| false);
    let mut theme = use_signal(load_theme);
    let focus_search = use_signal(|| 0u32);
    let mut main_ref: Signal<Option<Rc<MountedData>>> = use_signal(|| None);

    use_effect(move || {
        if let Some(el) = main_ref() {
            spawn(async move {
                let _ = el.set_focus(true).await;
            });
        }
    });
    let auth_cache = state.auth_cache;
    let link_previews = state.link_previews;
    let mut watcher_status = state.watcher_status;
    let filtered_entries = state.filtered_entries;
    let current_selected_id = state.current_selected_id;
    let current_query = state.current_query.clone();
    let detail_state = state.detail_state;
    let selected_text = state.selected_text;
    let selected_label = state.selected_label;
    let total_entries = state.total_entries;
    let current_watcher_status = state.current_watcher_status;
    let current_selected_ids: Vec<i64> = selected_ids().into_iter().collect();
    let overlay_image_src = match &detail_state {
        crate::ui::DetailState::Entry(crate::data::entry::ClipboardEntry::Image {
            data_url: Some(src),
            ..
        }) => Some(src.clone()),
        _ => None,
    };

    let handle_select = move |id: i64| {
        selected_id.set(Some(id));
        show_password.set(false);
        image_overlay_open.set(false);
    };

    let handle_query_input = move |value: String| {
        query.set(value);
        image_overlay_open.set(false);
    };

    let source_for_clear = source.clone();
    let cache_for_clear = cache.clone();
    let handle_clear = move |_| {
        let state = DeleteActionState {
            entries,
            selected_id,
            selected_ids,
            error,
            action_status,
        };
        confirm_and_clear_history(source_for_clear.clone(), cache_for_clear.clone(), state);
    };

    let handle_copy_text = {
        let copy_status_signal = copy_status;
        move |(entry_id, text, label): (i64, String, &'static str)| {
            copy_text_to_clipboard(copy_status_signal, entry_id, text, label);
        }
    };

    let handle_open_image = move |_| {
        image_overlay_open.set(true);
    };

    let handle_open_editor = {
        let filtered_entries = filtered_entries.clone();
        move |id: i64| {
            if let Some(crate::data::entry::ClipboardEntry::Text { content, .. }) = filtered_entries
                .iter()
                .find(|entry| entry.id() == id)
                .cloned()
            {
                aggregate_to_app(
                    &[crate::data::entry::ClipboardEntry::Text {
                        id,
                        timestamp: 0,
                        content,
                        kind: None,
                    }],
                    "",
                    None,
                    "editor",
                    action_status,
                );
            }
        }
    };

    let source_for_delete = source.clone();
    let cache_for_delete = cache.clone();
    let source_for_delete_keys = source_for_delete.clone();
    let cache_for_delete_keys = cache_for_delete.clone();
    let handle_delete = {
        let delete_entries = entries;
        let delete_selected_id = selected_id;
        let delete_selected_ids = selected_ids;
        let delete_error = error;
        let delete_action_status = action_status;
        move |id: i64| {
            image_overlay_open.set(false);
            let state = DeleteActionState {
                entries: delete_entries,
                selected_id: delete_selected_id,
                selected_ids: delete_selected_ids,
                error: delete_error,
                action_status: delete_action_status,
            };
            confirm_and_delete_entry(
                source_for_delete.clone(),
                cache_for_delete.clone(),
                state,
                id,
            );
        }
    };

    let source_for_bulk = source.clone();
    let cache_for_bulk = cache.clone();
    let handle_delete_selected = move |_| {
        let ids: Vec<i64> = selected_ids().into_iter().collect();
        if !ids.is_empty() {
            let state = DeleteActionState {
                entries,
                selected_id,
                selected_ids,
                error,
                action_status,
            };
            confirm_and_delete_entries(source_for_bulk.clone(), cache_for_bulk.clone(), state, ids);
        }
    };

    let handle_clear_selection = move |_| {
        selected_ids.set(HashSet::new());
    };

    let handle_send_to_notepad = {
        let filtered_entries = filtered_entries.clone();
        let selected_ids_set = selected_ids();
        let handler = notepad_handler.clone();
        move |_| {
            if let Some(handler) = handler.clone() {
                let entries_to_send: Vec<_> = filtered_entries
                    .iter()
                    .filter(|entry| selected_ids_set.contains(&entry.id()))
                    .cloned()
                    .collect();

                aggregate_to_app(
                    &entries_to_send,
                    handler.separator.as_deref().unwrap_or("\n---\n"),
                    handler.template.as_deref(),
                    handler.app.as_deref().unwrap_or("editor"),
                    action_status,
                );
                selected_ids.set(HashSet::new());
            }
        }
    };

    let source_for_watcher = source.clone();
    let source_for_watcher_key = source.clone();
    let handle_toggle_watcher = move |_| {
        let result = if current_watcher_status.paused {
            watcher_control::resume(&source_for_watcher)
        } else {
            watcher_control::pause(&source_for_watcher)
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
                watcher_status.set(crate::system::watcher_control::WatcherStatus::unavailable(
                    err,
                ));
                set_status(action_status, "Watcher control failed");
            }
        }
    };

    let handle_keydown = crate::app::keyboard_shortcuts::create_handler(
        filtered_entries.clone(),
        crate::app::keyboard_shortcuts::CurrentState {
            selected_id: current_selected_id,
            query: current_query.clone(),
            watcher_status: current_watcher_status.clone(),
        },
        crate::app::keyboard_shortcuts::SelectionSignals {
            selected_ids,
            selected_id,
            selected_text,
            selected_label,
        },
        crate::app::keyboard_shortcuts::UISignals {
            show_password,
            image_overlay_open,
            help_open,
            query,
            focus_search,
            copy_status,
            error,
            action_status,
            watcher_status,
            entries,
        },
        crate::app::keyboard_shortcuts::Services {
            source_for_delete: source_for_delete_keys.clone(),
            cache_for_delete: cache_for_delete_keys.clone(),
            source_for_watcher: source_for_watcher_key.clone(),
        },
    );

    // Check if any selected entry is a password
    let selected_contains_password = current_selected_ids.iter().any(|id| {
        entries()
            .iter()
            .find(|e| e.id() == *id)
            .and_then(|e| match e {
                crate::data::entry::ClipboardEntry::Text { content, .. } => {
                    Some(is_password(content))
                }
                _ => None,
            })
            .unwrap_or(false)
    });

    rsx! {
        style { "{APP_STYLE}" }
        if image_overlay_open() || help_open() {
            style { "body {{ overflow: hidden; }}" }
        }
        main {
            class: format!("app {}", theme().class_name()),
            tabindex: 0,
            onmounted: move |e| main_ref.set(Some(e.data())),
            onkeydown: handle_keydown,
            oncontextmenu: move |_event| {
                #[cfg(not(debug_assertions))]
                _event.prevent_default();
            },
            Sidebar {
                entries: filtered_entries.clone(),
                total_entries,
                selected_id: current_selected_id,
                query: current_query,
                error: error(),
                action_status: action_status(),
                focus_search,
                selected_ids: current_selected_ids,
                on_select: handle_select,
                on_query_input: handle_query_input,
                on_clear: handle_clear,
                on_delete_selected: handle_delete_selected,
                on_clear_selection: handle_clear_selection,
                on_send_to_notepad: handle_send_to_notepad,
                show_notepad_button: notepad_handler.is_some(),
                notepad_button_disabled: selected_contains_password,
            }
            DetailPane {
                state: detail_state,
                copy_status: copy_status().and_then(|(id, msg)| {
                    if Some(id) == current_selected_id { Some(msg) } else { None }
                }),
                show_password,
                auth_cache,
                action_status,
                link_previews,
                on_copy_text: handle_copy_text,
                on_delete: handle_delete,
                on_open_image: handle_open_image,
                on_open_editor: handle_open_editor,
            }
        }
        if let Some(src) = overlay_image_src {
            ImageOverlay {
                src,
                is_open: image_overlay_open(),
                on_close: move |_| image_overlay_open.set(false),
            }
        }
        HelpModal {
            is_open: help_open(),
            on_close: move |_| help_open.set(false),
            current_theme: theme(),
            on_theme_toggle: move |_| {
                let new_theme = theme().toggle();
                theme.set(new_theme);
                save_theme(new_theme);
            },
            watcher_status: current_watcher_status,
            on_toggle_watcher: handle_toggle_watcher,
        }
    }
}
