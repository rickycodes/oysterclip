use dioxus::prelude::*;

use crate::app::root::use_config;

#[component]
pub fn SettingsModal(mut is_open: Signal<bool>) -> Element {
    let mut config_signal = use_config();
    let config = config_signal();
    
    let mut password_len_input = use_signal(|| config.password.len.to_string());
    let mut password_score_input = use_signal(|| config.password.score_threshold.to_string());
    let mut error_message = use_signal(String::new);

    let handle_save = move |_| {
        error_message.set(String::new());

        // Parse and validate inputs
        let new_len = match password_len_input().parse::<usize>() {
            Ok(n) if n > 0 => n,
            _ => {
                error_message.set("Password length must be a positive number".to_string());
                return;
            }
        };

        let new_score = match password_score_input().parse::<u8>() {
            Ok(n) if n <= 4 => n,
            _ => {
                error_message.set("Password score must be 0-4".to_string());
                return;
            }
        };

        // Update config
        let mut updated_config = config_signal().clone();
        updated_config.password.len = new_len;
        updated_config.password.score_threshold = new_score;

        // Persist to file and update signal
        updated_config.save();
        config_signal.set(updated_config);

        // Close modal
        is_open.set(false);
    };

    let handle_cancel = move |_| {
        error_message.set(String::new());
        is_open.set(false);
    };

    if is_open() {
        rsx! {
            div { class: "modal-overlay",
                div { class: "modal",
                    div { class: "modal-header",
                        h2 { "Password Detection Settings" }
                        button {
                            class: "modal-close",
                            onclick: move |_| is_open.set(false),
                            "×"
                        }
                    }
                    div { class: "modal-body",
                        div { class: "settings-group",
                            label { "Minimum Password Length" }
                            input {
                                r#type: "number",
                                min: "1",
                                max: "1000",
                                value: "{password_len_input}",
                                oninput: move |evt| password_len_input.set(evt.value()),
                            }
                            p { class: "help-text",
                                "Minimum number of characters for clipboard content to be detected as a password"
                            }
                        }
                        div { class: "settings-group",
                            label { "Minimum Password Strength" }
                            div { class: "score-input-container",
                                input {
                                    r#type: "number",
                                    min: "0",
                                    max: "4",
                                    value: "{password_score_input}",
                                    oninput: move |evt| password_score_input.set(evt.value()),
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
                        if !error_message().is_empty() {
                            div { class: "error-message", "{error_message}" }
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
