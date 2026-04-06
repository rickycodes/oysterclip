pub const HISTORY_FILE: &str = ".clipboard_history.db";
pub const CONFIG_FILE: &str = "config.toml";
pub const SOCKET_FILE: &str = ".clipboard-watcher.sock";
pub const APP_NAME: &str = "oysterclip";
pub const APP_QUALIFIER: &str = "com";
pub const APP_ORGANIZATION: &str = "rickycodes";
pub const KEYRING_ACCOUNT: &str = "oysterclip-encryption-key";

// Database schema and queries
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
    content_hash TEXT UNIQUE
)"#;

pub const CREATE_INDICES: &[&str] = &[
    "CREATE INDEX IF NOT EXISTS idx_entries_created_at ON entries(created_at DESC)",
    "CREATE INDEX IF NOT EXISTS idx_entries_content_hash ON entries(content_hash)",
];

pub const SELECT_EXISTING_TEXT_ENTRY: &str =
    "SELECT id FROM entries WHERE entry_type = 'text' AND content_hash = ?1 LIMIT 1";

pub const INSERT_TEXT_ENTRY: &str = r#"
INSERT INTO entries (
    created_at,
    entry_type,
    text_kind,
    text_ciphertext,
    text_nonce,
    content_hash
) VALUES (?1, 'text', ?2, ?3, ?4, ?5)"#;

pub const INSERT_IMAGE_ENTRY: &str = r#"
INSERT INTO entries (
    created_at,
    entry_type,
    image_path,
    image_png,
    image_hash
) VALUES (?1, 'image', ?2, ?3, ?4)"#;

pub const DELETE_PRUNABLE_ENTRIES: &str = r#"
DELETE FROM entries
WHERE id IN (
    SELECT id FROM entries
    ORDER BY created_at DESC, id DESC
    LIMIT -1 OFFSET ?1
)"#;
