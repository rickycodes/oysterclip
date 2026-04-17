use regex::Regex;
use std::sync::LazyLock;
use zxcvbn::Score;

pub const PASSWORD_LEN: usize = 25;
const PASSWORD_PREVIEW_MASK_LEN: usize = 8;

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

pub fn is_password(text: &str, password_len: Option<usize>, score_threshold: u8) -> bool {
    let threshold = [
        Score::Zero,
        Score::One,
        Score::Two,
        Score::Three,
        Score::Four,
    ]
    .get(score_threshold.min(4) as usize)
    .copied()
    .unwrap_or(Score::Four);

    let len_matches = if let Some(len) = password_len {
        text.len() == len
    } else {
        true // If length check is disabled, always match
    };

    len_matches
        && !text.contains(' ')
        && !text.contains('\n')
        && !text.contains('\t')
        && !has_urls(text)
        && zxcvbn::zxcvbn(text, &[]).score() >= threshold
}

pub fn mask_password() -> String {
    "•".repeat(PASSWORD_PREVIEW_MASK_LEN)
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
    fn test_has_urls() {
        assert!(has_urls("Check https://example.com"));
        assert!(has_urls("Visit www.example.com"));
        assert!(!has_urls("No links here"));
    }

    #[test]
    fn test_is_password_with_exact_length() {
        let password = "MyStr0ng!P@ssw0rdP@ss1234";
        assert_eq!(password.len(), PASSWORD_LEN);
        assert!(is_password(&password, Some(PASSWORD_LEN), 0));
    }

    #[test]
    fn test_is_password_wrong_length() {
        let password = "MyStr0ng!P@ssw0rdP@ss12345";
        assert_eq!(password.len(), PASSWORD_LEN + 1);
        assert!(!is_password(&password, Some(PASSWORD_LEN), 0));
    }

    #[test]
    fn test_is_password_no_length_check() {
        let short_password = "MyStr0ng!P@ss";
        assert!(is_password(short_password, None, 0));
    }

    #[test]
    fn test_is_password_with_spaces() {
        let password = "MyStr0ng!P@ssw0rdP@ss1234 ";
        assert!(!is_password(&password, Some(PASSWORD_LEN + 1), 0));
    }

    #[test]
    fn test_is_password_with_newline() {
        let password = "MyStr0ng!P@ssw0rdP@ss1234\n";
        assert!(!is_password(&password, Some(PASSWORD_LEN + 1), 0));
    }

    #[test]
    fn test_is_password_with_url() {
        let password = "MyStr0ng!P@ssw0rdP@sshttps://example.com";
        assert!(!is_password(&password, None, 0));
    }

    #[test]
    fn test_is_password_weak_score() {
        let weak = "aaaaaa";
        assert!(!is_password(weak, None, 3));
    }

    #[test]
    fn test_is_password_score_threshold_clamping() {
        let password = "MyStr0ng!P@ssw0rdP@ss1234";
        // Threshold 5+ should clamp to 4 (Score::Four)
        assert!(is_password(&password, Some(PASSWORD_LEN), 5));
        assert!(is_password(&password, Some(PASSWORD_LEN), 255));
    }
}
