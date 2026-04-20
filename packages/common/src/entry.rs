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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_type_text_as_str() {
        assert_eq!(EntryType::Text.as_str(), "text");
    }

    #[test]
    fn test_entry_type_image_as_str() {
        assert_eq!(EntryType::Image.as_str(), "image");
    }

    #[test]
    fn test_entry_type_parse_text() {
        assert_eq!(EntryType::parse("text"), Some(EntryType::Text));
    }

    #[test]
    fn test_entry_type_parse_image() {
        assert_eq!(EntryType::parse("image"), Some(EntryType::Image));
    }

    #[test]
    fn test_entry_type_parse_invalid() {
        assert_eq!(EntryType::parse("invalid"), None);
        assert_eq!(EntryType::parse(""), None);
        assert_eq!(EntryType::parse("TEXT"), None);
    }

    #[test]
    fn test_common_entry_text_id() {
        let entry = CommonEntry::Text {
            id: 42,
            timestamp: 1000,
            content: "test".to_string(),
            kind: None,
        };
        assert_eq!(entry.id(), 42);
    }

    #[test]
    fn test_common_entry_image_id() {
        let entry = CommonEntry::Image {
            id: 99,
            timestamp: 2000,
            path: None,
            hash: 123,
        };
        assert_eq!(entry.id(), 99);
    }

    #[test]
    fn test_common_entry_text_timestamp() {
        let entry = CommonEntry::Text {
            id: 1,
            timestamp: 1234567890,
            content: "test".to_string(),
            kind: Some("url".to_string()),
        };
        assert_eq!(entry.timestamp(), 1234567890);
    }

    #[test]
    fn test_common_entry_image_timestamp() {
        let entry = CommonEntry::Image {
            id: 1,
            timestamp: 9876543210,
            path: Some("/path/to/image.png".to_string()),
            hash: 555,
        };
        assert_eq!(entry.timestamp(), 9876543210);
    }

    #[test]
    fn test_common_entry_text_entry_type() {
        let entry = CommonEntry::Text {
            id: 1,
            timestamp: 1000,
            content: "test".to_string(),
            kind: None,
        };
        assert_eq!(entry.entry_type(), EntryType::Text);
    }

    #[test]
    fn test_common_entry_image_entry_type() {
        let entry = CommonEntry::Image {
            id: 1,
            timestamp: 1000,
            path: None,
            hash: 123,
        };
        assert_eq!(entry.entry_type(), EntryType::Image);
    }

    #[test]
    fn test_storage_entry_to_common_entry_text() {
        let storage = StorageEntry {
            id: 5,
            created_at: 1000,
            entry_type: EntryType::Text,
            text_kind: Some("url".to_string()),
            text_ciphertext: Some(vec![1, 2, 3]),
            text_nonce: Some(vec![4, 5, 6]),
            image_path: None,
            image_png: None,
            image_hash: None,
            content_hash: Some("hash123".to_string()),
        };

        let common = CommonEntry::from(storage);
        match common {
            CommonEntry::Text {
                id,
                timestamp,
                kind,
                ..
            } => {
                assert_eq!(id, 5);
                assert_eq!(timestamp, 1000);
                assert_eq!(kind, Some("url".to_string()));
            }
            _ => panic!("Expected CommonEntry::Text"),
        }
    }

    #[test]
    fn test_storage_entry_to_common_entry_image_with_hash() {
        let storage = StorageEntry {
            id: 10,
            created_at: 2000,
            entry_type: EntryType::Image,
            text_kind: None,
            text_ciphertext: None,
            text_nonce: None,
            image_path: Some("/path/img.png".to_string()),
            image_png: None,
            image_hash: Some(999),
            content_hash: None,
        };

        let common = CommonEntry::from(storage);
        match common {
            CommonEntry::Image {
                id,
                timestamp,
                path,
                hash,
            } => {
                assert_eq!(id, 10);
                assert_eq!(timestamp, 2000);
                assert_eq!(path, Some("/path/img.png".to_string()));
                assert_eq!(hash, 999);
            }
            _ => panic!("Expected CommonEntry::Image"),
        }
    }

    #[test]
    fn test_storage_entry_to_common_entry_image_no_hash() {
        let storage = StorageEntry {
            id: 11,
            created_at: 2000,
            entry_type: EntryType::Image,
            text_kind: None,
            text_ciphertext: None,
            text_nonce: None,
            image_path: None,
            image_png: None,
            image_hash: None,
            content_hash: None,
        };

        let common = CommonEntry::from(storage);
        match common {
            CommonEntry::Image { hash, .. } => {
                assert_eq!(hash, 0); // Default when None
            }
            _ => panic!("Expected CommonEntry::Image"),
        }
    }

    #[test]
    fn test_entry_type_serde_text() {
        let entry_type = EntryType::Text;
        let json = serde_json::to_string(&entry_type).unwrap();
        assert_eq!(json, "\"text\"");
        let deserialized: EntryType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, EntryType::Text);
    }

    #[test]
    fn test_entry_type_serde_image() {
        let entry_type = EntryType::Image;
        let json = serde_json::to_string(&entry_type).unwrap();
        assert_eq!(json, "\"image\"");
        let deserialized: EntryType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, EntryType::Image);
    }
}
