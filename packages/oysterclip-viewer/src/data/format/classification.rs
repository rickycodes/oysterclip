use super::text_type::TextType;
use crate::config::settings::PasswordConfig;
use crate::data::entry::ClipboardEntry;
use common::classification::{is_password_with_config, mask_password};

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
    if is_password_with_config(
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
