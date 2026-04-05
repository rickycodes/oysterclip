use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub enum EmptyStateKind {
    Error(String),
    EmptyHistory,
    EmptySearch(String),
    Unselected,
}

#[component]
pub fn EmptyDetail(kind: EmptyStateKind) -> Element {
    let (kicker, title, body, is_error) = match kind {
        EmptyStateKind::Error(message) => (
            "Load issue",
            "Clipboard history could not be loaded",
            message,
            true,
        ),
        EmptyStateKind::EmptyHistory => (
            "Waiting",
            "No clipboard history yet",
            "Copy some text or an image and it will show up here automatically.".to_string(),
            false,
        ),
        EmptyStateKind::EmptySearch(query) => (
            "No matches",
            "Nothing matched your search",
            format!(
                "No history entries matched \"{query}\". Try a shorter term or a different keyword.",
            ),
            false,
        ),
        EmptyStateKind::Unselected => (
            "Ready",
            "Select an entry to inspect it",
            "Choose an item from the left to view its contents, copy it again, or delete it."
                .to_string(),
            false,
        ),
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
