use base64::{engine::general_purpose, Engine as _};
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use chrono::{DateTime, Local, Utc};
use keyring::Entry;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

const KEYRING_SERVICE: &str = "clipboard-manager";
const KEYRING_ACCOUNT: &str = "default-encryption-key";

#[derive(Clone)]
pub struct ClipboardSource {
    kind: SourceKind,
    error: Option<String>,
}

#[derive(Clone)]
enum SourceKind {
    File(PathBuf),
    RawJson(String),
    Empty,
}

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

impl ClipboardSource {
    pub fn from_env() -> Self {
        let mut args = env::args().skip(1);
        let Some(arg) = args.next() else {
            return Self {
                kind: SourceKind::Empty,
                error: Some("Missing clipboard history argument.".to_string()),
            };
        };

        let trimmed = arg.trim();
        if trimmed.starts_with('[') || trimmed.starts_with('{') {
            Self {
                kind: SourceKind::RawJson(trimmed.to_string()),
                error: None,
            }
        } else {
            Self {
                kind: SourceKind::File(PathBuf::from(trimmed)),
                error: None,
            }
        }
    }
}

pub fn delete_entry(source: &ClipboardSource, id: i64) -> Result<(), String> {
    let path = source_file_path(source)?;
    let conn = Connection::open(path)
        .map_err(|e| format!("Failed to open history database: {e}"))?;
    conn.execute("DELETE FROM entries WHERE id = ?1", [id])
        .map_err(|e| format!("Failed to delete history entry: {e}"))?;
    Ok(())
}

pub fn clear_history(source: &ClipboardSource) -> Result<(), String> {
    let path = source_file_path(source)?;
    let conn = Connection::open(path)
        .map_err(|e| format!("Failed to open history database: {e}"))?;
    conn.execute("DELETE FROM entries", [])
        .map_err(|e| format!("Failed to clear history: {e}"))?;
    Ok(())
}

fn source_file_path(source: &ClipboardSource) -> Result<&Path, String> {
    if let Some(err) = source.error.as_ref() {
        return Err(err.clone());
    }

    match &source.kind {
        SourceKind::File(path) => Ok(path.as_path()),
        SourceKind::RawJson(_) => Err("History mutations are not supported for raw JSON input.".to_string()),
        SourceKind::Empty => Err("Missing clipboard history argument.".to_string()),
    }
}

pub fn get_clipboard_entries(
    source: &ClipboardSource,
    cache: &mut Option<CachedEntries>,
) -> ClipboardPayload {
    let stamp = match source_stamp(source) {
        Ok(stamp) => stamp,
        Err(err) => {
            return ClipboardPayload {
                entries: Vec::new(),
                error: Some(err),
            };
        }
    };

    if let Some(cached) = cache.as_ref() {
        if cached.stamp == stamp {
            return ClipboardPayload {
                entries: cached.entries.clone(),
                error: None,
            };
        }
    }

    match load_entries(source) {
        Ok(entries) => {
            *cache = Some(CachedEntries {
                stamp,
                entries: entries.clone(),
            });
            ClipboardPayload {
                entries,
                error: None,
            }
        }
        Err(err) => ClipboardPayload {
            entries: Vec::new(),
            error: Some(err),
        },
    }
}

pub fn preview_text(content: &str, limit: usize) -> String {
    let line = content.lines().next().unwrap_or("");
    let mut preview: String = line.chars().take(limit).collect();
    if line.chars().count() > limit {
        preview.push('…');
    }
    preview
}

pub fn is_image_data_uri(content: &str) -> bool {
    let trimmed = content.trim();
    trimmed.starts_with("data:image/") && trimmed.contains(";base64,")
}

pub fn image_data_uri_summary(content: &str) -> String {
    let trimmed = content.trim();
    let media_type = trimmed
        .strip_prefix("data:")
        .and_then(|value| value.split(';').next())
        .unwrap_or("image data");
    format!("{} hidden for readability ({} chars)", media_type, trimmed.chars().count())
}

pub fn entry_label(entry: &ClipboardEntry) -> &'static str {
    match entry {
        ClipboardEntry::Text { .. } => "Text",
        ClipboardEntry::Image { .. } => "Image",
    }
}

pub fn format_timestamp(timestamp: u64) -> String {
    if let Some(utc) = DateTime::<Utc>::from_timestamp(timestamp as i64, 0) {
        utc.with_timezone(&Local)
            .format("%A, %b %d, %Y %I:%M %p")
            .to_string()
    } else {
        timestamp.to_string()
    }
}

fn source_stamp(source: &ClipboardSource) -> Result<SourceStamp, String> {
    if let Some(err) = source.error.as_ref() {
        return Err(err.clone());
    }

    match &source.kind {
        SourceKind::File(path) => {
            let metadata = fs::metadata(path)
                .map_err(|e| format!("Failed to read history file metadata: {e}"))?;
            Ok(SourceStamp::File {
                path: path.clone(),
                modified: metadata.modified().ok(),
                size: metadata.len(),
            })
        }
        SourceKind::RawJson(json) => Ok(SourceStamp::RawJson {
            hash: hash_str(json),
            len: json.len(),
        }),
        SourceKind::Empty => Err("Missing clipboard history argument.".to_string()),
    }
}

fn load_entries(source: &ClipboardSource) -> Result<Vec<ClipboardEntry>, String> {
    if let Some(err) = source.error.as_ref() {
        return Err(err.clone());
    }

    match &source.kind {
        SourceKind::File(path) => load_entries_from_db(path),
        SourceKind::RawJson(json) => serde_json::from_str(json)
            .map_err(|e| format!("Invalid clipboard JSON: {e}")),
        SourceKind::Empty => Err("Missing clipboard history argument.".to_string()),
    }
}

fn load_entries_from_db(path: &Path) -> Result<Vec<ClipboardEntry>, String> {
    let conn = Connection::open(path)
        .map_err(|e| format!("Failed to open history database: {e}"))?;
    let mut stmt = conn
        .prepare(
            "SELECT id, created_at, entry_type, text_kind, text_ciphertext, text_nonce, image_path, image_hash FROM entries ORDER BY id ASC",
        )
        .map_err(|e| format!("Failed to prepare history query: {e}"))?;
    let mut rows = stmt
        .query([])
        .map_err(|e| format!("Failed to query history database: {e}"))?;

    let key = load_encryption_key()?;
    let base_dir = path.parent();
    let mut entries = Vec::new();

    while let Some(row) = rows
        .next()
        .map_err(|e| format!("Failed to iterate history rows: {e}"))?
    {
        let id: i64 = row
            .get(0)
            .map_err(|e| format!("Failed to read entry id: {e}"))?;
        let timestamp: i64 = row
            .get(1)
            .map_err(|e| format!("Failed to read entry timestamp: {e}"))?;
        let entry_type: String = row
            .get(2)
            .map_err(|e| format!("Failed to read entry type: {e}"))?;

        match entry_type.as_str() {
            "text" => {
                let kind: Option<String> = row
                    .get(3)
                    .map_err(|e| format!("Failed to read text kind: {e}"))?;
                let ciphertext: Option<Vec<u8>> = row
                    .get(4)
                    .map_err(|e| format!("Failed to read encrypted text content: {e}"))?;
                let nonce: Option<Vec<u8>> = row
                    .get(5)
                    .map_err(|e| format!("Failed to read text nonce: {e}"))?;
                let content = decrypt_text(
                    ciphertext
                        .as_deref()
                        .ok_or_else(|| "Missing encrypted text content.".to_string())?,
                    nonce
                        .as_deref()
                        .ok_or_else(|| "Missing text nonce.".to_string())?,
                    &key,
                )?;

                entries.push(ClipboardEntry::Text {
                    id,
                    timestamp: timestamp as u64,
                    content,
                    kind,
                });
            }
            "image" => {
                let image_path: Option<String> = row
                    .get(6)
                    .map_err(|e| format!("Failed to read image path: {e}"))?;
                let image_hash: Option<i64> = row
                    .get(7)
                    .map_err(|e| format!("Failed to read image hash: {e}"))?;
                let path = image_path.ok_or_else(|| "Missing image path.".to_string())?;
                let hash = image_hash.ok_or_else(|| "Missing image hash.".to_string())? as u64;
                let resolved = resolve_image_path(base_dir, &path);
                let data_url = resolved.and_then(|resolved_path| {
                    fs::read(resolved_path).ok().map(|bytes| {
                        format!(
                            "data:image/png;base64,{}",
                            general_purpose::STANDARD.encode(bytes)
                        )
                    })
                });

                entries.push(ClipboardEntry::Image {
                    id,
                    timestamp: timestamp as u64,
                    path,
                    hash,
                    data_url,
                });
            }
            other => return Err(format!("Unknown history entry type: {other}")),
        }
    }

    Ok(entries)
}

fn load_encryption_key() -> Result<[u8; 32], String> {
    let entry = Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)
        .map_err(|e| format!("Failed to access OS keychain entry: {e}"))?;
    let encoded = entry
        .get_password()
        .map_err(|e| format!("Failed to read encryption key from OS keychain: {e}"))?;
    let decoded = general_purpose::STANDARD
        .decode(encoded)
        .map_err(|e| format!("Failed to decode keychain encryption key: {e}"))?;

    if decoded.len() != 32 {
        return Err(format!(
            "Invalid key length in keychain: expected 32 bytes, got {}",
            decoded.len()
        ));
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&decoded);
    Ok(key)
}

fn decrypt_text(ciphertext: &[u8], nonce: &[u8], key: &[u8; 32]) -> Result<String, String> {
    if nonce.len() != 24 {
        return Err(format!(
            "Invalid text nonce length: expected 24 bytes, got {}",
            nonce.len()
        ));
    }

    let cipher = XChaCha20Poly1305::new(key.into());
    let plaintext = cipher
        .decrypt(XNonce::from_slice(nonce), ciphertext)
        .map_err(|e| format!("Failed to decrypt clipboard text: {e}"))?;

    String::from_utf8(plaintext)
        .map_err(|e| format!("Failed to decode decrypted clipboard text: {e}"))
}

fn resolve_image_path(base_dir: Option<&Path>, path_str: &str) -> Option<PathBuf> {
    let path = Path::new(path_str);
    if path.is_absolute() {
        return Some(path.to_path_buf());
    }
    base_dir.map(|base| base.join(path))
}

fn hash_str(value: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}
