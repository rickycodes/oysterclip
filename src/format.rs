use chrono::{DateTime, Local, Utc};

use crate::entry::ClipboardEntry;

pub fn preview_text(content: &str, limit: usize) -> String {
    let line = content.lines().next().unwrap_or("");
    let mut preview: String = line.chars().take(limit).collect();
    if line.chars().count() > limit {
        preview.push('…');
    }
    preview
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
        ClipboardEntry::Text { .. } => "Text",
        ClipboardEntry::Image { .. } => "Image",
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
