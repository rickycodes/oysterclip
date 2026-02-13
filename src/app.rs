use gloo_timers::callback::Interval;
use js_sys::Promise;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::common::{
    entry_label, format_timestamp, preview_text, ClipboardPayload, PasteEntry,
};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;

    #[wasm_bindgen(js_namespace = ["navigator", "clipboard"], js_name = writeText)]
    fn write_clipboard_text(text: &str) -> Promise;
}

#[function_component(App)]
pub fn app() -> Html {
    let entries = use_state(|| Vec::<PasteEntry>::new());
    let selected = use_state(|| Option::<usize>::None);
    let error = use_state(|| Option::<String>::None);
    let copy_status = use_state(|| Option::<String>::None);
    let in_flight = use_mut_ref(|| false);

    {
        let entries = entries.clone();
        let selected = selected.clone();
        let error = error.clone();
        let in_flight = in_flight.clone();
        use_effect_with((), move |_| {
            let interval = Interval::new(500, move || {
                if *in_flight.borrow() {
                    return;
                }
                *in_flight.borrow_mut() = true;

                let entries = entries.clone();
                let selected = selected.clone();
                let error = error.clone();
                let in_flight = in_flight.clone();
                spawn_local(async move {
                    let value = invoke("get_clipboard_entries", JsValue::NULL).await;
                    match serde_wasm_bindgen::from_value::<ClipboardPayload>(value) {
                        Ok(payload) => {
                            if let Some(err) = payload.error {
                                if error.as_ref() != Some(&err) {
                                    error.set(Some(err));
                                }
                            } else {
                                if (*error).is_some() {
                                    error.set(None);
                                }

                                let new_entries = payload.entries;
                                let new_len = new_entries.len();
                                if entries.as_slice() != new_entries.as_slice() {
                                    entries.set(new_entries);
                                }

                                match (*selected).clone() {
                                    Some(idx) if idx < new_len => {}
                                    _ => {
                                        if new_len == 0 {
                                            selected.set(None);
                                        }
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            error.set(Some(err.to_string()));
                        }
                    }
                    *in_flight.borrow_mut() = false;
                });
            });

            || drop(interval)
        });
    }

    let on_select = {
        let selected = selected.clone();
        let copy_status = copy_status.clone();
        Callback::from(move |idx: usize| {
            selected.set(Some(idx));
            if (*copy_status).is_some() {
                copy_status.set(None);
            }
        })
    };

    let detail = match (*selected).and_then(|idx| (*entries).get(idx)) {
        Some(PasteEntry::Text { timestamp, content }) => {
            let copy_status = copy_status.clone();
            let copy_status_for_click = copy_status.clone();
            let text = content.clone();
            let on_copy = Callback::from(move |_| {
                let copy_status = copy_status_for_click.clone();
                let text = text.clone();
                spawn_local(async move {
                    let result =
                        wasm_bindgen_futures::JsFuture::from(write_clipboard_text(&text)).await;
                    match result {
                        Ok(_) => copy_status.set(Some("Copied".to_string())),
                        Err(_) => copy_status.set(Some("Copy failed".to_string())),
                    }
                });
            });

            html! {
            <div class="detail">
                <div class="detail-meta">
                    <span class="detail-type">{ "Text" }</span>
                    <span class="detail-ts">{ format!("Timestamp: {}", format_timestamp(*timestamp)) }</span>
                </div>
                <pre class="detail-text">{ content.clone() }</pre>
                <div class="detail-actions">
                    <button class="detail-copy-btn" onclick={on_copy}>{ "Copy" }</button>
                    if let Some(status) = (*copy_status).clone() {
                        <span class="detail-copy-status">{ status }</span>
                    }
                </div>
            </div>
            }
        }
        Some(PasteEntry::Image { timestamp, path, hash, data_url }) => html! {
            <div class="detail">
                <div class="detail-meta">
                    <span class="detail-type">{ "Image" }</span>
                    <span class="detail-ts">{ format!("Timestamp: {}", format_timestamp(*timestamp)) }</span>
                </div>
                <div class="detail-image-wrap">
                    {
                        if let Some(src) = data_url.clone() {
                            html! { <img class="detail-image" src={src} alt="Clipboard image" /> }
                        } else {
                            html! { <div class="detail-image-missing">{ "Image data not available." }</div> }
                        }
                    }
                </div>
                <div class="detail-footer">
                    <span>{ format!("Path: {}", path) }</span>
                    <span>{ format!("Hash: {}", hash) }</span>
                </div>
            </div>
        },
        None => html! {
            <div class="detail detail-empty">
            </div>
        },
    };

    html! {
        <main class="app">
            <aside class="sidebar">
                <div class="sidebar-header">
                    <h1>{ "Clipboard" }</h1>
                    <span class="sidebar-count">{ format!("{} entries", entries.len()) }</span>
                </div>
                if let Some(err) = (*error).clone() {
                    <div class="sidebar-error">{ err }</div>
                }
                <div class="entry-list">
                    {
                        entries.iter().enumerate().map(|(idx, entry)| {
                            let is_active = Some(idx) == *selected;
                            let class = if is_active { "entry-card active" } else { "entry-card" };
                            let on_select = on_select.clone();
                            let preview = match entry {
                                PasteEntry::Text { content, .. } => preview_text(content, 56),
                                PasteEntry::Image { path, .. } => preview_text(path, 56),
                            };
                            let timestamp = match entry {
                                PasteEntry::Text { timestamp, .. } => *timestamp,
                                PasteEntry::Image { timestamp, .. } => *timestamp,
                            };
                            html! {
                                <button class={class} onclick={Callback::from(move |_| on_select.emit(idx))}>
                                    <div class="entry-title">{ entry_label(entry) }</div>
                                    <div class="entry-preview">{ preview }</div>
                                    <div class="entry-ts">{ format!("{}", format_timestamp(timestamp)) }</div>
                                </button>
                            }
                        }).collect::<Html>()
                    }
                </div>
            </aside>
            <section class="content">
                { detail }
            </section>
        </main>
    }
}
