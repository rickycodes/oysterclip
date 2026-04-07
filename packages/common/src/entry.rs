use crate::{ENTRY_TYPE_IMAGE, ENTRY_TYPE_TEXT};
use serde::{Deserialize, Serialize};

/// Entry type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryType {
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "image")]
    Image,
}

impl EntryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EntryType::Text => ENTRY_TYPE_TEXT,
            EntryType::Image => ENTRY_TYPE_IMAGE,
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            ENTRY_TYPE_TEXT => Some(EntryType::Text),
            ENTRY_TYPE_IMAGE => Some(EntryType::Image),
            _ => None,
        }
    }
}

/// StorageEntry: Exact representation of what's stored in the database.
/// This is the canonical source of truth for the storage layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageEntry {
    pub id: i64,
    pub created_at: u64,
    pub entry_type: EntryType,
    pub text_kind: Option<String>,
    pub text_ciphertext: Option<Vec<u8>>,
    pub text_nonce: Option<Vec<u8>>,
    pub image_path: Option<String>,
    pub image_png: Option<Vec<u8>>,
    pub image_hash: Option<u64>,
    pub content_hash: Option<String>,
}

/// CommonEntry: Minimal, shared representation of a clipboard entry.
/// Used as the common type between watcher and viewer for conversions.
/// Apps extend this with their own fields (PasteEntry for watcher, ClipboardEntry for viewer).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CommonEntry {
    Text {
        id: i64,
        timestamp: u64,
        content: String,
        kind: Option<String>,
    },
    Image {
        id: i64,
        timestamp: u64,
        path: Option<String>,
        hash: u64,
    },
}

impl CommonEntry {
    /// Get the ID of this entry
    pub fn id(&self) -> i64 {
        match self {
            CommonEntry::Text { id, .. } => *id,
            CommonEntry::Image { id, .. } => *id,
        }
    }

    /// Get the timestamp of this entry
    pub fn timestamp(&self) -> u64 {
        match self {
            CommonEntry::Text { timestamp, .. } => *timestamp,
            CommonEntry::Image { timestamp, .. } => *timestamp,
        }
    }

    /// Get the entry type
    pub fn entry_type(&self) -> EntryType {
        match self {
            CommonEntry::Text { .. } => EntryType::Text,
            CommonEntry::Image { .. } => EntryType::Image,
        }
    }
}

impl From<StorageEntry> for CommonEntry {
    fn from(entry: StorageEntry) -> Self {
        match entry.entry_type {
            EntryType::Text => CommonEntry::Text {
                id: entry.id,
                timestamp: entry.created_at,
                content: String::new(), // Caller must populate after decryption
                kind: entry.text_kind,
            },
            EntryType::Image => CommonEntry::Image {
                id: entry.id,
                timestamp: entry.created_at,
                path: entry.image_path,
                hash: entry.image_hash.unwrap_or(0),
            },
        }
    }
}
