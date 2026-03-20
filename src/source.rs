use std::env;
use std::path::{Path, PathBuf};

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

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn file_path(&self) -> Result<&Path, String> {
        if let Some(err) = self.error.as_ref() {
            return Err(err.clone());
        }

        match &self.kind {
            SourceKind::File(path) => Ok(path.as_path()),
            SourceKind::RawJson(_) => {
                Err("History mutations are not supported for raw JSON input.".to_string())
            }
            SourceKind::Empty => Err("Missing clipboard history argument.".to_string()),
        }
    }

    pub fn raw_json(&self) -> Option<&str> {
        match &self.kind {
            SourceKind::RawJson(json) => Some(json),
            _ => None,
        }
    }
}
