use dioxus::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::app_actions::{open_url, set_status};
use crate::auth::{authenticate_admin_action, AuthCache};
use crate::entry::ClipboardEntry;
use crate::format::{
    entry_label, extract_single_url, format_timestamp, has_urls, image_data_uri_summary,
    is_image_data_uri, is_password, mask_password, preview_text, split_text_with_urls,
    TextSegment,
};
use crate::link_preview::LinkPreviewState;
use crate::watcher_control::WatcherStatus;

#[component]
pub fn LinkableText(text: String) -> Element {
    rsx! {
        span { class: "linkable-text",
            for (idx , segment) in split_text_with_urls(&text).into_iter().enumerate() {
                match segment {
                    TextSegment::Plain(t) => rsx! {
                        span { key: "{idx}", "{t}" }
                    },
                    TextSegment::Url(url) => {
                        let url_clone = url.clone();
                        rsx! {
                            a {
                                key: "{idx}",
                                class: "text-link",
                                onclick: move |_| open_url(&url_clone),
                                href: "#",
                                "{url}"
                            }
                        }
                    }
                }
            }
        }
    }
}

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
    watcher_status: WatcherStatus,
    on_select: EventHandler<i64>,
    on_query_input: EventHandler<String>,
    on_clear: EventHandler<()>,
    on_toggle_watcher: EventHandler<()>,
) -> Element {
    let watcher_state_class = if watcher_status.available {
        if watcher_status.paused {
            "watcher-pill paused"
        } else {
            "watcher-pill running"
        }
    } else {
        "watcher-pill offline"
    };

    let watcher_button_class = if watcher_status.available {
        if watcher_status.paused {
            "watcher-toggle-btn resume"
        } else {
            "watcher-toggle-btn pause"
        }
    } else {
        "watcher-toggle-btn disabled"
    };

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
            div { class: "watcher-card",
                div { class: "watcher-card-top",
                    div {
                        span { class: "watcher-eyebrow", "Watcher" }
                        div { class: "watcher-title-row",
                            span { class: "{watcher_state_class}", "{watcher_status.label}" }
                        }
                    }
                    button {
                        class: "{watcher_button_class}",
                        disabled: !watcher_status.available,
                        onclick: move |_| on_toggle_watcher.call(()),
                        if watcher_status.available {
                            if watcher_status.paused {
                                "Resume"
                            } else {
                                "Pause"
                            }
                        } else {
                            "Unavailable"
                        }
                    }
                }
                div { class: "watcher-detail", "{watcher_status.detail}" }
                div { class: "watcher-subtle-row",
                    if let Some(last_capture_at) = watcher_status.last_capture_at {
                        span { class: "watcher-subtle-label", "Last capture" }
                        span { class: "watcher-subtle-value", "{format_timestamp(last_capture_at)}" }
                    } else if watcher_status.available {
                        span { class: "watcher-subtle-label", "Last capture" }
                        span { class: "watcher-subtle-value", "No captures yet" }
                    } else {
                        span { class: "watcher-subtle-value", "Waiting for watcher status" }
                    }
                }
                if let Some(last_error) = watcher_status.last_error.as_ref() {
                    if !last_error.is_empty() {
                        div { class: "watcher-warning", "Last error: {last_error}" }
                    }
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
    show_password: Signal<bool>,
    auth_cache: Signal<Arc<Mutex<AuthCache>>>,
    action_status: Signal<Option<String>>,
    link_previews: Signal<HashMap<String, LinkPreviewState>>,
    on_copy_text: EventHandler<String>,
    on_delete: EventHandler<i64>,
    on_open_image: EventHandler<()>,
) -> Element {
    rsx! {
        section { class: "content",
            {
                match state {
                    DetailState::Entry(ClipboardEntry::Text { id, timestamp, content, .. }) => {
                        let text = content.clone();
                        let exact_url = extract_single_url(&content).map(str::to_string);
                        let preview_state = exact_url
                            .as_ref()
                            .and_then(|url| link_previews().get(url).cloned());
                        let is_data_uri = is_image_data_uri(&content);
                        let is_password_text = is_password(&content);
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
                                    span { class: "detail-type",
                                        if is_password_text {
                                            "Password"
                                        } else {
                                            "Text"
                                        }
                                    }
                                    span { class: "detail-ts", "Timestamp: {format_timestamp(timestamp)}" }
                                }
                                if let Some(LinkPreviewState::Ready(preview)) = preview_state {
                                    {
                                        let open_target = preview.url.clone();
                                        rsx! {
                                            button {
                                                class: "link-preview-card",
                                                onclick: move |_| open_url(&open_target),
                                                div { class: "link-preview-copy",
                                                    div { class: "link-preview-site",
                                                        if let Some(site_name) = preview.site_name.as_ref() {
                                                            span { class: "link-preview-site-name", "{site_name}" }
                                                        }
                                                        span { class: "link-preview-display-url", "{preview.display_url}" }
                                                    }
                                                    div { class: "link-preview-title", "{preview.title}" }
                                                    if let Some(description) = preview.description.as_ref() {
                                                        div { class: "link-preview-description", "{description}" }
                                                    }
                                                }
                                                if let Some(image_url) = preview.image_url.as_ref() {
                                                    img {
                                                        class: "link-preview-image",
                                                        src: "{image_url}",
                                                        alt: "Link preview image"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                if let Some(summary) = summary {
                                    div { class: "detail-note", "{summary}. Copy still uses the full value." }
                                }
                                if is_password_text {
                                    div { class: "detail-password-area",
                                        if show_password() {
                                            pre { class: "detail-text", "{content}" }
                                        } else {
                                            pre { class: "detail-text detail-password-masked", "{mask_password(&content)}" }
                                        }
                                    }
                                } else if is_data_uri {
                                    pre { class: "detail-text detail-text-truncated", "{display_text}" }
                                } else if has_urls(&content) {
                                    div { class: "detail-text",
                                        LinkableText { text: content.clone() }
                                    }
                                } else {
                                    pre { class: "detail-text", "{content}" }
                                }
                                div { class: "detail-actions",
                                    button {
                                        class: "detail-copy-btn",
                                        onclick: move |_| on_copy_text.call(text.clone()),
                                        "Copy"
                                    }
                                    if is_password_text {
                                        button {
                                            class: "detail-password-btn",
                                            onclick: move |_| {
                                                if show_password() {
                                                    show_password.set(false);
                                                } else {
                                                    if let Ok(mut cache_guard) = auth_cache().lock() {
                                                        if cache_guard.is_authenticated() {
                                                            show_password.set(true);
                                                        } else {
                                                            let auth_result = authenticate_admin_action();
                                                            if auth_result.success {
                                                                cache_guard.set_authenticated(true);
                                                                show_password.set(true);
                                                            } else {
                                                                set_status(action_status, "Authentication failed");
                                                            }
                                                        }
                                                    }
                                                }
                                            },
                                            if show_password() {
                                                "Hide"
                                            } else {
                                                "Show"
                                            }
                                        }
                                    }
                                    button { class: "detail-delete-btn", onclick: move |_| on_delete.call(id), "Delete" }
                                    if let Some(status) = copy_status.clone() {
                                        span { class: "detail-copy-status", "{status}" }
                                    }
                                }
                            }
                        }
                    }
                    DetailState::Entry(
                        ClipboardEntry::Image { id, timestamp, path, hash, data_url },
                    ) => rsx! {
                        div { class: "detail",
                            div { class: "detail-meta",
                                span { class: "detail-type", "Image" }
                                span { class: "detail-ts", "Timestamp: {format_timestamp(timestamp)}" }
                            }
                            div { class: "detail-image-wrap",
                                if let Some(src) = data_url {
                                    {
                                        rsx! {
                                            button {
                                                class: "detail-image-button",
                                                onclick: move |_| on_open_image.call(()),
                                                aria_label: "Open clipboard image in overlay",
                                                img { class: "detail-image", src, alt: "Clipboard image" }
                                            }
                                        }
                                    }
                                } else {
                                    div { class: "detail-image-missing", "Image data not available." }
                                }
                            }
                            div { class: "detail-image-hint", "Click image to open a larger view." }
                            div { class: "detail-footer",
                                if let Some(path) = path {
                                    span { "Export path: {path}" }
                                    span { "Hash is: {hash}" }
                                } else {
                                    span { "Hash is: {hash}" }
                                }
                            }
                            div { class: "detail-actions",
                                button { class: "detail-delete-btn", onclick: move |_| on_delete.call(id), "Delete" }
                            }
                        }
                    },
                    detail_state @ (DetailState::Error(_)
                    | DetailState::EmptyHistory
                    | DetailState::EmptySearch(_)
                    | DetailState::Unselected) => {
                        let (kicker, title, body, is_error) = match detail_state {
                            DetailState::Error(message) => {
                                (
                                    "Load issue",
                                    "Clipboard history could not be loaded",
                                    message,
                                    true,
                                )
                            }
                            DetailState::EmptyHistory => {
                                (
                                    "Waiting",
                                    "No clipboard history yet",
                                    "Copy some text or an image and it will show up here automatically."
                                        .to_string(),
                                    false,
                                )
                            }
                            DetailState::EmptySearch(query) => {
                                (
                                    "No matches",
                                    "Nothing matched your search",
                                    format!(
                                        "No history entries matched \"{query}\". Try a shorter term or a different keyword.",
                                    ),
                                    false,
                                )
                            }
                            DetailState::Unselected => {
                                (
                                    "Ready",
                                    "Select an entry to inspect it",
                                    "Choose an item from the left to view its contents, copy it again, or delete it."
                                        .to_string(),
                                    false,
                                )
                            }
                            _ => unreachable!(),
                        };
                        let class = if is_error {
                            "detail detail-empty detail-message-card detail-error-card"
                        } else {
                            "detail detail-empty detail-message-card"
                        };
                        rsx! {
                            div { class,
                                span { class: "detail-empty-kicker", "{kicker}" }
                                h2 { class: "detail-empty-title", "{title}" }
                                p { class: "detail-empty-body", "{body}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
