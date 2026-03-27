use chrono::{DateTime, Local, Utc};

use crate::data::entry::ClipboardEntry;

pub fn format_relative_timestamp(timestamp: u64) -> String {
    let now = Utc::now().timestamp() as u64;
    let secs = now.saturating_sub(timestamp);

    if secs < 60 {
        return "just now".to_string();
    }
    let mins = secs / 60;
    if mins < 60 {
        return format!("{}m ago", mins);
    }
    let hours = mins / 60;
    if hours < 24 {
        return format!("{}h ago", hours);
    }
    // For older entries, show a short date
    if let Some(utc) = DateTime::<Utc>::from_timestamp(timestamp as i64, 0) {
        let local = utc.with_timezone(&Local);
        let today = Local::now().date_naive();
        let entry_date = local.date_naive();
        let days_ago = (today - entry_date).num_days();
        if days_ago == 1 {
            return "yesterday".to_string();
        }
        if days_ago < 7 {
            return local.format("%A").to_string(); // e.g. "Monday"
        }
        return local.format("%b %-d").to_string(); // e.g. "Mar 20"
    }
    timestamp.to_string()
}

pub fn preview_text(content: &str, limit: usize) -> String {
    if is_password(content) {
        mask_password_preview()
    } else {
        let line = content.lines().next().unwrap_or("");
        let mut preview: String = line.chars().take(limit).collect();
        if line.chars().count() > limit {
            preview.push('…');
        }
        preview
    }
}

pub fn is_image_data_uri(content: &str) -> bool {
    let trimmed = content.trim();
    trimmed.starts_with("data:image/") && trimmed.contains(";base64,")
}

pub fn image_data_uri_summary(content: &str) -> String {
    let trimmed = content.trim();
    let media_type = trimmed
        .strip_prefix("data:")
        .and_then(|value| value.split(';').next())
        .unwrap_or("image data");
    format!(
        "{} hidden for readability ({} chars)",
        media_type,
        trimmed.chars().count()
    )
}

pub fn entry_label(entry: &ClipboardEntry) -> &'static str {
    match entry {
        ClipboardEntry::Text { content, kind, .. } => {
            if is_password(content) {
                "Pass"
            } else if extract_single_url(content).is_some() {
                "Link"
            } else if kind.as_deref() == Some("json") {
                "JSON"
            } else if kind.as_deref() == Some("path") {
                "Path"
            } else {
                "Text"
            }
        }
        ClipboardEntry::Image { .. } => "Image",
    }
}

pub fn entry_icon_name(entry: &ClipboardEntry) -> &'static str {
    match entry {
        ClipboardEntry::Text { content, kind, .. } => {
            if is_password(content) {
                "lock"
            } else if extract_single_url(content).is_some() {
                "link"
            } else if kind.as_deref() == Some("json") {
                "braces"
            } else if kind.as_deref() == Some("path") {
                "folder"
            } else {
                "file-text"
            }
        }
        ClipboardEntry::Image { .. } => "image",
    }
}

pub fn format_timestamp(timestamp: u64) -> String {
    if let Some(utc) = DateTime::<Utc>::from_timestamp(timestamp as i64, 0) {
        utc.with_timezone(&Local)
            .format("%A, %b %d, %Y %I:%M %p")
            .to_string()
    } else {
        timestamp.to_string()
    }
}

use regex::Regex;
use std::sync::LazyLock;

static URL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(https?://[^\s/$.?#].[^\s]*)").unwrap());

pub fn extract_urls(text: &str) -> Vec<(usize, usize)> {
    URL_REGEX
        .find_iter(text)
        .map(|m| (m.start(), m.end()))
        .collect()
}

pub fn has_urls(text: &str) -> bool {
    !extract_urls(text).is_empty()
}

pub fn extract_single_url(text: &str) -> Option<&str> {
    let trimmed = text.trim();
    let urls = extract_urls(trimmed);
    if urls.len() != 1 {
        return None;
    }

    let (start, end) = urls[0];
    if start == 0 && end == trimmed.len() {
        Some(&trimmed[start..end])
    } else {
        None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TextSegment {
    Plain(String),
    Url(String),
}

pub fn split_text_with_urls(text: &str) -> Vec<TextSegment> {
    let mut segments = Vec::new();
    let mut last_end = 0;

    for (start, end) in extract_urls(text) {
        if start > last_end {
            segments.push(TextSegment::Plain(text[last_end..start].to_string()));
        }
        segments.push(TextSegment::Url(text[start..end].to_string()));
        last_end = end;
    }

    if last_end < text.len() {
        segments.push(TextSegment::Plain(text[last_end..].to_string()));
    }

    segments
}

const PASSWORD_LEN: usize = 25;
const PASSWORD_PREVIEW_MASK_LEN: usize = 8;

pub fn is_password(text: &str) -> bool {
    text.len() == PASSWORD_LEN
        && !text.contains(' ')
        && !text.contains("\n")
        && !text.contains("\t")
        && !has_urls(text)
}

pub fn mask_password_preview() -> String {
    "•".repeat(PASSWORD_PREVIEW_MASK_LEN)
}
