use base64::engine::general_purpose;
use base64::Engine as _;
use rusqlite::Connection;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;

use crate::config::source::ClipboardSource;
use crate::data::entry::{CachedEntries, ClipboardEntry, ClipboardPayload, SourceStamp};
use common::crypto::{decrypt_text, get_or_create_key};

pub fn delete_entry(source: &ClipboardSource, id: i64) -> Result<(), String> {
    let path = source.file_path()?;
    let conn =
        Connection::open(path).map_err(|e| format!("Failed to open history database: {e}"))?;
    conn.execute("DELETE FROM entries WHERE id = ?1", [id])
        .map_err(|e| format!("Failed to delete history entry: {e}"))?;
    Ok(())
}

pub fn delete_entries(source: &ClipboardSource, ids: &[i64]) -> Result<(), String> {
    if ids.is_empty() {
        return Ok(());
    }
    let path = source.file_path()?;
    let conn =
        Connection::open(path).map_err(|e| format!("Failed to open history database: {e}"))?;
    let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!("DELETE FROM entries WHERE id IN ({placeholders})");
    conn.execute(&sql, rusqlite::params_from_iter(ids.iter()))
        .map_err(|e| format!("Failed to bulk delete history entries: {e}"))?;
    Ok(())
}

pub fn clear_history(source: &ClipboardSource) -> Result<(), String> {
    let path = source.file_path()?;
    let conn =
        Connection::open(path).map_err(|e| format!("Failed to open history database: {e}"))?;
    conn.execute("DELETE FROM entries", [])
        .map_err(|e| format!("Failed to clear history: {e}"))?;
    Ok(())
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

fn source_stamp(source: &ClipboardSource) -> Result<SourceStamp, String> {
    if let Some(err) = source.error() {
        return Err(err.to_string());
    }

    if let Some(json) = source.raw_json() {
        return Ok(SourceStamp::RawJson {
            hash: hash_str(json),
            len: json.len(),
        });
    }

    let path = source.file_path()?;
    let metadata =
        fs::metadata(path).map_err(|e| format!("Failed to read history file metadata: {e}"))?;
    Ok(SourceStamp::File {
        path: path.to_path_buf(),
        modified: metadata.modified().ok(),
        size: metadata.len(),
    })
}

fn load_entries(source: &ClipboardSource) -> Result<Vec<ClipboardEntry>, String> {
    if let Some(err) = source.error() {
        return Err(err.to_string());
    }

    if let Some(json) = source.raw_json() {
        return serde_json::from_str(json).map_err(|e| format!("Invalid clipboard JSON: {e}"));
    }

    load_entries_from_db(source.file_path()?)
}

fn load_entries_from_db(path: &Path) -> Result<Vec<ClipboardEntry>, String> {
    let conn =
        Connection::open(path).map_err(|e| format!("Failed to open history database: {e}"))?;
    let has_image_blob = has_column(&conn, "entries", "image_png")?;
    let mut stmt = conn
        .prepare(
            if has_image_blob {
                "SELECT id, created_at, entry_type, text_kind, text_ciphertext, text_nonce, image_path, image_png, image_hash FROM entries ORDER BY id ASC"
            } else {
                "SELECT id, created_at, entry_type, text_kind, text_ciphertext, text_nonce, image_path, image_hash FROM entries ORDER BY id ASC"
            },
        )
        .map_err(|e| format!("Failed to prepare history query: {e}"))?;
    let mut rows = stmt
        .query([])
        .map_err(|e| format!("Failed to query history database: {e}"))?;

    let key = load_encryption_key()?;
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
                )
                .map_err(|e| format!("Failed to decrypt text: {e}"))?;

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
                let image_png: Option<Vec<u8>> = if has_image_blob {
                    row.get(7)
                        .map_err(|e| format!("Failed to read image blob: {e}"))?
                } else {
                    None
                };
                let image_hash: Option<i64> = row
                    .get(if has_image_blob { 8 } else { 7 })
                    .map_err(|e| format!("Failed to read image hash: {e}"))?;
                let hash = image_hash.ok_or_else(|| "Missing image hash.".to_string())? as u64;
                let data_url = image_png.map(data_url_from_png).or_else(|| {
                    load_image_data_url_from_path(path.parent(), image_path.as_deref())
                });

                entries.push(ClipboardEntry::Image {
                    id,
                    timestamp: timestamp as u64,
                    path: image_path,
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
    get_or_create_key().map_err(|e| e.to_string())
}

fn has_column(conn: &Connection, table_name: &str, column_name: &str) -> Result<bool, String> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table_name})"))
        .map_err(|e| format!("Failed to inspect history schema: {e}"))?;
    let mut rows = stmt
        .query([])
        .map_err(|e| format!("Failed to query history schema: {e}"))?;

    while let Some(row) = rows
        .next()
        .map_err(|e| format!("Failed to iterate history schema: {e}"))?
    {
        let name: String = row
            .get(1)
            .map_err(|e| format!("Failed to read history schema column: {e}"))?;
        if name == column_name {
            return Ok(true);
        }
    }

    Ok(false)
}

fn data_url_from_png(bytes: Vec<u8>) -> String {
    format!(
        "data:image/png;base64,{}",
        general_purpose::STANDARD.encode(bytes)
    )
}

fn load_image_data_url_from_path(
    base_dir: Option<&Path>,
    path_str: Option<&str>,
) -> Option<String> {
    let path_str = path_str?;
    let path = Path::new(path_str);
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        base_dir?.join(path)
    };

    fs::read(resolved).ok().map(data_url_from_png)
}

fn hash_str(value: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}
