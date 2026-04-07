use base64::engine::general_purpose;
use base64::Engine as _;
use rusqlite::Connection;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;

use crate::config::source::ClipboardSource;
use crate::data::entry::{CachedEntries, ClipboardEntry, ClipboardPayload, SourceStamp};
use common::crypto::{decrypt_text, get_or_create_key};
use common::{ENTRY_TYPE_IMAGE, ENTRY_TYPE_TEXT, ERR_OPEN_HISTORY_DB};

pub fn delete_entry(source: &ClipboardSource, id: i64) -> Result<(), String> {
    let path = source.file_path()?;
    let conn = Connection::open(path).map_err(|e| format!("{}: {e}", ERR_OPEN_HISTORY_DB))?;
    conn.execute("DELETE FROM entries WHERE id = ?1", [id])
        .map_err(|e| format!("Failed to delete history entry: {e}"))?;
    Ok(())
}

pub fn delete_entries(source: &ClipboardSource, ids: &[i64]) -> Result<(), String> {
    if ids.is_empty() {
        return Ok(());
    }
    let path = source.file_path()?;
    let conn = Connection::open(path).map_err(|e| format!("{}: {e}", ERR_OPEN_HISTORY_DB))?;
    let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!("DELETE FROM entries WHERE id IN ({placeholders})");
    conn.execute(&sql, rusqlite::params_from_iter(ids.iter()))
        .map_err(|e| format!("Failed to bulk delete history entries: {e}"))?;
    Ok(())
}

pub fn clear_history(source: &ClipboardSource) -> Result<(), String> {
    let path = source.file_path()?;
    let conn = Connection::open(path).map_err(|e| format!("{}: {e}", ERR_OPEN_HISTORY_DB))?;
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

struct RowData {
    id: i64,
    timestamp: i64,
    entry_type: String,
    text_kind: Option<String>,
    text_ciphertext: Option<Vec<u8>>,
    text_nonce: Option<Vec<u8>>,
    image_path: Option<String>,
    image_png: Option<Vec<u8>>,
    image_hash: Option<i64>,
}

fn load_entries_from_db(path: &Path) -> Result<Vec<ClipboardEntry>, String> {
    let conn = Connection::open(path).map_err(|e| format!("{}: {e}", ERR_OPEN_HISTORY_DB))?;
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
    let base_dir = path.parent();
    let mut entries = Vec::new();

    while let Some(row) = rows
        .next()
        .map_err(|e| format!("Failed to iterate history rows: {e}"))?
    {
        let row_data = read_row_data(&row, has_image_blob)?;
        let entry = match row_data.entry_type.as_str() {
            ENTRY_TYPE_TEXT => parse_text_entry(&row_data, &key)?,
            ENTRY_TYPE_IMAGE => parse_image_entry(&row_data, base_dir)?,
            other => return Err(format!("Unknown history entry type: {other}")),
        };
        entries.push(entry);
    }

    Ok(entries)
}

fn read_row_data(row: &rusqlite::Row, has_image_blob: bool) -> Result<RowData, String> {
    let id: i64 = row
        .get(0)
        .map_err(|e| format!("Failed to read entry id: {e}"))?;
    let timestamp: i64 = row
        .get(1)
        .map_err(|e| format!("Failed to read entry timestamp: {e}"))?;
    let entry_type: String = row
        .get(2)
        .map_err(|e| format!("Failed to read entry type: {e}"))?;
    let text_kind: Option<String> = row
        .get(3)
        .map_err(|e| format!("Failed to read text kind: {e}"))?;
    let text_ciphertext: Option<Vec<u8>> = row
        .get(4)
        .map_err(|e| format!("Failed to read encrypted text content: {e}"))?;
    let text_nonce: Option<Vec<u8>> = row
        .get(5)
        .map_err(|e| format!("Failed to read text nonce: {e}"))?;
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

    Ok(RowData {
        id,
        timestamp,
        entry_type,
        text_kind,
        text_ciphertext,
        text_nonce,
        image_path,
        image_png,
        image_hash,
    })
}

fn parse_text_entry(row_data: &RowData, key: &[u8; 32]) -> Result<ClipboardEntry, String> {
    let ciphertext = row_data
        .text_ciphertext
        .as_deref()
        .ok_or_else(|| "Missing encrypted text content.".to_string())?;
    let nonce = row_data
        .text_nonce
        .as_deref()
        .ok_or_else(|| "Missing text nonce.".to_string())?;

    let content =
        decrypt_text(ciphertext, nonce, key).map_err(|e| format!("Failed to decrypt text: {e}"))?;

    Ok(ClipboardEntry::Text {
        id: row_data.id,
        timestamp: row_data.timestamp as u64,
        content,
        kind: row_data.text_kind.clone(),
    })
}

fn parse_image_entry(
    row_data: &RowData,
    base_dir: Option<&Path>,
) -> Result<ClipboardEntry, String> {
    let hash = row_data
        .image_hash
        .ok_or_else(|| "Missing image hash.".to_string())? as u64;

    let data_url = row_data
        .image_png
        .as_ref()
        .map(|bytes| data_url_from_png(bytes.clone()))
        .or_else(|| load_image_data_url_from_path(base_dir, row_data.image_path.as_deref()));

    Ok(ClipboardEntry::Image {
        id: row_data.id,
        timestamp: row_data.timestamp as u64,
        path: row_data.image_path.clone(),
        hash,
        data_url,
    })
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
