use dioxus::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::text_type::TextDetailType;
use crate::app::actions::open_url;
use crate::data::entry::ClipboardEntry;
use crate::data::format::{
    entry_icon_name, entry_label, extract_single_url, format_timestamp, has_urls, is_html_img_tag,
    is_image_data_uri, is_password, mask_password_preview,
};
use crate::data::link_preview::LinkPreviewState;
use crate::ui::icon::Icon;
use crate::ui::linkable_text::LinkableText;
use common::{authenticate_admin_action, AuthCache};
use common::{TEXT_KIND_JSON, TEXT_KIND_PATH};

#[component]
pub fn TextDetail(
    id: i64,
    timestamp: u64,
    content: String,
    kind: Option<String>,
    copy_status: Option<String>,
    show_password: Signal<bool>,
    auth_cache: Signal<Arc<Mutex<AuthCache>>>,
    action_status: Signal<Option<String>>,
    link_previews: Signal<HashMap<String, LinkPreviewState>>,
    on_copy_text: EventHandler<(i64, String, &'static str)>,
    on_delete: EventHandler<i64>,
    on_open_editor: EventHandler<i64>,
) -> Element {
    let text = content.clone();
    let type_label = entry_label(&ClipboardEntry::Text {
        id,
        timestamp,
        content: content.clone(),
        kind: kind.clone(),
    });
    let exact_url = extract_single_url(&content).map(str::to_string);
    let preview_state = exact_url
        .as_ref()
        .and_then(|url| link_previews().get(url).cloned());
    let is_data_uri = is_image_data_uri(&content);
    let is_password_text = is_password(&content);
    let is_json = kind.as_deref() == Some(TEXT_KIND_JSON);
    let is_path = kind.as_deref() == Some(TEXT_KIND_PATH);
    let is_html_image = is_html_img_tag(&content);

    let detail_type = TextDetailType::classify(
        &content,
        kind.as_deref(),
        exact_url.is_some(),
        is_html_image,
    );
    let detail_label = detail_type.label();
    let display_data = detail_type.extract_display_data(&content);

    rsx! {
        div { class: "detail",
            div { class: "detail-meta",
                div { class: "detail-type-with-icon",
                    svg { class: "detail-icon",
                        view_box: "0 0 24 24",
                        Icon { name: entry_icon_name(&ClipboardEntry::Text { id, timestamp, content: content.clone(), kind: kind.clone() }) }
                    }
                    span { class: "detail-type", "{detail_label}" }
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
            if let Some(summary) = &display_data.summary {
                div { class: "detail-note", "{summary}. Copy still uses the full value." }
            }
            if is_password_text {
                div { class: "detail-password-area",
                    if show_password() {
                        pre { class: "detail-text detail-password", "{content}" }
                    } else {
                        pre { class: "detail-text detail-password detail-password-masked", "{mask_password_preview()}" }
                    }
                }
            } else if is_data_uri {
                pre { class: "detail-text detail-text-truncated detail-data-uri", "{display_data.display_text}" }
            } else if is_json {
                pre { class: "detail-text detail-json",
                    "{display_data.pretty_json.as_deref().unwrap_or(&content)}"
                }
            } else if is_path {
                pre { class: "detail-text detail-path",
                    {
                        let path_to_open = content.clone();
                        rsx! {
                            a {
                                class: "text-link",
                                href: "#",
                                onclick: move |e: dioxus::prelude::MouseEvent| { e.prevent_default(); open_url(&path_to_open); },
                                "{content}"
                            }
                        }
                    }
                }
            } else if let Some(image_src) = &display_data.html_image_src {
                div { class: "detail-image-wrap",
                    img { class: "detail-image", src: "{image_src}", alt: "Extracted HTML image" }
                }
                div { class: "detail-image-hint", "HTML image extracted from clipboard." }
            } else if has_urls(&content) {
                div { class: "detail-text detail-url",
                    LinkableText { text: content.clone() }
                }
            } else {
                pre { class: "detail-text", "{content}" }
            }
            div { class: "detail-actions",
                button {
                    class: "detail-copy-btn",
                    onclick: move |_| on_copy_text.call((id, text.clone(), type_label)),
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
                                            action_status.set(Some("Authentication failed".to_string()));
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
                button {
                    class: "detail-notepad-btn",
                    disabled: is_password_text && !show_password(),
                    onclick: move |_| on_open_editor.call(id),
                    "Notepad"
                }
                button { class: "detail-delete-btn", onclick: move |_| on_delete.call(id), "Delete" }
                if let Some(status) = copy_status.clone() {
                    span { class: "detail-copy-status", "{status}" }
                }
            }
        }
    }
}
