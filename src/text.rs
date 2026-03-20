use crate::constants::{
    TEXT_KIND_EMPTY, TEXT_KIND_IMAGE_DATA_URI, TEXT_KIND_JSON, TEXT_KIND_MULTILINE,
    TEXT_KIND_PLAIN, TEXT_KIND_URL,
};

pub(crate) fn detect_text_kind(text: &str) -> &'static str {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return TEXT_KIND_EMPTY;
    }

    if is_image_data_url(trimmed) {
        return TEXT_KIND_IMAGE_DATA_URI;
    }

    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return TEXT_KIND_URL;
    }

    if serde_json::from_str::<serde_json::Value>(trimmed).is_ok() {
        return TEXT_KIND_JSON;
    }

    if trimmed.contains('\n') || trimmed.contains('\r') {
        return TEXT_KIND_MULTILINE;
    }

    TEXT_KIND_PLAIN
}

fn is_image_data_url(text: &str) -> bool {
    text.starts_with("data:image/") && text.contains(";base64,")
}

#[cfg(test)]
mod tests {
    use super::detect_text_kind;
    use crate::constants::{
        TEXT_KIND_EMPTY, TEXT_KIND_IMAGE_DATA_URI, TEXT_KIND_JSON, TEXT_KIND_MULTILINE,
        TEXT_KIND_PLAIN, TEXT_KIND_URL,
    };

    #[test]
    fn detect_text_kind_classifies_common_types() {
        assert_eq!(detect_text_kind("https://example.com"), TEXT_KIND_URL);
        assert_eq!(
            detect_text_kind("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAA"),
            TEXT_KIND_IMAGE_DATA_URI
        );
        assert_eq!(detect_text_kind("{\"a\":1}"), TEXT_KIND_JSON);
        assert_eq!(detect_text_kind("line1\nline2"), TEXT_KIND_MULTILINE);
        assert_eq!(detect_text_kind("hello"), TEXT_KIND_PLAIN);
        assert_eq!(detect_text_kind("   "), TEXT_KIND_EMPTY);
    }
}
