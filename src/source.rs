use std::env;
use std::path::{Path, PathBuf};

use crate::paths::default_history_path;

const HISTORY_PATH_ENV: &str = "CLIPBOARD_HISTORY_DB";

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
        if let Some(arg) = &crate::cli::args().db {
            return Self::from_arg(arg.clone());
        }

        match default_history_path_from_env() {
            Ok(path) => Self {
                kind: SourceKind::File(path),
                error: None,
            },
            Err(err) => Self {
                kind: SourceKind::Empty,
                error: Some(err),
            },
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
            SourceKind::Empty => Err("Clipboard history source is not available.".to_string()),
        }
    }

    pub fn raw_json(&self) -> Option<&str> {
        match &self.kind {
            SourceKind::RawJson(json) => Some(json),
            _ => None,
        }
    }

    fn from_arg(arg: String) -> Self {
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

fn default_history_path_from_env() -> Result<PathBuf, String> {
    if let Some(path) = env::var_os(HISTORY_PATH_ENV)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
    {
        return Ok(path);
    }

    default_history_path().map_err(|err| {
        format!(
            "Failed to resolve default clipboard history path: {err}. Pass a database path with --db or set {HISTORY_PATH_ENV}."
        )
    })
}

#[cfg(test)]
mod tests {
    use super::ClipboardSource;
    use crate::paths::default_history_path;
    use std::path::PathBuf;

    #[test]
    fn cli_json_input_is_treated_as_raw_json() {
        let source = ClipboardSource::from_arg("[{}]".to_string());
        assert_eq!(source.raw_json(), Some("[{}]"));
    }

    #[test]
    fn cli_path_input_is_treated_as_file() {
        let source = ClipboardSource::from_arg("/tmp/history.db".to_string());
        assert_eq!(source.file_path().unwrap(), PathBuf::from("/tmp/history.db"));
    }

    #[test]
    fn canonical_default_history_path_has_expected_filename() {
        let path = default_history_path().unwrap();
        assert_eq!(path.file_name().unwrap(), ".clipboard_history.db");
    }
}
