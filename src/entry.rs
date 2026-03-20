use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Clone, PartialEq, Eq)]
pub enum SourceStamp {
    File {
        path: PathBuf,
        modified: Option<SystemTime>,
        size: u64,
    },
    RawJson {
        hash: u64,
        len: usize,
    },
}

#[derive(Clone)]
pub struct CachedEntries {
    pub stamp: SourceStamp,
    pub entries: Vec<ClipboardEntry>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum ClipboardEntry {
    Text {
        id: i64,
        timestamp: u64,
        content: String,
        kind: Option<String>,
    },
    Image {
        id: i64,
        timestamp: u64,
        path: String,
        hash: u64,
        data_url: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct ClipboardPayload {
    pub entries: Vec<ClipboardEntry>,
    pub error: Option<String>,
}
