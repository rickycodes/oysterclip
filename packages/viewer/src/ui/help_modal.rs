use crate::config::{modal, HELP_KEYBOARD_SECTIONS};
use crate::data::format::format_timestamp;
use crate::system::watcher_control::WatcherStatus;
use crate::ui::theme::Theme;
use dioxus::prelude::*;
use std::rc::Rc;

#[component]
pub fn HelpModal(
    is_open: bool,
    on_close: EventHandler<()>,
    current_theme: Theme,
    on_theme_toggle: EventHandler<()>,
    watcher_status: WatcherStatus,
    on_toggle_watcher: EventHandler<()>,
) -> Element {
    let mut overlay_ref: Signal<Option<Rc<dioxus::prelude::MountedData>>> = use_signal(|| None);

    use_effect(move || {
        if is_open {
            if let Some(el) = overlay_ref() {
                spawn(async move {
                    let _ = el.set_focus(true).await;
                });
            }
        }
    });

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
        if is_open {
            div {
                class: format!("help-overlay is-open {}", current_theme.class_name()),
                tabindex: 0,
                onmounted: move |e| overlay_ref.set(Some(e.data())),
                onclick: move |_| on_close.call(()),
                onkeydown: move |event: KeyboardEvent| {
                    let code = event.code();
                    match code {
                        Code::Escape => {
                            event.prevent_default();
                            on_close.call(());
                        }
                        Code::KeyP => {
                            event.prevent_default();
                            on_toggle_watcher.call(());
                        }
                        _ => {}
                    }
                },
                div {
                    class: "help-dialog",
                    onclick: move |event| event.stop_propagation(),
                    button {
                        class: "help-close",
                        onclick: move |_| on_close.call(()),
                        aria_label: modal::CLOSE_LABEL,
                        "×"
                    }

                    h2 { class: "help-title", "{modal::TITLE}" }
                    div { class: "help-content",
                        {
                            rsx! {
                                for section in HELP_KEYBOARD_SECTIONS {
                                    div { class: "help-section",
                                        h3 { "{section.title}" }
                                        for entry in section.entries {
                                            div { class: "help-row",
                                                code { "{entry.code}" }
                                                span { "{entry.description}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        div { class: "help-section",
                            span { class: "help-tip", "{modal::FILTER_TIP}" }
                        }
                    }

                    h2 { class: "help-title help-title-secondary", "Controls" }
                    div { class: "help-content",
                        div { class: "help-section",
                            h3 { "{modal::controls::SECTION_THEME}" }
                            div { class: "help-row",
                                button {
                                    class: "theme-toggle-btn",
                                    onclick: move |_| on_theme_toggle.call(()),
                                    "Switch to {current_theme.toggle().label()} Mode"
                                }
                            }
                        }
                        div { class: "help-section",
                            h3 { "{modal::controls::SECTION_WATCHER}" }
                            div { class: "help-row help-row-watcher",
                                span { class: "{watcher_state_class}", "{watcher_status.label}" }
                                button {
                                    class: "{watcher_button_class}",
                                    disabled: !watcher_status.available,
                                    onclick: move |_| on_toggle_watcher.call(()),
                                    if watcher_status.available {
                                        if watcher_status.paused { "{modal::controls::watcher::RESUME}" } else { "{modal::controls::watcher::PAUSE}" }
                                    } else {
                                        "{modal::controls::watcher::UNAVAILABLE}"
                                    }
                                }
                                code { class: "help-key-aside", "p" }
                            }
                            div { class: "watcher-detail", "{watcher_status.detail}" }
                            div { class: "watcher-subtle-row",
                                if let Some(last_capture_at) = watcher_status.last_capture_at {
                                    span { class: "watcher-subtle-label", "{modal::controls::watcher::LAST_CAPTURE}" }
                                    span { class: "watcher-subtle-value", "{format_timestamp(last_capture_at)}" }
                                } else if watcher_status.available {
                                    span { class: "watcher-subtle-label", "{modal::controls::watcher::LAST_CAPTURE}" }
                                    span { class: "watcher-subtle-value", "{modal::controls::watcher::NO_CAPTURES_YET}" }
                                } else {
                                    span { class: "watcher-subtle-value", "{modal::controls::watcher::WAITING_FOR_STATUS}" }
                                }
                            }
                            if let Some(last_error) = watcher_status.last_error.as_ref() {
                                if !last_error.is_empty() {
                                    div { class: "watcher-warning", "{modal::controls::watcher::LAST_ERROR_PREFIX}{last_error}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
