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
