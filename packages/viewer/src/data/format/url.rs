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
