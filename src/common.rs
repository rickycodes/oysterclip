use serde::{Deserialize, Serialize};

pub(crate) const INTERVAL_MS: u64 = 500;
pub(crate) const HISTORY_FILE: &str = ".clipboard_history.json";
pub(crate) const IMAGE_DIR: &str = "clipboard_images";
pub(crate) const CLIPBOARD_NOT_AVAILABLE: &str = "Clipboard not available";
pub(crate) const FAILED_IMAGE_BUFFER: &str = "Failed to create image buffer";

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub(crate) enum PasteEntry {
    Text {
        timestamp: u64,
        content: String,
        kind: Option<String>,
    },
    Image {
        timestamp: u64,
        path: String,
        hash: u64,
    },
}
