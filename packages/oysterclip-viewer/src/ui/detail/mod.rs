pub mod empty;
pub mod image;
pub mod text;
pub mod text_type;

use dioxus::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use self::empty::{EmptyDetail, EmptyStateKind};
use self::image::ImageDetail;
use self::text::TextDetail;
use crate::data::entry::ClipboardEntry;
use crate::data::link_preview::LinkPreviewState;
use common::AuthCache;

#[derive(Clone, PartialEq)]
pub enum DetailState {
    Error(String),
    EmptyHistory,
    EmptySearch(String),
    Unselected,
    Entry(ClipboardEntry),
}

#[component]
pub fn DetailPane(
    state: DetailState,
    copy_status: Option<String>,
    show_password: Signal<bool>,
    auth_cache: Signal<Arc<Mutex<AuthCache>>>,
    action_status: Signal<Option<String>>,
    link_previews: Signal<HashMap<String, LinkPreviewState>>,
    on_copy_text: EventHandler<(i64, String, &'static str)>,
    on_delete: EventHandler<i64>,
    on_open_image: EventHandler<()>,
    on_open_editor: EventHandler<i64>,
) -> Element {
    rsx! {
        section { class: "content",
            {
                match state {
                    DetailState::Entry(ClipboardEntry::Text { id, timestamp, content, kind, .. }) => {
                        rsx! {
                            TextDetail {
                                id,
                                timestamp,
                                content,
                                kind,
                                copy_status,
                                show_password,
                                auth_cache,
                                action_status,
                                link_previews,
                                on_copy_text,
                                on_delete,
                                on_open_editor,
                            }
                        }
                    }
                    DetailState::Entry(
                        ClipboardEntry::Image { id, timestamp, path, hash, data_url },
                    ) => rsx! {
                        ImageDetail {
                            id,
                            timestamp,
                            path,
                            hash,
                            data_url,
                            on_open_image,
                            on_delete,
                        }
                    },
                    DetailState::Error(message) => rsx! {
                        EmptyDetail { kind: EmptyStateKind::Error(message) }
                    },
                    DetailState::EmptyHistory => rsx! {
                        EmptyDetail { kind: EmptyStateKind::EmptyHistory }
                    },
                    DetailState::EmptySearch(query) => rsx! {
                        EmptyDetail { kind: EmptyStateKind::EmptySearch(query) }
                    },
                    DetailState::Unselected => rsx! {
                        EmptyDetail { kind: EmptyStateKind::Unselected }
                    },
                }
            }
        }
    }
}
