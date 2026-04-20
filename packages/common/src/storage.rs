use crate::paths::{resolve_app_paths, AppPaths};
use std::path::PathBuf;

/// StorageConfig: Information about where and how data is stored
#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub paths: AppPaths,
    pub db_path: PathBuf,
}

impl StorageConfig {
    /// Create a new StorageConfig with canonical paths
    pub fn new() -> std::io::Result<Self> {
        let paths = resolve_app_paths()?;
        let db_path = paths.db_path.clone();
        Ok(StorageConfig { paths, db_path })
    }

    /// Create a StorageConfig with a custom database path
    pub fn with_db_path(db_path: PathBuf) -> std::io::Result<Self> {
        let paths = resolve_app_paths()?;
        Ok(StorageConfig { paths, db_path })
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        StorageConfig::new().expect("Failed to initialize StorageConfig")
    }
}

/// Database schema SQL
/// Use this in your app's history module to initialize the database:
/// ```ignore
/// conn.execute_batch(CREATE_ENTRIES_TABLE)?;
/// ```
pub const CREATE_ENTRIES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at INTEGER NOT NULL,
    entry_type TEXT NOT NULL CHECK (entry_type IN ('text', 'image')),
    text_kind TEXT,
    text_ciphertext BLOB,
    text_nonce BLOB,
    image_path TEXT,
    image_png BLOB,
    image_hash INTEGER,
    content_hash TEXT
)
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_config_with_custom_path() {
        let custom_path = PathBuf::from("/tmp/test.db");
        let config = StorageConfig::with_db_path(custom_path.clone()).unwrap();
        assert_eq!(config.db_path, custom_path);
    }

    #[test]
    fn test_storage_config_has_paths() {
        let config = StorageConfig::with_db_path(PathBuf::from("/tmp/test.db")).unwrap();
        assert!(!config.paths.base_dir.as_os_str().is_empty());
    }

    #[test]
    fn test_create_entries_table_contains_schema() {
        assert!(CREATE_ENTRIES_TABLE.contains("CREATE TABLE"));
        assert!(CREATE_ENTRIES_TABLE.contains("entries"));
        assert!(CREATE_ENTRIES_TABLE.contains("id INTEGER PRIMARY KEY"));
        assert!(CREATE_ENTRIES_TABLE.contains("created_at INTEGER"));
        assert!(CREATE_ENTRIES_TABLE.contains("entry_type TEXT"));
        assert!(CREATE_ENTRIES_TABLE.contains("CHECK (entry_type IN ('text', 'image'))"));
    }

    #[test]
    fn test_create_entries_table_has_all_columns() {
        assert!(CREATE_ENTRIES_TABLE.contains("text_kind"));
        assert!(CREATE_ENTRIES_TABLE.contains("text_ciphertext"));
        assert!(CREATE_ENTRIES_TABLE.contains("text_nonce"));
        assert!(CREATE_ENTRIES_TABLE.contains("image_path"));
        assert!(CREATE_ENTRIES_TABLE.contains("image_png"));
        assert!(CREATE_ENTRIES_TABLE.contains("image_hash"));
        assert!(CREATE_ENTRIES_TABLE.contains("content_hash"));
    }
}
