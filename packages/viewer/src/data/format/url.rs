use regex::Regex;
use std::sync::LazyLock;

// Matches https:// or http:// URLs
static SCHEME_URL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"https?://[^\s<>]+").unwrap());

// Matches bare www.example.com URLs (no scheme)
static WWW_URL_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"www\.[^\s<>]+").unwrap());

/// Characters that commonly appear at the end of text but shouldn't be part of the URL
const TRAILING_CHARS: &[char] = &['.', ',', ')', '!', ';', ':', '"', '\'', '>', ']', '}'];

/// Strip trailing punctuation from a URL
fn strip_trailing_punctuation(url: &str) -> &str {
    let mut url = url;
    while !url.is_empty() && TRAILING_CHARS.contains(&url.chars().last().unwrap()) {
        url = &url[..url.len() - url.chars().last().unwrap().len_utf8()];
    }
    url
}

pub fn extract_urls(text: &str) -> Vec<(usize, usize)> {
    let mut urls = Vec::new();

    // Find scheme-based URLs (https://, http://)
    for m in SCHEME_URL_REGEX.find_iter(text) {
        let url_text = strip_trailing_punctuation(m.as_str());
        let url_end = m.start() + url_text.len();
        urls.push((m.start(), url_end));
    }

    // Find bare www. URLs that don't overlap with already-found URLs
    for m in WWW_URL_REGEX.find_iter(text) {
        let url_text = strip_trailing_punctuation(m.as_str());
        let url_end = m.start() + url_text.len();

        // Check if this overlaps with an existing URL
        let overlaps = urls.iter().any(|(start, end)| {
            (m.start() >= *start && m.start() < *end) || (*start >= m.start() && *start < url_end)
        });

        if !overlaps {
            urls.push((m.start(), url_end));
        }
    }

    // Sort by start position for consistent ordering
    urls.sort_by_key(|(start, _)| *start);
    urls
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheme_url_basic() {
        let text = "Check this https://example.com";
        let urls = extract_urls(text);
        assert_eq!(urls.len(), 1);
        assert_eq!(&text[urls[0].0..urls[0].1], "https://example.com");
    }

    #[test]
    fn test_strip_trailing_period() {
        let text = "Visit https://example.com.";
        let urls = extract_urls(text);
        assert_eq!(urls.len(), 1);
        assert_eq!(&text[urls[0].0..urls[0].1], "https://example.com");
    }

    #[test]
    fn test_strip_trailing_comma() {
        let text = "Visit https://example.com, it's great";
        let urls = extract_urls(text);
        assert_eq!(urls.len(), 1);
        assert_eq!(&text[urls[0].0..urls[0].1], "https://example.com");
    }

    #[test]
    fn test_strip_trailing_paren() {
        let text = "Check (https://example.com)";
        let urls = extract_urls(text);
        assert_eq!(urls.len(), 1);
        assert_eq!(&text[urls[0].0..urls[0].1], "https://example.com");
    }

    #[test]
    fn test_strip_multiple_trailing_punctuation() {
        let text = "See https://example.com)!";
        let urls = extract_urls(text);
        assert_eq!(urls.len(), 1);
        assert_eq!(&text[urls[0].0..urls[0].1], "https://example.com");
    }

    #[test]
    fn test_bare_www_url() {
        let text = "Visit www.example.com";
        let urls = extract_urls(text);
        assert_eq!(urls.len(), 1);
        assert_eq!(&text[urls[0].0..urls[0].1], "www.example.com");
    }

    #[test]
    fn test_bare_www_with_trailing_period() {
        let text = "Visit www.example.com.";
        let urls = extract_urls(text);
        assert_eq!(urls.len(), 1);
        assert_eq!(&text[urls[0].0..urls[0].1], "www.example.com");
    }

    #[test]
    fn test_bare_www_with_path() {
        let text = "Check www.example.com/path/to/page";
        let urls = extract_urls(text);
        assert_eq!(urls.len(), 1);
        assert_eq!(&text[urls[0].0..urls[0].1], "www.example.com/path/to/page");
    }

    #[test]
    fn test_multiple_urls() {
        let text = "Visit https://github.com and www.example.com";
        let urls = extract_urls(text);
        assert_eq!(urls.len(), 2);
        assert_eq!(&text[urls[0].0..urls[0].1], "https://github.com");
        assert_eq!(&text[urls[1].0..urls[1].1], "www.example.com");
    }

    #[test]
    fn test_url_with_query_params() {
        let text = "Search https://example.com?q=rust&sort=stars";
        let urls = extract_urls(text);
        assert_eq!(urls.len(), 1);
        assert_eq!(
            &text[urls[0].0..urls[0].1],
            "https://example.com?q=rust&sort=stars"
        );
    }

    #[test]
    fn test_url_with_fragment() {
        let text = "Jump to https://example.com#section";
        let urls = extract_urls(text);
        assert_eq!(urls.len(), 1);
        assert_eq!(&text[urls[0].0..urls[0].1], "https://example.com#section");
    }

    #[test]
    fn test_http_url() {
        let text = "Old site http://example.com";
        let urls = extract_urls(text);
        assert_eq!(urls.len(), 1);
        assert_eq!(&text[urls[0].0..urls[0].1], "http://example.com");
    }

    #[test]
    fn test_no_urls() {
        let text = "Just plain text with no links";
        let urls = extract_urls(text);
        assert_eq!(urls.len(), 0);
    }

    #[test]
    fn test_extract_single_url_standalone() {
        let text = "https://example.com";
        let url = extract_single_url(text);
        assert_eq!(url, Some("https://example.com"));
    }

    #[test]
    fn test_extract_single_url_with_text_returns_none() {
        let text = "Visit https://example.com today";
        let url = extract_single_url(text);
        assert_eq!(url, None);
    }

    #[test]
    fn test_split_text_with_single_url() {
        let text = "Visit https://example.com please";
        let segments = split_text_with_urls(text);
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0], TextSegment::Plain("Visit ".to_string()));
        assert_eq!(
            segments[1],
            TextSegment::Url("https://example.com".to_string())
        );
        assert_eq!(segments[2], TextSegment::Plain(" please".to_string()));
    }

    #[test]
    fn test_split_text_with_multiple_urls() {
        let text = "Check https://github.com and www.rust-lang.org";
        let segments = split_text_with_urls(text);
        assert_eq!(segments.len(), 4);
        assert_eq!(segments[0], TextSegment::Plain("Check ".to_string()));
        assert_eq!(
            segments[1],
            TextSegment::Url("https://github.com".to_string())
        );
        assert_eq!(segments[2], TextSegment::Plain(" and ".to_string()));
        assert_eq!(
            segments[3],
            TextSegment::Url("www.rust-lang.org".to_string())
        );
    }

    #[test]
    fn test_has_urls() {
        assert!(has_urls("Check https://example.com"));
        assert!(has_urls("Visit www.example.com"));
        assert!(!has_urls("No links here"));
    }
}
