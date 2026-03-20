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
        path: String,
        hash: u64,
    },
}
