use dioxus::prelude::*;

use crate::entry::ClipboardEntry;
use crate::format::{
    entry_label, format_timestamp, image_data_uri_summary, is_image_data_uri, preview_text,
};

#[component]
pub fn Sidebar(
    entries: Vec<ClipboardEntry>,
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
                    span { class: "sidebar-count", "{entries.len()} entries" }
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
                }
            }
            if let Some(err) = error {
                div { class: "sidebar-error", "{err}" }
            }
            if let Some(status) = action_status {
                div { class: "sidebar-status", "{status}" }
            }
            div { class: "entry-list",
                if entries.is_empty() {
                    div { class: "sidebar-empty",
                        if query.is_empty() {
                            "No history yet."
                        } else {
                            "No matching entries."
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
    detail: Option<ClipboardEntry>,
    copy_status: Option<String>,
    on_copy_text: EventHandler<String>,
    on_delete: EventHandler<i64>,
) -> Element {
    rsx! {
        section { class: "content",
            {
                match detail {
                    Some(ClipboardEntry::Text { id, timestamp, content, .. }) => {
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
                    }
                    Some(ClipboardEntry::Image {
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
                    None => rsx! {
                        div { class: "detail detail-empty" }
                    },
                }
            }
        }
    }
}
