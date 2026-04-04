use super::url::extract_single_url;
use crate::data::entry::ClipboardEntry;

const PASSWORD_LEN: usize = 25;
const PASSWORD_PREVIEW_MASK_LEN: usize = 8;

pub fn entry_label(entry: &ClipboardEntry) -> &'static str {
    match entry {
        ClipboardEntry::Text { content, kind, .. } => {
            if is_password(content) {
                "Pass"
            } else if extract_single_url(content).is_some() {
                "Link"
            } else if kind.as_deref() == Some("json") {
                "JSON"
            } else if kind.as_deref() == Some("path") {
                "Path"
            } else {
                "Text"
            }
        }
        ClipboardEntry::Image { .. } => "Image",
    }
}

pub fn entry_icon_name(entry: &ClipboardEntry) -> &'static str {
    match entry {
        ClipboardEntry::Text { content, kind, .. } => {
            if is_password(content) {
                "lock"
            } else if extract_single_url(content).is_some() {
                "link"
            } else if kind.as_deref() == Some("json") {
                "braces"
            } else if kind.as_deref() == Some("path") {
                "folder"
            } else {
                "file-text"
            }
        }
        ClipboardEntry::Image { .. } => "image",
    }
}

pub fn is_password(text: &str) -> bool {
    text.len() == PASSWORD_LEN
        && !text.contains(' ')
        && !text.contains("\n")
        && !text.contains("\t")
        && !super::url::has_urls(text)
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

pub fn mask_password_preview() -> String {
    "•".repeat(PASSWORD_PREVIEW_MASK_LEN)
}
