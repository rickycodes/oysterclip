use crate::data::format::{
    extract_html_img_src, image_data_uri_summary, is_html_img_tag, is_image_data_uri, preview_text,
};
use crate::config::settings::PasswordConfig;
use common::{TEXT_KIND_JSON, TEXT_KIND_PATH, classification::is_password_with_config};

/// Check if content is a valid JSON object or array (not just a string).
fn is_valid_json_object_or_array(content: &str) -> bool {
    matches!(
        serde_json::from_str::<serde_json::Value>(content),
        Ok(serde_json::Value::Object(_)) | Ok(serde_json::Value::Array(_))
    )
}

/// Prepared display data for a text entry based on its type.
#[derive(Debug, Clone)]
pub struct TextDisplayData {
    pub display_text: String,
    pub summary: Option<String>,
    pub html_image_src: Option<String>,
    pub pretty_json: Option<String>,
}

/// Classifies different text content types for display purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDetailType {
    Password,
    HtmlImage,
    Link,
    Json,
    Path,
    Text,
}

impl TextDetailType {
    pub fn label(self) -> &'static str {
        match self {
            Self::Password => "Password",
            Self::HtmlImage => "HTML Image",
            Self::Link => "Link",
            Self::Json => "JSON",
            Self::Path => "Path",
            Self::Text => "Text",
        }
    }

    /// Classify text content based on its properties.
    /// Priority order matters: check more specific types first.
    pub fn classify(content: &str, kind: Option<&str>, has_url: bool, is_html_image: bool, password_config: &PasswordConfig) -> Self {
        match () {
            _ if is_password_with_config(content, password_config.len, password_config.score_threshold) => Self::Password,
            _ if is_html_image => Self::HtmlImage,
            _ if has_url => Self::Link,
            _ if kind == Some(TEXT_KIND_JSON) && is_valid_json_object_or_array(content) => {
                Self::Json
            }
            _ if kind == Some(TEXT_KIND_PATH) => Self::Path,
            _ => Self::Text,
        }
    }

    /// Extract and prepare all display data for this text type.
    pub fn extract_display_data(self, content: &str, password_config: &PasswordConfig) -> TextDisplayData {
        let is_data_uri = is_image_data_uri(content);
        let is_html_image = is_html_img_tag(content);
        let is_json = self == Self::Json;

        TextDisplayData {
            display_text: if is_data_uri {
                preview_text(content, 96, password_config)
            } else {
                content.to_string()
            },
            summary: if is_data_uri {
                Some(image_data_uri_summary(content))
            } else {
                None
            },
            html_image_src: if is_html_image {
                extract_html_img_src(content)
            } else {
                None
            },
            pretty_json: if is_json && is_valid_json_object_or_array(content) {
                serde_json::from_str::<serde_json::Value>(content)
                    .ok()
                    .and_then(|v| serde_json::to_string_pretty(&v).ok())
            } else {
                None
            },
        }
    }
}
