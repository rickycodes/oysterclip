use dioxus::prelude::*;
use crate::theme::Theme;

#[component]
pub fn HelpModal(is_open: bool, on_close: EventHandler<()>, current_theme: Theme, on_theme_toggle: EventHandler<()>) -> Element {
    rsx! {
        if is_open {
            div {
                class: format!("help-overlay is-open {}", current_theme.class_name()),
                onclick: move |_| on_close.call(()),
                div {
                    class: "help-dialog",
                    onclick: move |event| event.stop_propagation(),
                    button {
                        class: "help-close",
                        onclick: move |_| on_close.call(()),
                        aria_label: "Close help",
                        "×"
                    }
                    h2 { class: "help-title", "Keyboard Shortcuts" }
                    div { class: "help-content",
                        div { class: "help-section",
                            h3 { "Navigation" }
                            div { class: "help-row",
                                code { "↑ / k" }
                                span { "Previous entry" }
                            }
                            div { class: "help-row",
                                code { "↓ / j" }
                                span { "Next entry" }
                            }
                            div { class: "help-row",
                                code { "Home" }
                                span { "First entry" }
                            }
                            div { class: "help-row",
                                code { "End" }
                                span { "Last entry" }
                            }
                        }
                        div { class: "help-section",
                            h3 { "Actions" }
                            div { class: "help-row",
                                code { "Enter / y" }
                                span { "Copy to clipboard" }
                            }
                            div { class: "help-row",
                                code { "Delete / Backspace / d" }
                                span { "Delete entry" }
                            }
                            div { class: "help-row",
                                code { "Escape" }
                                span { "Close overlay / clear search" }
                            }
                        }
                        div { class: "help-section",
                            h3 { "Search" }
                            div { class: "help-row",
                                code { "/ or Ctrl+F" }
                                span { "Focus search" }
                            }
                            div { class: "help-row",
                                code { "type:image" }
                                span { "Show only images" }
                            }
                            div { class: "help-row",
                                code { "type:password" }
                                span { "Show only passwords" }
                            }
                            div { class: "help-row",
                                code { "kind:url" }
                                span { "Show only URLs" }
                            }
                            div { class: "help-row",
                                code { "kind:json" }
                                span { "Show only JSON" }
                            }
                            div { class: "help-row",
                                span { class: "help-tip", "Combine filters with free-text search" }
                            }
                        }
                        div { class: "help-section",
                            h3 { "Theme" }
                            div { class: "help-row",
                                button {
                                    class: "theme-toggle-btn",
                                    onclick: move |_| on_theme_toggle.call(()),
                                    "Switch to {current_theme.toggle().label()} Mode"
                                }
                            }
                        }
                        div { class: "help-section",
                            h3 { "Other" }
                            div { class: "help-row",
                                code { "?" }
                                span { "Show this help" }
                            }
                        }
                    }
                }
            }
        }
    }
}
