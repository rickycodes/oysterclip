use dioxus::prelude::*;

use crate::app::actions::open_url;
use crate::data::format::{split_text_with_urls, TextSegment};

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
                                onclick: move |e: dioxus::prelude::MouseEvent| { e.prevent_default(); open_url(&url_clone); },
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
