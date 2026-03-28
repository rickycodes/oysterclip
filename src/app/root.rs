use dioxus::prelude::*;
use std::collections::HashSet;

use crate::app::actions::{
    adjacent_entry_id, confirm_and_clear_history, confirm_and_delete_entries,
    confirm_and_delete_entry, copy_text_to_clipboard, set_status,
};
use crate::app::state::use_app_state;
use crate::ui::{DetailPane, Sidebar};
use crate::ui::help_modal::HelpModal;
use crate::ui::theme::{load_theme, save_theme};
use crate::system::watcher_control;

const APP_STYLE: &str = include_str!("../../styles.css");

#[component]
pub fn App() -> Element {
    let state = use_app_state();
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
    let mut focus_search = use_signal(|| 0u32);
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
        confirm_and_clear_history(
            source_for_clear.clone(),
            cache_for_clear.clone(),
            entries,
            selected_id,
            selected_ids,
            error,
            action_status,
        );
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
            image_overlay_open.set(false);
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

    let source_for_bulk = source.clone();
    let cache_for_bulk = cache.clone();
    let handle_delete_selected = move |_| {
        let ids: Vec<i64> = selected_ids().into_iter().collect();
        if !ids.is_empty() {
            confirm_and_delete_entries(
                source_for_bulk.clone(),
                cache_for_bulk.clone(),
                entries,
                selected_id,
                selected_ids,
                error,
                action_status,
                ids,
            );
        }
    };

    let handle_clear_selection = move |_| {
        selected_ids.set(HashSet::new());
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
                    "Watcher paused"
                } else {
                    "Watcher resumed"
                };
                watcher_status.set(next_status);
                set_status(action_status, message);
            }
            Err(err) => {
                watcher_status.set(crate::system::watcher_control::WatcherStatus::unavailable(err));
                set_status(action_status, "Watcher control failed");
            }
        }
    };

    let handle_keydown = {
        let keyboard_entries = filtered_entries.clone();
        let selected_text_for_enter = selected_text.clone();
        let selected_label_for_enter = selected_label;
        let copy_status_for_enter = copy_status;
        let current_query_for_escape = current_query.clone();
        move |event: KeyboardEvent| {
            let code = event.code();
            
            match code {
                // Navigation: Arrow keys (Shift+Arrow extends selection)
                Code::ArrowDown | Code::KeyJ => {
                    event.prevent_default();
                    if let Some(id) = adjacent_entry_id(&keyboard_entries, current_selected_id, 1) {
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
                    if let Some(id) = adjacent_entry_id(&keyboard_entries, current_selected_id, -1) {
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
                    if let Some(id) = keyboard_entries.first().map(|entry| match entry {
                        crate::data::entry::ClipboardEntry::Text { id, .. }
                        | crate::data::entry::ClipboardEntry::Image { id, .. } => *id,
                    }) {
                        selected_id.set(Some(id));
                        show_password.set(false);
                        image_overlay_open.set(false);
                    }
                }
                // Jump to last entry: End
                Code::End => {
                    event.prevent_default();
                    if let Some(id) = keyboard_entries.last().map(|entry| match entry {
                        crate::data::entry::ClipboardEntry::Text { id, .. }
                        | crate::data::entry::ClipboardEntry::Image { id, .. } => *id,
                    }) {
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
                    if let Some(text) = selected_text_for_enter.clone() {
                        event.prevent_default();
                        if let Some(id) = current_selected_id {
                            copy_text_to_clipboard(copy_status_for_enter, id, text, selected_label_for_enter);
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
                            let message = if next_status.paused { "Watcher paused" } else { "Watcher resumed" };
                            watcher_status.set(next_status);
                            set_status(action_status, message);
                        }
                        Err(err) => {
                            watcher_status.set(crate::system::watcher_control::WatcherStatus::unavailable(err));
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
                        confirm_and_delete_entries(
                            source_for_delete_keys.clone(),
                            cache_for_delete_keys.clone(),
                            entries,
                            selected_id,
                            selected_ids,
                            error,
                            action_status,
                            ids,
                        );
                    } else if let Some(id) = current_selected_id {
                        event.prevent_default();
                        image_overlay_open.set(false);
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
                    } else if !current_query_for_escape.is_empty() {
                        query.set(String::new());
                    }
                }
                _ => {}
            }
        }
    };

    rsx! {
        style { "{APP_STYLE}" }
        if image_overlay_open() || help_open() {
            style { "body {{ overflow: hidden; }}" }
        }
        main {
            class: format!("app {}", theme().class_name()),
            tabindex: 0,
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
            }
        }
        if let Some(src) = overlay_image_src {
            div {
                class: if image_overlay_open() { "image-overlay is-open" } else { "image-overlay" },
                onclick: move |_| image_overlay_open.set(false),
                aria_hidden: if image_overlay_open() { "false" } else { "true" },
                button {
                    class: "image-overlay-close",
                    onclick: move |_| image_overlay_open.set(false),
                    aria_label: "Close image overlay",
                    tabindex: if image_overlay_open() { "0" } else { "-1" },
                    svg {
                        class: "image-overlay-close-icon",
                        view_box: "0 0 24 24",
                        width: "44",
                        height: "44",
                        stroke_width: "2",
                        stroke: "currentColor",
                        fill: "none",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        path {
                            d: "M18 6L6 18M6 6l12 12",
                        }
                    }
                }
                div {
                    class: "image-overlay-dialog",
                    img {
                        class: "image-overlay-image",
                        src,
                        alt: "Clipboard image expanded",
                        onclick: move |event| event.stop_propagation(),
                    }
                }
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
