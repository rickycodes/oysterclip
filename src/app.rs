use gloo_timers::callback::Interval;
use js_sys::Date;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type")]
enum PasteEntry {
    Text {
        #[serde(deserialize_with = "deserialize_timestamp")]
        timestamp: u64,
        content: String,
    },
    Image {
        #[serde(deserialize_with = "deserialize_timestamp")]
        timestamp: u64,
        path: String,
        #[serde(deserialize_with = "deserialize_u64")]
        hash: u64,
        data_url: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
struct ClipboardPayload {
    entries: Vec<PasteEntry>,
    error: Option<String>,
}

fn preview_text(content: &str, limit: usize) -> String {
    let line = content.lines().next().unwrap_or("");
    let mut preview: String = line.chars().take(limit).collect();
    if line.chars().count() > limit {
        preview.push('…');
    }
    preview
}

fn entry_label(entry: &PasteEntry) -> &'static str {
    match entry {
        PasteEntry::Text { .. } => "Text",
        PasteEntry::Image { .. } => "Image",
    }
}

fn format_timestamp(timestamp: u64) -> String {
    let ms = (timestamp as f64) * 1000.0;
    let date = Date::new(&JsValue::from_f64(ms));
    let options = js_sys::Object::new();
    let _ = js_sys::Reflect::set(
        &options,
        &JsValue::from_str("weekday"),
        &JsValue::from_str("long"),
    );
    let _ = js_sys::Reflect::set(
        &options,
        &JsValue::from_str("year"),
        &JsValue::from_str("numeric"),
    );
    let _ = js_sys::Reflect::set(
        &options,
        &JsValue::from_str("month"),
        &JsValue::from_str("short"),
    );
    let _ = js_sys::Reflect::set(
        &options,
        &JsValue::from_str("day"),
        &JsValue::from_str("2-digit"),
    );
    let _ = js_sys::Reflect::set(
        &options,
        &JsValue::from_str("hour"),
        &JsValue::from_str("2-digit"),
    );
    let _ = js_sys::Reflect::set(
        &options,
        &JsValue::from_str("minute"),
        &JsValue::from_str("2-digit"),
    );

    date.to_locale_string("en-US", &options.into()).into()
}

fn deserialize_timestamp<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserialize_u64(deserializer)
}

fn deserialize_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Number(num) => {
            if let Some(u) = num.as_u64() {
                Ok(u)
            } else if let Some(f) = num.as_f64() {
                Ok(f.round() as u64)
            } else {
                Err(D::Error::custom("invalid number for u64"))
            }
        }
        serde_json::Value::String(s) => s
            .parse::<u64>()
            .map_err(|_| D::Error::custom("invalid string for u64")),
        _ => Err(D::Error::custom("invalid type for u64")),
    }
}

#[function_component(App)]
pub fn app() -> Html {
    let entries = use_state(|| Vec::<PasteEntry>::new());
    let selected = use_state(|| Option::<usize>::None);
    let error = use_state(|| Option::<String>::None);

    {
        let entries = entries.clone();
        let selected = selected.clone();
        let error = error.clone();
        use_effect_with((), move |_| {
            let interval = Interval::new(500, move || {
                let entries = entries.clone();
                let selected = selected.clone();
                let error = error.clone();
                spawn_local(async move {
                    let value = invoke("get_clipboard_entries", JsValue::NULL).await;
                    match serde_wasm_bindgen::from_value::<ClipboardPayload>(value) {
                        Ok(payload) => {
                            if let Some(err) = payload.error {
                                error.set(Some(err));
                                return;
                            }

                            error.set(None);
                            let new_len = payload.entries.len();
                            entries.set(payload.entries);

                            match (*selected).clone() {
                                Some(idx) if idx < new_len => {}
                                _ => {
                                    if new_len == 0 {
                                        selected.set(None);
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            error.set(Some(err.to_string()));
                        }
                    }
                });
            });

            || drop(interval)
        });
    }

    let on_select = {
        let selected = selected.clone();
        Callback::from(move |idx: usize| {
            selected.set(Some(idx));
        })
    };

    let detail = match (*selected).and_then(|idx| (*entries).get(idx)) {
        Some(PasteEntry::Text { timestamp, content }) => html! {
            <div class="detail">
                <div class="detail-meta">
                    <span class="detail-type">{ "Text" }</span>
                    <span class="detail-ts">{ format!("Timestamp: {}", format_timestamp(*timestamp)) }</span>
                </div>
                <pre class="detail-text">{ content.clone() }</pre>
            </div>
        },
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
