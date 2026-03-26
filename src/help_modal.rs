use dioxus::prelude::*;
use crate::theme::Theme;
use crate::watcher_control::WatcherStatus;
use crate::format::format_timestamp;

#[component]
pub fn HelpModal(
    is_open: bool,
    on_close: EventHandler<()>,
    current_theme: Theme,
    on_theme_toggle: EventHandler<()>,
    watcher_status: WatcherStatus,
    on_toggle_watcher: EventHandler<()>,
) -> Element {
    let watcher_state_class = if watcher_status.available {
        if watcher_status.paused { "watcher-pill paused" } else { "watcher-pill running" }
    } else {
        "watcher-pill offline"
    };
    let watcher_button_class = if watcher_status.available {
        if watcher_status.paused { "watcher-toggle-btn resume" } else { "watcher-toggle-btn pause" }
    } else {
        "watcher-toggle-btn disabled"
    };

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
                            h3 { "Selection" }
                            div { class: "help-row",
                                code { "Space" }
                                span { "Toggle selection" }
                            }
                            div { class: "help-row",
                                code { "Shift+↑ / Shift+↓" }
                                span { "Extend selection" }
                            }
                            div { class: "help-row",
                                code { "Escape" }
                                span { "Clear selection" }
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
                                span { "Delete entry (or selection)" }
                            }
                            div { class: "help-row",
                                code { "Escape" }
                                span { "Close overlay / clear selection / clear search" }
                            }
                            div { class: "help-row",
                                code { "p" }
                                span { "Pause / resume watcher" }
                            }
                            div { class: "help-row",
                                code { "?" }
                                span { "Show this help" }
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
                                code { "since:1h" }
                                span { "Last hour (also: 24h, 7d, 30d, today, yesterday)" }
                            }
                            div { class: "help-row",
                                span { class: "help-tip", "Combine filters with free-text search" }
                            }
                        }
                    }

                    h2 { class: "help-title help-title-secondary", "Controls" }
                    div { class: "help-content",
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
                            h3 { "Watcher" }
                            div { class: "help-row help-row-watcher",
                                span { class: "{watcher_state_class}", "{watcher_status.label}" }
                                button {
                                    class: "{watcher_button_class}",
                                    disabled: !watcher_status.available,
                                    onclick: move |_| on_toggle_watcher.call(()),
                                    if watcher_status.available {
                                        if watcher_status.paused { "Resume" } else { "Pause" }
                                    } else {
                                        "Unavailable"
                                    }
                                }
                                code { class: "help-key-aside", "p" }
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
                    }
                }
            }
        }
    }
}
