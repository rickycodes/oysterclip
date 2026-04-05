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
    pub fn classify(content: &str, kind: Option<&str>, has_url: bool, is_html_image: bool) -> Self {
        match () {
            _ if crate::data::format::is_password(content) => Self::Password,
            _ if is_html_image => Self::HtmlImage,
            _ if has_url => Self::Link,
            _ if kind == Some("json") => Self::Json,
            _ if kind == Some("path") => Self::Path,
            _ => Self::Text,
        }
    }
}
