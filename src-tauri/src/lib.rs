use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone)]
struct ClipboardSource {
    kind: SourceKind,
    error: Option<String>,
}

#[derive(Clone)]
enum SourceKind {
    File(PathBuf),
    RawJson(String),
    Empty,
}

impl ClipboardSource {
    fn from_env() -> Self {
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

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Clone)]
#[serde(tag = "type")]
enum ViewEntry {
    Text { timestamp: u64, content: String },
    Image {
        timestamp: u64,
        path: String,
        hash: u64,
        data_url: Option<String>,
    },
}

#[derive(Serialize, Clone)]
struct ClipboardPayload {
    entries: Vec<ViewEntry>,
    error: Option<String>,
}

fn resolve_image_path(base_dir: Option<&Path>, path_str: &str) -> Option<PathBuf> {
    let path = Path::new(path_str);
    if path.is_absolute() {
        return Some(path.to_path_buf());
    }
    base_dir.map(|base| base.join(path))
}

fn load_entries(source: &ClipboardSource) -> Result<Vec<ViewEntry>, String> {
    if let Some(err) = source.error.as_ref() {
        return Err(err.clone());
    }

    let (data, base_dir) = match &source.kind {
        SourceKind::File(path) => {
            let data = fs::read_to_string(path)
                .map_err(|e| format!("Failed to read history file: {e}"))?;
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
            FileEntry::Text { timestamp, content } => ViewEntry::Text { timestamp, content },
            FileEntry::Image {
                timestamp,
                path,
                hash,
            } => {
                let resolved = resolve_image_path(base_dir.as_deref(), &path);
                let data_url = resolved.and_then(|resolved_path| {
                    fs::read(resolved_path)
                        .ok()
                        .map(|bytes| format!("data:image/png;base64,{}", general_purpose::STANDARD.encode(bytes)))
                });

                ViewEntry::Image {
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

#[tauri::command]
fn get_clipboard_entries(state: tauri::State<ClipboardSource>) -> ClipboardPayload {
    match load_entries(&state) {
        Ok(entries) => ClipboardPayload {
            entries,
            error: None,
        },
        Err(err) => ClipboardPayload {
            entries: Vec::new(),
            error: Some(err),
        },
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let source = ClipboardSource::from_env();

    tauri::Builder::default()
        .manage(source)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_clipboard_entries])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
