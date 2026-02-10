use js_sys::Date;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum PasteEntry {
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
pub struct ClipboardPayload {
    pub entries: Vec<PasteEntry>,
    pub error: Option<String>,
}

pub fn preview_text(content: &str, limit: usize) -> String {
    let line = content.lines().next().unwrap_or("");
    let mut preview: String = line.chars().take(limit).collect();
    if line.chars().count() > limit {
        preview.push('…');
    }
    preview
}

pub fn entry_label(entry: &PasteEntry) -> &'static str {
    match entry {
        PasteEntry::Text { content, .. } => {
            if is_password_like(content) {
                "Password"
            } else {
                "Text"
            }
        }
        PasteEntry::Image { .. } => "Image",
    }
}

pub fn is_password_like(content: &str) -> bool {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return false;
    }
    if trimmed.chars().any(|ch| ch.is_whitespace()) {
        return false;
    }

    let mut has_upper = false;
    let mut has_lower = false;
    let mut has_digit = false;
    let mut has_punct = false;
    let mut unique = HashSet::new();
    let mut len = 0usize;

    for ch in trimmed.chars() {
        len += 1;
        unique.insert(ch);
        if ch.is_ascii_uppercase() {
            has_upper = true;
        } else if ch.is_ascii_lowercase() {
            has_lower = true;
        } else if ch.is_ascii_digit() {
            has_digit = true;
        } else if ch.is_ascii_punctuation() {
            has_punct = true;
        } else {
            // Non-ASCII likely means it's not a generated password.
            return false;
        }
    }

    if len < 8 {
        return false;
    }

    let classes = [has_upper, has_lower, has_digit, has_punct]
        .iter()
        .filter(|&&v| v)
        .count();
    let unique_ratio = unique.len() as f32 / len as f32;

    classes >= 3 && unique_ratio >= 0.6
}

pub fn mask_text(content: &str) -> String {
    content.chars().map(|_| '•').collect()
}

pub fn entry_key(entry: &PasteEntry) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    match entry {
        PasteEntry::Text { timestamp, content } => {
            timestamp.hash(&mut hasher);
            content.hash(&mut hasher);
        }
        PasteEntry::Image {
            timestamp,
            path,
            hash,
            ..
        } => {
            timestamp.hash(&mut hasher);
            path.hash(&mut hasher);
            hash.hash(&mut hasher);
        }
    }
    hasher.finish()
}

pub fn format_timestamp(timestamp: u64) -> String {
    let ms = (timestamp as f64) * 1000.0;
    let date = Date::new(&JsValue::from_f64(ms));
    let options = js_sys::Object::new();
    let parts = [
        ("weekday", "long"),
        ("year", "numeric"),
        ("month", "short"),
        ("day", "2-digit"),
        ("hour", "2-digit"),
        ("minute", "2-digit"),
    ];

    for (key, value) in parts {
        let _ = js_sys::Reflect::set(
            &options,
            &JsValue::from_str(key),
            &JsValue::from_str(value),
        );
    }

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
