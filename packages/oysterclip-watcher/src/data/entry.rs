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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paste_entry_text_creation() {
        let entry = PasteEntry::Text {
            timestamp: 1000,
            content: "hello".to_string(),
            kind: Some("url".to_string()),
        };
        match entry {
            PasteEntry::Text {
                timestamp,
                content,
                kind,
            } => {
                assert_eq!(timestamp, 1000);
                assert_eq!(content, "hello");
                assert_eq!(kind, Some("url".to_string()));
            }
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn test_paste_entry_text_no_kind() {
        let entry = PasteEntry::Text {
            timestamp: 2000,
            content: "plain text".to_string(),
            kind: None,
        };
        match entry {
            PasteEntry::Text {
                timestamp,
                content,
                kind,
            } => {
                assert_eq!(timestamp, 2000);
                assert_eq!(content, "plain text");
                assert_eq!(kind, None);
            }
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn test_paste_entry_image_creation() {
        let png_data = vec![137, 80, 78, 71]; // PNG magic bytes
        let entry = PasteEntry::Image {
            timestamp: 3000,
            png_bytes: png_data.clone(),
            path: Some("/tmp/image.png".to_string()),
            hash: 12345,
        };
        match entry {
            PasteEntry::Image {
                timestamp,
                png_bytes,
                path,
                hash,
            } => {
                assert_eq!(timestamp, 3000);
                assert_eq!(png_bytes, png_data);
                assert_eq!(path, Some("/tmp/image.png".to_string()));
                assert_eq!(hash, 12345);
            }
            _ => panic!("Expected Image variant"),
        }
    }

    #[test]
    fn test_paste_entry_image_no_path() {
        let entry = PasteEntry::Image {
            timestamp: 4000,
            png_bytes: vec![],
            path: None,
            hash: 999,
        };
        match entry {
            PasteEntry::Image {
                timestamp,
                path,
                hash,
                ..
            } => {
                assert_eq!(timestamp, 4000);
                assert_eq!(path, None);
                assert_eq!(hash, 999);
            }
            _ => panic!("Expected Image variant"),
        }
    }

    #[test]
    fn test_paste_entry_text_serialization() {
        let entry = PasteEntry::Text {
            timestamp: 5000,
            content: "test".to_string(),
            kind: Some("email".to_string()),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: PasteEntry = serde_json::from_str(&json).unwrap();
        match deserialized {
            PasteEntry::Text {
                timestamp,
                content,
                kind,
            } => {
                assert_eq!(timestamp, 5000);
                assert_eq!(content, "test");
                assert_eq!(kind, Some("email".to_string()));
            }
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn test_paste_entry_image_serialization() {
        let entry = PasteEntry::Image {
            timestamp: 6000,
            png_bytes: vec![1, 2, 3],
            path: Some("/home/user/pic.png".to_string()),
            hash: 555,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: PasteEntry = serde_json::from_str(&json).unwrap();
        match deserialized {
            PasteEntry::Image {
                timestamp, hash, ..
            } => {
                assert_eq!(timestamp, 6000);
                assert_eq!(hash, 555);
            }
            _ => panic!("Expected Image variant"),
        }
    }
}
