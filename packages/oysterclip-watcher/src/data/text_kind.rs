/// Classifies different text content types detected during clipboard monitoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextKind {
    Empty,
    ImageDataUri,
    Url,
    Json,
    Multiline,
    Path,
    Plain,
}

impl TextKind {
    /// Get the corresponding string constant for storage/display.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Empty => crate::config::constants::TEXT_KIND_EMPTY,
            Self::ImageDataUri => crate::config::constants::TEXT_KIND_IMAGE_DATA_URI,
            Self::Url => crate::config::constants::TEXT_KIND_URL,
            Self::Json => crate::config::constants::TEXT_KIND_JSON,
            Self::Multiline => crate::config::constants::TEXT_KIND_MULTILINE,
            Self::Path => crate::config::constants::TEXT_KIND_PATH,
            Self::Plain => crate::config::constants::TEXT_KIND_PLAIN,
        }
    }

    /// Classify text content based on its properties.
    /// Priority order matters: check more specific types first.
    pub fn classify(text: &str) -> Self {
        let trimmed = text.trim();

        match () {
            _ if trimmed.is_empty() => Self::Empty,
            _ if is_image_data_url(trimmed) => Self::ImageDataUri,
            _ if trimmed.starts_with("http://") || trimmed.starts_with("https://") => Self::Url,
            _ if is_json_object_or_array(trimmed) => Self::Json,
            _ if trimmed.contains('\n') || trimmed.contains('\r') => Self::Multiline,
            _ if is_file_path(trimmed) => Self::Path,
            _ => Self::Plain,
        }
    }
}

fn is_image_data_url(text: &str) -> bool {
    text.starts_with("data:image/") && text.contains(";base64,")
}

fn is_json_object_or_array(text: &str) -> bool {
    matches!(
        serde_json::from_str::<serde_json::Value>(text),
        Ok(serde_json::Value::Object(_)) | Ok(serde_json::Value::Array(_))
    )
}

fn is_file_path(text: &str) -> bool {
    // Unix absolute path: /something
    if text.starts_with('/') && text.len() > 1 {
        return true;
    }
    // Unix home-relative path: ~/something
    if text.starts_with("~/") {
        return true;
    }
    // Windows absolute path: C:\ or C:/
    let b = text.as_bytes();
    if b.len() >= 3 && b[0].is_ascii_alphabetic() && b[1] == b':' && (b[2] == b'\\' || b[2] == b'/')
    {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_empty_string() {
        assert_eq!(TextKind::classify(""), TextKind::Empty);
        assert_eq!(TextKind::classify("   "), TextKind::Empty);
        assert_eq!(TextKind::classify("\n\t"), TextKind::Empty);
    }

    #[test]
    fn test_classify_image_data_uri_valid() {
        let uri = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
        assert_eq!(TextKind::classify(uri), TextKind::ImageDataUri);
    }

    #[test]
    fn test_classify_image_data_uri_jpeg() {
        let uri = "data:image/jpeg;base64,/9j/4AAQSkZJRg==";
        assert_eq!(TextKind::classify(uri), TextKind::ImageDataUri);
    }

    #[test]
    fn test_classify_image_data_uri_invalid_no_base64() {
        let uri = "data:image/png,iVBORw0KGgoAAAA";
        assert_ne!(TextKind::classify(uri), TextKind::ImageDataUri);
    }

    #[test]
    fn test_classify_url_https() {
        assert_eq!(TextKind::classify("https://example.com"), TextKind::Url);
    }

    #[test]
    fn test_classify_url_http() {
        assert_eq!(TextKind::classify("http://example.com/path"), TextKind::Url);
    }

    #[test]
    fn test_classify_url_with_query() {
        assert_eq!(
            TextKind::classify("https://example.com?key=value"),
            TextKind::Url
        );
    }

    #[test]
    fn test_classify_json_object() {
        assert_eq!(TextKind::classify(r#"{"key":"value"}"#), TextKind::Json);
    }

    #[test]
    fn test_classify_json_array() {
        assert_eq!(
            TextKind::classify(r#"[1, 2, {"nested": true}]"#),
            TextKind::Json
        );
    }

    #[test]
    fn test_classify_json_nested() {
        let json = r#"{"users":[{"name":"Alice","age":30},{"name":"Bob","age":25}]}"#;
        assert_eq!(TextKind::classify(json), TextKind::Json);
    }

    #[test]
    fn test_classify_json_invalid_is_not_json() {
        assert_ne!(TextKind::classify("{invalid json}"), TextKind::Json);
    }

    #[test]
    fn test_classify_multiline_with_newline() {
        assert_eq!(
            TextKind::classify("line1\nline2"),
            TextKind::Multiline
        );
    }

    #[test]
    fn test_classify_multiline_with_carriage_return() {
        assert_eq!(
            TextKind::classify("line1\rline2"),
            TextKind::Multiline
        );
    }

    #[test]
    fn test_classify_path_unix_absolute() {
        assert_eq!(TextKind::classify("/home/user/file.txt"), TextKind::Path);
        assert_eq!(TextKind::classify("/etc/config"), TextKind::Path);
    }

    #[test]
    fn test_classify_path_unix_home() {
        assert_eq!(TextKind::classify("~/Documents/file.txt"), TextKind::Path);
        assert_eq!(TextKind::classify("~/.config/app"), TextKind::Path);
    }

    #[test]
    fn test_classify_path_windows_backslash() {
        assert_eq!(TextKind::classify("C:\\Users\\Admin\\file.txt"), TextKind::Path);
        assert_eq!(TextKind::classify("D:\\data\\"), TextKind::Path);
    }

    #[test]
    fn test_classify_path_windows_forward_slash() {
        assert_eq!(TextKind::classify("C:/Users/Admin/file.txt"), TextKind::Path);
        assert_eq!(TextKind::classify("E:/projects/rust"), TextKind::Path);
    }

    #[test]
    fn test_classify_path_invalid_single_slash() {
        assert_ne!(TextKind::classify("/"), TextKind::Path);
    }

    #[test]
    fn test_classify_plain_text() {
        assert_eq!(TextKind::classify("Hello, World!"), TextKind::Plain);
        assert_eq!(TextKind::classify("Some regular text"), TextKind::Plain);
        assert_eq!(
            TextKind::classify("This is not a URL or path or JSON"),
            TextKind::Plain
        );
    }

    #[test]
    fn test_classify_plain_single_path_component() {
        assert_eq!(TextKind::classify("filename.txt"), TextKind::Plain);
        assert_eq!(TextKind::classify("document"), TextKind::Plain);
    }

    #[test]
    fn test_classify_priority_url_over_plain() {
        // Text starting with URL is still classified as Url (trimmed check)
        assert_eq!(
            TextKind::classify("https://example.com is cool"),
            TextKind::Url
        ); // Starts with https://
    }

    #[test]
    fn test_classify_priority_json_over_plain() {
        // JSON object takes priority
        assert_eq!(
            TextKind::classify(r#"{"data": 123}"#),
            TextKind::Json
        );
    }

    #[test]
    fn test_text_kind_as_str_all_variants() {
        assert!(!TextKind::Empty.as_str().is_empty());
        assert!(!TextKind::ImageDataUri.as_str().is_empty());
        assert!(!TextKind::Url.as_str().is_empty());
        assert!(!TextKind::Json.as_str().is_empty());
        assert!(!TextKind::Multiline.as_str().is_empty());
        assert!(!TextKind::Path.as_str().is_empty());
        assert!(!TextKind::Plain.as_str().is_empty());
    }

    #[test]
    fn test_text_kind_as_str_unique() {
        let strs = vec![
            TextKind::Empty.as_str(),
            TextKind::ImageDataUri.as_str(),
            TextKind::Url.as_str(),
            TextKind::Json.as_str(),
            TextKind::Multiline.as_str(),
            TextKind::Path.as_str(),
            TextKind::Plain.as_str(),
        ];
        let mut sorted = strs.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), 7, "All TextKind variants should have unique strings");
    }
}
