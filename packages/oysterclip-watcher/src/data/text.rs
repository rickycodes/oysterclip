use super::text_kind::TextKind;

pub(crate) fn detect_text_kind(text: &str) -> &'static str {
    TextKind::classify(text).as_str()
}

#[cfg(test)]
mod tests {
    use super::detect_text_kind;
    use crate::config::constants::{
        TEXT_KIND_EMPTY, TEXT_KIND_IMAGE_DATA_URI, TEXT_KIND_JSON, TEXT_KIND_MULTILINE,
        TEXT_KIND_PATH, TEXT_KIND_PLAIN, TEXT_KIND_URL,
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

    #[test]
    fn detect_text_kind_classifies_paths() {
        assert_eq!(
            detect_text_kind("/home/user/documents/file.txt"),
            TEXT_KIND_PATH
        );
        assert_eq!(detect_text_kind("/usr/local/bin"), TEXT_KIND_PATH);
        assert_eq!(detect_text_kind("~/Downloads/archive.zip"), TEXT_KIND_PATH);
        assert_eq!(
            detect_text_kind("C:\\Users\\Alice\\Desktop\\notes.txt"),
            TEXT_KIND_PATH
        );
        assert_eq!(detect_text_kind("D:/projects/myapp"), TEXT_KIND_PATH);
        // These should not be classified as paths
        assert_eq!(detect_text_kind("https://example.com/path"), TEXT_KIND_URL);
        assert_eq!(detect_text_kind("just some text"), TEXT_KIND_PLAIN);
        assert_eq!(detect_text_kind("/"), TEXT_KIND_PLAIN); // bare slash is not a useful path
    }
}
