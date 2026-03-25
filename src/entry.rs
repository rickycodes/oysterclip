use serde::{Deserialize, Serialize};

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
        png_bytes: Vec<u8>,
        path: Option<String>,
        hash: u64,
    },
}
