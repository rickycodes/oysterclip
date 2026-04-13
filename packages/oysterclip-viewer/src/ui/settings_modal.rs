use dioxus::prelude::*;

use crate::app::root::use_config;
use common::classification::PASSWORD_LEN;

#[component]
pub fn SettingsModal(mut is_open: Signal<bool>) -> Element {
    let mut config_signal = use_config();
    let config = config_signal();

    let mut password_len_enabled = use_signal(|| config.password.len.is_some());
    let mut password_len_value = use_signal(|| config.password.len.unwrap_or(PASSWORD_LEN));
    let mut password_score_value = use_signal(|| config.password.score_threshold);

    let handle_save = move |_| {
        // Update config
        let mut updated_config = config_signal().clone();
        updated_config.password.len = if password_len_enabled() {
            Some(password_len_value())
        } else {
            None
        };
        updated_config.password.score_threshold = password_score_value();

        // Persist to file and update signal
        updated_config.save();
        config_signal.set(updated_config);

        // Close modal
        is_open.set(false);
    };

    let handle_cancel = move |_| {
        is_open.set(false);
    };

    let score_labels = ["Any", "Weak", "Fair", "Good", "Strong"];

    if is_open() {
        rsx! {
            div { class: format!("modal-overlay {}", if is_open() { "is-open" } else { "" }),
                div { class: "modal",
                    div { class: "modal-header",
                        h2 { "Settings" }
                        button {
                            class: "modal-close",
                            onclick: move |_| is_open.set(false),
                            "×"
                        }
                    }
                    div { class: "modal-body",
                        div { class: "settings-section",
                            div { class: "settings-label-with-badge",
                                label { "Minimum Password Length" }
                                if password_len_enabled() {
                                    span { class: "settings-badge", "{password_len_value()}" }
                                } else {
                                    span { class: "settings-badge disabled", "Disabled" }
                                }
                            }
                            div { class: "settings-toggle",
                                label {
                                    input {
                                        r#type: "checkbox",
                                        checked: password_len_enabled(),
                                        onchange: move |evt| password_len_enabled.set(evt.checked()),
                                    }
                                    span { "Enabled" }
                                }
                            }
                            input {
                                r#type: "range",
                                min: "5",
                                max: "200",
                                value: "{password_len_value()}",
                                oninput: move |evt| password_len_value.set(evt.value().parse().unwrap_or(PASSWORD_LEN)),
                                class: "settings-slider",
                                disabled: !password_len_enabled(),
                            }
                        }
                        div { class: "settings-section",
                            div { class: "settings-label-with-badge",
                                label { "Minimum Password Strength" }
                                span { class: "settings-badge", "{score_labels[password_score_value() as usize]}" }
                            }
                            input {
                                r#type: "range",
                                min: "0",
                                max: "4",
                                value: "{password_score_value()}",
                                oninput: move |evt| password_score_value.set(evt.value().parse().unwrap_or(3)),
                                class: "settings-slider",
                            }
                            div { class: "score-legend",
                                span { "0: Any" }
                                span { "1: Weak" }
                                span { "2: Fair" }
                                span { "3: Good" }
                                span { "4: Strong" }
                            }
                        }
                    }
                    div { class: "modal-footer",
                        button {
                            class: "btn-secondary",
                            onclick: handle_cancel,
                            "Cancel"
                        }
                        button {
                            class: "btn-primary",
                            onclick: handle_save,
                            "Save"
                        }
                    }
                }
            }
        }
    } else {
        rsx! {}
    }
}
