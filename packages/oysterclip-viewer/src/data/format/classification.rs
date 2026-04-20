use super::text_type::TextType;
use crate::config::settings::PasswordConfig;
use crate::data::entry::ClipboardEntry;
use common::classification::{is_password, mask_password};

pub fn entry_label(entry: &ClipboardEntry, password_config: &PasswordConfig) -> &'static str {
    match entry {
        ClipboardEntry::Text { content, kind, .. } => {
            TextType::classify(content, kind.as_deref(), password_config).label()
        }
        ClipboardEntry::Image { .. } => "Image",
    }
}

pub fn entry_icon_name(entry: &ClipboardEntry, password_config: &PasswordConfig) -> &'static str {
    match entry {
        ClipboardEntry::Text { content, kind, .. } => {
            TextType::classify(content, kind.as_deref(), password_config).icon()
        }
        ClipboardEntry::Image { .. } => "image",
    }
}

pub fn preview_text(content: &str, limit: usize, password_config: &PasswordConfig) -> String {
    if is_password(
        content,
        password_config.len,
        password_config.score_threshold,
    ) {
        mask_password()
    } else {
        let line = content.lines().next().unwrap_or("");
        let mut preview: String = line.chars().take(limit).collect();
        if line.chars().count() > limit {
            preview.push('…');
        }
        preview
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_label_image() {
        let entry = ClipboardEntry::Image {
            id: 1,
            timestamp: 0,
            hash: 0,
            path: None,
            data_url: None,
        };
        let config = PasswordConfig::default();
        assert_eq!(entry_label(&entry, &config), "Image");
    }

    #[test]
    fn test_entry_icon_name_image() {
        let entry = ClipboardEntry::Image {
            id: 1,
            timestamp: 0,
            hash: 0,
            path: None,
            data_url: None,
        };
        let config = PasswordConfig::default();
        assert_eq!(entry_icon_name(&entry, &config), "image");
    }

    #[test]
    fn test_entry_label_text() {
        let entry = ClipboardEntry::Text {
            id: 1,
            timestamp: 0,
            content: "some text".to_string(),
            kind: None,
        };
        let config = PasswordConfig::default();
        let label = entry_label(&entry, &config);
        assert!(!label.is_empty());
    }

    #[test]
    fn test_preview_text_short() {
        let config = PasswordConfig::default();
        let preview = preview_text("hello", 10, &config);
        assert_eq!(preview, "hello");
    }

    #[test]
    fn test_preview_text_truncated() {
        let config = PasswordConfig::default();
        let preview = preview_text("hello world test", 5, &config);
        assert!(preview.contains("…"));
    }

    #[test]
    fn test_preview_text_multiline() {
        let config = PasswordConfig::default();
        let preview = preview_text("line1\nline2", 20, &config);
        assert_eq!(preview, "line1");
    }

    #[test]
    fn test_preview_text_empty_string() {
        let config = PasswordConfig::default();
        let preview = preview_text("", 10, &config);
        assert_eq!(preview, "");
    }
}
