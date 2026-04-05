use dioxus::prelude::*;

use crate::data::format::format_timestamp;
use crate::ui::icon::Icon;

#[component]
pub fn ImageDetail(
    id: i64,
    timestamp: u64,
    path: Option<String>,
    hash: u64,
    data_url: Option<String>,
    on_open_image: EventHandler<()>,
    on_delete: EventHandler<i64>,
) -> Element {
    rsx! {
        div { class: "detail",
            div { class: "detail-meta",
                div { class: "detail-type-with-icon",
                    svg { class: "detail-icon",
                        view_box: "0 0 24 24",
                        Icon { name: "image" }
                    }
                    span { class: "detail-type", "Image" }
                }
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
                    {
                        rsx! {
                            span { "Export path: {path}" }
                            span { "Hash is: {hash}" }
                        }
                    }
                } else {
                    span { "Hash is: {hash}" }
                }
            }
            div { class: "detail-actions",
                button { class: "detail-delete-btn", onclick: move |_| on_delete.call(id), "Delete" }
            }
        }
    }
}
