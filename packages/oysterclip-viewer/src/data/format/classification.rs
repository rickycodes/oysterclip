use super::text_type::TextType;
use crate::data::entry::ClipboardEntry;
use common::classification::{is_password, mask_password_preview};

pub fn entry_label(entry: &ClipboardEntry) -> &'static str {
    match entry {
        ClipboardEntry::Text { content, kind, .. } => {
            TextType::classify(content, kind.as_deref()).label()
        }
        ClipboardEntry::Image { .. } => "Image",
    }
}

pub fn entry_icon_name(entry: &ClipboardEntry) -> &'static str {
    match entry {
        ClipboardEntry::Text { content, kind, .. } => {
            TextType::classify(content, kind.as_deref()).icon()
        }
        ClipboardEntry::Image { .. } => "image",
    }
}

pub fn preview_text(content: &str, limit: usize) -> String {
    if is_password(content) {
        mask_password_preview()
    } else {
        let line = content.lines().next().unwrap_or("");
        let mut preview: String = line.chars().take(limit).collect();
        if line.chars().count() > limit {
            preview.push('…');
        }
        preview
    }
}
