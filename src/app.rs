use arboard::Clipboard;
use dioxus::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::common::{
    entry_label, format_timestamp, get_clipboard_entries, preview_text, CachedEntries,
    ClipboardEntry, ClipboardSource,
};

const APP_STYLE: &str = include_str!("../styles.css");

#[component]
pub fn App() -> Element {
    let source = use_hook(|| Arc::new(ClipboardSource::from_env()));
    let cache = use_hook(|| Arc::new(Mutex::new(None::<CachedEntries>)));
    let mut entries = use_signal(Vec::<ClipboardEntry>::new);
    let mut selected = use_signal(|| None::<usize>);
    let mut error = use_signal(|| None::<String>);
    let copy_status = use_signal(|| None::<String>);

    use_future(move || {
        let source = source.clone();
        let cache = cache.clone();
        async move {
            loop {
                let payload = {
                    if let Ok(mut cache_guard) = cache.lock() {
                        get_clipboard_entries(&source, &mut cache_guard)
                    } else {
                        crate::common::ClipboardPayload {
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

    rsx! {
        style { "{APP_STYLE}" }
        main { class: "app",
            aside { class: "sidebar",
                div { class: "sidebar-header",
                    h1 { "Clipboard" }
                    span { class: "sidebar-count", "{current_entries.len()} entries" }
                }
                if let Some(err) = error() {
                    div { class: "sidebar-error", "{err}" }
                }
                div { class: "entry-list",
                    for (idx, entry) in current_entries.iter().enumerate() {
                        {
                            let is_active = Some(idx) == current_selected;
                            let class = if is_active { "entry-card active" } else { "entry-card" };
                            let preview = match entry {
                                ClipboardEntry::Text { content, .. } => preview_text(content, 56),
                                ClipboardEntry::Image { path, .. } => preview_text(path, 56),
                            };
                            let timestamp = match entry {
                                ClipboardEntry::Text { timestamp, .. } => *timestamp,
                                ClipboardEntry::Image { timestamp, .. } => *timestamp,
                            };
                            let mut selected = selected;
                            let mut copy_status = copy_status;
                            rsx! {
                                button {
                                    class: "{class}",
                                    onclick: move |_| {
                                        selected.set(Some(idx));
                                        if copy_status().is_some() {
                                            copy_status.set(None);
                                        }
                                    },
                                    div { class: "entry-title", "{entry_label(entry)}" }
                                    div { class: "entry-preview", "{preview}" }
                                    div { class: "entry-ts", "{format_timestamp(timestamp)}" }
                                }
                            }
                        }
                    }
                }
            }
            section { class: "content",
                {
                    match detail {
                        Some(ClipboardEntry::Text { timestamp, content }) => {
                            let mut copy_status = copy_status;
                            let text = content.clone();
                            rsx! {
                                div { class: "detail",
                                    div { class: "detail-meta",
                                        span { class: "detail-type", "Text" }
                                        span { class: "detail-ts", "Timestamp: {format_timestamp(timestamp)}" }
                                    }
                                    pre { class: "detail-text", "{content}" }
                                    div { class: "detail-actions",
                                        button {
                                            class: "detail-copy-btn",
                                            onclick: move |_| {
                                                let result = Clipboard::new()
                                                    .and_then(|mut cb| cb.set_text(text.clone()));
                                                match result {
                                                    Ok(_) => copy_status.set(Some("Copied".to_string())),
                                                    Err(_) => copy_status.set(Some("Copy failed".to_string())),
                                                }
                                            },
                                            "Copy"
                                        }
                                        if let Some(status) = copy_status() {
                                            span { class: "detail-copy-status", "{status}" }
                                        }
                                    }
                                }
                            }
                        }
                        Some(ClipboardEntry::Image {
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
}
