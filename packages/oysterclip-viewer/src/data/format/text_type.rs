/// Classifies text content into semantic types for display and icon selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextType {
    Password,
    Link,
    Json,
    Path,
    Text,
}

use crate::config::settings::PasswordConfig;
use common::{classification::is_password_with_config, TEXT_KIND_JSON, TEXT_KIND_PATH};

impl TextType {
    /// Get display label for this text type (used in sidebar/list).
    pub fn label(self) -> &'static str {
        match self {
            Self::Password => "Pass",
            Self::Link => "Link",
            Self::Json => "JSON",
            Self::Path => "Path",
            Self::Text => "Text",
        }
    }

    /// Get icon name for this text type.
    pub fn icon(self) -> &'static str {
        match self {
            Self::Password => "lock",
            Self::Link => "link",
            Self::Json => "braces",
            Self::Path => "folder",
            Self::Text => "file-text",
        }
    }

    /// Classify text content based on its properties.
    /// Priority order matters: check more specific types first.
    pub fn classify(content: &str, kind: Option<&str>, password_config: &PasswordConfig) -> Self {
        match () {
            _ if is_password_with_config(content, password_config.len, password_config.score_threshold) => Self::Password,
            _ if super::url::extract_single_url(content).is_some() => Self::Link,
            _ if kind == Some(TEXT_KIND_JSON) => Self::Json,
            _ if kind == Some(TEXT_KIND_PATH) => Self::Path,
            _ => Self::Text,
        }
    }
}
