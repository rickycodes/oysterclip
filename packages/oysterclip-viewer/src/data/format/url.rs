// Re-export from common for backward compatibility, but now also include extract_urls
pub use common::classification::{extract_urls, has_urls};

// Use the same logic as common for consistency
#[derive(Clone, Debug, PartialEq)]
pub enum TextSegment {
    Plain(String),
    Url(String),
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
}
