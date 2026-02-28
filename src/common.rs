use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

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
#[serde(tag = "type")]
pub enum ClipboardEntry {
    Text {
        #[serde(deserialize_with = "deserialize_timestamp")]
        timestamp: u64,
        content: String,
    },
    Image {
        #[serde(deserialize_with = "deserialize_timestamp")]
        timestamp: u64,
        path: String,
        #[serde(deserialize_with = "deserialize_u64")]
        hash: u64,
        data_url: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct ClipboardPayload {
    pub entries: Vec<ClipboardEntry>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum FileEntry {
    Text {
        #[serde(deserialize_with = "deserialize_timestamp")]
        timestamp: u64,
        content: String,
    },
    Image {
        #[serde(deserialize_with = "deserialize_timestamp")]
        timestamp: u64,
        path: String,
        #[serde(deserialize_with = "deserialize_u64")]
        hash: u64,
    },
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

    let (data, base_dir) = match &source.kind {
        SourceKind::File(path) => {
            let data =
                fs::read_to_string(path).map_err(|e| format!("Failed to read history file: {e}"))?;
            let base_dir = path.parent().map(|p| p.to_path_buf());
            (data, base_dir)
        }
        SourceKind::RawJson(json) => (json.clone(), env::current_dir().ok()),
        SourceKind::Empty => {
            return Err("Missing clipboard history argument.".to_string());
        }
    };

    let entries: Vec<FileEntry> =
        serde_json::from_str(&data).map_err(|e| format!("Invalid history JSON: {e}"))?;

    let view_entries = entries
        .into_iter()
        .map(|entry| match entry {
            FileEntry::Text { timestamp, content } => ClipboardEntry::Text { timestamp, content },
            FileEntry::Image {
                timestamp,
                path,
                hash,
            } => {
                let resolved = resolve_image_path(base_dir.as_deref(), &path);
                let data_url = resolved.and_then(|resolved_path| {
                    fs::read(resolved_path).ok().map(|bytes| {
                        format!(
                            "data:image/png;base64,{}",
                            general_purpose::STANDARD.encode(bytes)
                        )
                    })
                });

                ClipboardEntry::Image {
                    timestamp,
                    path,
                    hash,
                    data_url,
                }
            }
        })
        .collect();

    Ok(view_entries)
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

fn deserialize_timestamp<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserialize_u64(deserializer)
}

fn deserialize_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Number(num) => {
            if let Some(u) = num.as_u64() {
                Ok(u)
            } else if let Some(f) = num.as_f64() {
                Ok(f.round() as u64)
            } else {
                Err(D::Error::custom("invalid number for u64"))
            }
        }
        serde_json::Value::String(s) => s
            .parse::<u64>()
            .map_err(|_| D::Error::custom("invalid string for u64")),
        _ => Err(D::Error::custom("invalid type for u64")),
    }
}
