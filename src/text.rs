pub(crate) fn detect_text_kind(text: &str) -> &'static str {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return "empty";
    }

    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return "url";
    }

    if serde_json::from_str::<serde_json::Value>(trimmed).is_ok() {
        return "json";
    }

    if trimmed.contains('\n') || trimmed.contains('\r') {
        return "multiline";
    }

    "plain"
}

#[cfg(test)]
mod tests {
    use super::detect_text_kind;

    #[test]
    fn detect_text_kind_classifies_common_types() {
        assert_eq!(detect_text_kind("https://example.com"), "url");
        assert_eq!(detect_text_kind("{\"a\":1}"), "json");
        assert_eq!(detect_text_kind("line1\nline2"), "multiline");
        assert_eq!(detect_text_kind("hello"), "plain");
        assert_eq!(detect_text_kind("   "), "empty");
    }
}
