use once_cell::sync::Lazy;

pub const APP_NAME: &str = "oysterclip";
pub const APP_QUALIFIER: &str = "com";
pub const APP_ORGANIZATION: &str = "rickycodes";

pub static HISTORY_FILE: Lazy<String> = Lazy::new(|| format!(".{}.db", APP_NAME));
pub static CONFIG_FILE: Lazy<String> = Lazy::new(|| format!(".{}.toml", APP_NAME));
pub static SOCKET_FILE: Lazy<String> = Lazy::new(|| format!(".{}.sock", APP_NAME));
pub const IMAGE_DIR: &str = "clipboard_images";
pub const TEMP_BULK_FILE: &str = "clipboard_bulk_temp.txt";
pub static KEYRING_ACCOUNT: Lazy<String> = Lazy::new(|| format!("{}-encryption-key", APP_NAME));

// Theme mode constants
pub const THEME_LIGHT: &str = "light";
pub const THEME_DARK: &str = "dark";

// Authentication messages
pub const AUTH_SUCCESS: &str = "Authentication successful";
pub const AUTH_FAILED: &str = "Authentication failed or canceled";

// UI polling interval (milliseconds)
pub const UI_REFRESH_INTERVAL_MS: u64 = 500;

// Path resolution error messages
pub const ERR_RESOLVE_APP_DIR: &str = "failed to resolve application data directory";

// Database error messages
pub const ERR_OPEN_HISTORY_DB: &str = "Failed to open history database";

// Entry type constants
pub const ENTRY_TYPE_TEXT: &str = "text";
pub const ENTRY_TYPE_IMAGE: &str = "image";

// Text kind classification constants
pub const TEXT_KIND_EMPTY: &str = "empty";
pub const TEXT_KIND_IMAGE_DATA_URI: &str = "image-data-uri";
pub const TEXT_KIND_URL: &str = "url";
pub const TEXT_KIND_JSON: &str = "json";
pub const TEXT_KIND_MULTILINE: &str = "multiline";
pub const TEXT_KIND_PATH: &str = "path";
pub const TEXT_KIND_PLAIN: &str = "plain";

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

// Entries are ordered by ID descending (newest first) for both viewer and TUI
pub const ORDER_ENTRIES: &str = "ORDER BY id DESC";

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
