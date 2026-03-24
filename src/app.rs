use dioxus::prelude::*;

use crate::app_actions::{
    adjacent_entry_id, confirm_and_clear_history, confirm_and_delete_entry, copy_text_to_clipboard,
};
use crate::app_state::use_app_state;
use crate::components::{DetailPane, Sidebar};

const APP_STYLE: &str = include_str!("../styles.css");

#[component]
pub fn App() -> Element {
    let state = use_app_state();
    let source = state.source.clone();
    let cache = state.cache.clone();
    let entries = state.entries;
    let mut selected_id = state.selected_id;
    let mut query = state.query;
    let error = state.error;
    let mut copy_status = state.copy_status;
    let action_status = state.action_status;
    let mut show_password = state.show_password;
    let auth_cache = state.auth_cache.clone();
    let filtered_entries = state.filtered_entries;
    let current_selected_id = state.current_selected_id;
    let current_query = state.current_query;
    let detail_state = state.detail_state;
    let selected_text = state.selected_text;
    let total_entries = state.total_entries;

    let handle_select = move |id: i64| {
        selected_id.set(Some(id));
        show_password.set(false);
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
        confirm_and_clear_history(
            source_for_clear.clone(),
            cache_for_clear.clone(),
            entries,
            selected_id,
            error,
            action_status,
        );
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
        move |event: KeyboardEvent| match event.code() {
            Code::ArrowDown => {
                event.prevent_default();
                if let Some(id) = adjacent_entry_id(&keyboard_entries, current_selected_id, 1) {
                    selected_id.set(Some(id));
                    show_password.set(false);
                    if copy_status().is_some() {
                        copy_status.set(None);
                    }
                }
            }
            Code::ArrowUp => {
                event.prevent_default();
                if let Some(id) = adjacent_entry_id(&keyboard_entries, current_selected_id, -1) {
                    selected_id.set(Some(id));
                    show_password.set(false);
                    if copy_status().is_some() {
                        copy_status.set(None);
                    }
                }
            }
            Code::Home => {
                event.prevent_default();
                if let Some(id) = keyboard_entries.first().map(|entry| match entry {
                    crate::entry::ClipboardEntry::Text { id, .. }
                    | crate::entry::ClipboardEntry::Image { id, .. } => *id,
                }) {
                    selected_id.set(Some(id));
                    show_password.set(false);
                    if copy_status().is_some() {
                        copy_status.set(None);
                    }
                }
            }
            Code::End => {
                event.prevent_default();
                if let Some(id) = keyboard_entries.last().map(|entry| match entry {
                    crate::entry::ClipboardEntry::Text { id, .. }
                    | crate::entry::ClipboardEntry::Image { id, .. } => *id,
                }) {
                    selected_id.set(Some(id));
                    show_password.set(false);
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
    };

    rsx! {
        style { "{APP_STYLE}" }
        main { class: "app", tabindex: 0, onkeydown: handle_keydown,
            Sidebar {
                entries: filtered_entries.clone(),
                total_entries,
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
                show_password,
                auth_cache,
                on_copy_text: handle_copy_text,
                on_delete: handle_delete,
            }
        }
    }
}
