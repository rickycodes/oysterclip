use dioxus::prelude::*;

use super::icon::Icon;
use super::search_bar::SearchBar;
use crate::config::APP_NAME;
use crate::data::entry::ClipboardEntry;
use crate::data::format::{
    entry_icon_name, entry_label, extract_single_url, format_relative_timestamp, is_image_data_uri,
    is_password, preview_text,
};
use common::{TEXT_KIND_JSON, TEXT_KIND_PATH};

/// Check if content is a valid JSON object or array (not just a string).
fn is_valid_json_object_or_array(content: &str) -> bool {
    matches!(
        serde_json::from_str::<serde_json::Value>(content),
        Ok(serde_json::Value::Object(_)) | Ok(serde_json::Value::Array(_))
    )
}

#[component]
pub fn Sidebar(
    entries: Vec<ClipboardEntry>,
    total_entries: usize,
    selected_id: Option<i64>,
    query: String,
    error: Option<String>,
    action_status: Option<String>,
    focus_search: ReadSignal<u32>,
    selected_ids: Vec<i64>,
    on_select: EventHandler<i64>,
    on_query_input: EventHandler<String>,
    on_clear: EventHandler<()>,
    on_delete_selected: EventHandler<()>,
    on_clear_selection: EventHandler<()>,
    on_send_to_notepad: EventHandler<()>,
    show_notepad_button: bool,
    #[props(default)] notepad_button_disabled: bool,
) -> Element {
    rsx! {
        aside { class: "sidebar",
            div { class: "sidebar-header",
                h1 { "{APP_NAME}" }
                div { class: "sidebar-header-actions",
                    span { class: "sidebar-count", "{total_entries} entries" }
                    button {
                        class: "sidebar-clear-btn",
                        onclick: move |_| on_clear.call(()),
                        "Clear"
                    }
                }
            }
            SearchBar {
                query: query.clone(),
                focus_search,
                on_query_input,
            }
            if let Some(err) = error {
                div { class: "sidebar-error",
                    strong { "Load issue" }
                    div { "{err}" }
                }
            }
            if let Some(status) = action_status {
                div { class: "sidebar-status", "{status}" }
            }
            if !selected_ids.is_empty() {
                div { class: "selection-toolbar",
                    span { class: "selection-count",
                        "{selected_ids.len()} selected"
                    }
                    if show_notepad_button {
                        button {
                            class: "selection-notepad-btn",
                            disabled: notepad_button_disabled,
                            onclick: move |_| on_send_to_notepad.call(()),
                            "Notepad"
                        }
                    }
                    button {
                        class: "selection-delete-btn",
                        onclick: move |_| on_delete_selected.call(()),
                        "Delete"
                    }
                    button {
                        class: "selection-clear-btn",
                        onclick: move |_| on_clear_selection.call(()),
                        "✕"
                    }
                }
            }
            div { class: "entry-list",
                if entries.is_empty() {
                    div { class: "sidebar-empty",
                        if query.is_empty() {
                            "Clipboard history will appear here once new entries are captured."
                        } else {
                            "Try a different search term."
                        }
                    }
                }
                for entry in entries.iter() {
                    {
                        let entry_id = entry.id();
                        let is_active = Some(entry_id) == selected_id;
                        let is_checked = selected_ids.contains(&entry_id);
                        let type_class = match entry {
                            ClipboardEntry::Image { .. } => "entry-card-image",
                            ClipboardEntry::Text { content, kind, .. } => {
                                if is_password(content) {
                                    "entry-card-pass"
                                } else if is_image_data_uri(content) {
                                    "entry-card-image"
                                } else if kind.as_deref() == Some(TEXT_KIND_JSON) && is_valid_json_object_or_array(content) {
                                    "entry-card-json"
                                } else if kind.as_deref() == Some(TEXT_KIND_PATH) {
                                    "entry-card-path"
                                } else if extract_single_url(content).is_some() {
                                    "entry-card-url"
                                } else {
                                    ""
                                }
                            }
                        };
                        let mut class = if is_active { "entry-card active".to_string() } else { "entry-card".to_string() };
                        if !type_class.is_empty() {
                            class.push(' ');
                            class.push_str(type_class);
                        }
                        if is_checked {
                            class.push_str(" entry-card-checked");
                        }
                         let preview = match entry {
                            ClipboardEntry::Text { content, kind, .. } => {
                                if kind.as_deref() == Some(TEXT_KIND_JSON) && is_valid_json_object_or_array(content) {
                                    // Minify JSON to a single line for sidebar preview
                                    let minified = content.split_whitespace().collect::<Vec<_>>().join(" ");
                                    preview_text(&minified, 56)
                                } else {
                                    preview_text(content, 56)
                                }
                            }
                            ClipboardEntry::Image { path, hash, .. } => {
                                let preview_source = path
                                    .clone()
                                    .unwrap_or_else(|| hash.to_string());
                                preview_text(&preview_source, 56)
                            }
                        };
                        let timestamp = match entry {
                            ClipboardEntry::Text { timestamp, .. } => *timestamp,
                            ClipboardEntry::Image { timestamp, .. } => *timestamp,
                        };
                        rsx! {
                            button { class: "{class}", onclick: move |_| on_select.call(entry_id),
                                div { class: "entry-title",
                                    svg { class: "entry-icon",
                                        view_box: "0 0 24 24",
                                        Icon { name: entry_icon_name(entry) }
                                    }
                                    span { "{entry_label(entry)}" }
                                }
                                div { class: "entry-preview", "{preview}" }
                                div { class: "entry-ts", "{format_relative_timestamp(timestamp)}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
