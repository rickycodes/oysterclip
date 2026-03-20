use dioxus::prelude::*;

use crate::entry::ClipboardEntry;
use crate::format::{
    entry_label, format_timestamp, image_data_uri_summary, is_image_data_uri, preview_text,
};

#[derive(Clone, PartialEq)]
pub enum DetailState {
    Error(String),
    EmptyHistory,
    EmptySearch(String),
    Unselected,
    Entry(ClipboardEntry),
}

#[component]
pub fn Sidebar(
    entries: Vec<ClipboardEntry>,
    total_entries: usize,
    selected_id: Option<i64>,
    query: String,
    error: Option<String>,
    action_status: Option<String>,
    on_select: EventHandler<i64>,
    on_query_input: EventHandler<String>,
    on_clear: EventHandler<()>,
) -> Element {
    rsx! {
        aside { class: "sidebar",
            div { class: "sidebar-header",
                h1 { "Clipboard" }
                div { class: "sidebar-header-actions",
                    span { class: "sidebar-count", "{total_entries} entries" }
                    button {
                        class: "sidebar-clear-btn",
                        onclick: move |_| on_clear.call(()),
                        "Clear"
                    }
                }
            }
            div { class: "sidebar-search",
                input {
                    class: "sidebar-search-input",
                    r#type: "search",
                    placeholder: "Search history",
                    value: "{query}",
                    oninput: move |event| on_query_input.call(event.value().to_string()),
                    onkeydown: move |event| event.stop_propagation(),
                }
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
                        let entry_id = match entry {
                            ClipboardEntry::Text { id, .. } | ClipboardEntry::Image { id, .. } => *id,
                        };
                        let is_active = Some(entry_id) == selected_id;
                        let class = if is_active { "entry-card active" } else { "entry-card" };
                        let preview = match entry {
                            ClipboardEntry::Text { content, .. } => preview_text(content, 56),
                            ClipboardEntry::Image { path, .. } => preview_text(path, 56),
                        };
                        let timestamp = match entry {
                            ClipboardEntry::Text { timestamp, .. } => *timestamp,
                            ClipboardEntry::Image { timestamp, .. } => *timestamp,
                        };
                        rsx! {
                            button {
                                class: "{class}",
                                onclick: move |_| on_select.call(entry_id),
                                div { class: "entry-title", "{entry_label(entry)}" }
                                div { class: "entry-preview", "{preview}" }
                                div { class: "entry-ts", "{format_timestamp(timestamp)}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn DetailPane(
    state: DetailState,
    copy_status: Option<String>,
    on_copy_text: EventHandler<String>,
    on_delete: EventHandler<i64>,
) -> Element {
    rsx! {
        section { class: "content",
            {
                match state {
                    DetailState::Entry(ClipboardEntry::Text { id, timestamp, content, .. }) => {
                        let text = content.clone();
                        let is_data_uri = is_image_data_uri(&content);
                        let summary = if is_data_uri {
                            Some(image_data_uri_summary(&content))
                        } else {
                            None
                        };
                        let display_text = if is_data_uri {
                            preview_text(&content, 96)
                        } else {
                            content.clone()
                        };
                        rsx! {
                            div { class: "detail",
                                div { class: "detail-meta",
                                    span { class: "detail-type", "Text" }
                                    span { class: "detail-ts", "Timestamp: {format_timestamp(timestamp)}" }
                                }
                                if let Some(summary) = summary {
                                    div { class: "detail-note", "{summary}. Copy still uses the full value." }
                                }
                                pre { class: if is_data_uri { "detail-text detail-text-truncated" } else { "detail-text" }, "{display_text}" }
                                div { class: "detail-actions",
                                    button {
                                        class: "detail-copy-btn",
                                        onclick: move |_| on_copy_text.call(text.clone()),
                                        "Copy"
                                    }
                                    button {
                                        class: "detail-delete-btn",
                                        onclick: move |_| on_delete.call(id),
                                        "Delete"
                                    }
                                    if let Some(status) = copy_status.clone() {
                                        span { class: "detail-copy-status", "{status}" }
                                    }
                                }
                            }
                        }
                    },
                    DetailState::Entry(ClipboardEntry::Image {
                        id,
                        timestamp,
                        path,
                        hash,
                        data_url,
                    }) => rsx! {
                        div { class: "detail",
                            div { class: "detail-meta",
                                span { class: "detail-type", "Image" }
                                span { class: "detail-ts", "Timestamp: {format_timestamp(timestamp)}" }
                            }
                            div { class: "detail-image-wrap",
                                if let Some(src) = data_url {
                                    img { class: "detail-image", src: src, alt: "Clipboard image" }
                                } else {
                                    div { class: "detail-image-missing", "Image data not available." }
                                }
                            }
                            div { class: "detail-footer",
                                span { "Path: {path}" }
                                span { "Hash: {hash}" }
                            }
                            div { class: "detail-actions",
                                button {
                                    class: "detail-delete-btn",
                                    onclick: move |_| on_delete.call(id),
                                    "Delete"
                                }
                            }
                        }
                    },
                    detail_state @ (DetailState::Error(_)
                    | DetailState::EmptyHistory
                    | DetailState::EmptySearch(_)
                    | DetailState::Unselected) => {
                        let (kicker, title, body, is_error) = match detail_state {
                            DetailState::Error(message) => (
                                "Load issue",
                                "Clipboard history couldn't be loaded",
                                message,
                                true,
                            ),
                            DetailState::EmptyHistory => (
                                "Waiting",
                                "No clipboard history yet",
                                "Copy some text or an image and it will show up here automatically.".to_string(),
                                false,
                            ),
                            DetailState::EmptySearch(query) => (
                                "No matches",
                                "Nothing matched your search",
                                format!(
                                    "No history entries matched \"{query}\". Try a shorter term or a different keyword."
                                ),
                                false,
                            ),
                            DetailState::Unselected => (
                                "Ready",
                                "Select an entry to inspect it",
                                "Choose an item from the left to view its contents, copy it again, or delete it.".to_string(),
                                false,
                            ),
                            _ => unreachable!(),
                        };
                        let class = if is_error {
                            "detail detail-empty detail-message-card detail-error-card"
                        } else {
                            "detail detail-empty detail-message-card"
                        };

                        rsx! {
                            div { class: class,
                                span { class: "detail-empty-kicker", "{kicker}" }
                                h2 { class: "detail-empty-title", "{title}" }
                                p { class: "detail-empty-body", "{body}" }
                            }
                        }
                    },
                }
            }
        }
    }
}
