pub const HISTORY_FILE: &str = ".clipboard_history.db";
pub const CONFIG_FILE: &str = "config.toml";
pub const SOCKET_FILE: &str = ".clipboard-watcher.sock";
pub const APP_NAME: &str = "oysterclip";
pub const KEYRING_ACCOUNT: &str = "default-encryption-key";
pub const ENCRYPTION_KEY_ID: &str = "oysterclip-encryption-key";

// Database schema
pub const DB_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS entries (
    id INTEGER PRIMARY KEY,
    created_at INTEGER NOT NULL,
    entry_type TEXT NOT NULL,
    text_kind TEXT,
    text_ciphertext BLOB,
    text_nonce BLOB,
    image_path TEXT,
    image_png BLOB,
    image_hash TEXT,
    content_hash TEXT NOT NULL UNIQUE
)
"#;

pub const DB_INIT_STATEMENTS: &[&str] = &[
    DB_SCHEMA,
    "CREATE INDEX IF NOT EXISTS idx_created_at ON entries(created_at DESC)",
    "CREATE INDEX IF NOT EXISTS idx_content_hash ON entries(content_hash)",
];
