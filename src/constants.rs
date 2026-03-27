pub(crate) const INTERVAL_MS: u64 = 500;
pub(crate) const MAX_HISTORY_ENTRIES: usize = 500;
pub(crate) const HISTORY_FILE: &str = ".clipboard_history.db";
pub(crate) const CONFIG_FILE: &str = ".clipboard-watcher.toml";
pub(crate) const CONTROL_SOCKET_FILE: &str = ".clipboard-watcher.sock";
pub(crate) const CLIPBOARD_NOT_AVAILABLE: &str = "Clipboard not available";
pub(crate) const FAILED_IMAGE_BUFFER: &str = "Failed to create image buffer";
pub(crate) const KEYRING_SERVICE: &str = "clipboard-manager";
pub(crate) const KEYRING_ACCOUNT: &str = "default-encryption-key";
pub(crate) const TEXT_KIND_EMPTY: &str = "empty";
pub(crate) const TEXT_KIND_IMAGE_DATA_URI: &str = "image-data-uri";
pub(crate) const TEXT_KIND_URL: &str = "url";
pub(crate) const TEXT_KIND_JSON: &str = "json";
pub(crate) const TEXT_KIND_MULTILINE: &str = "multiline";
pub(crate) const TEXT_KIND_PATH: &str = "path";
pub(crate) const TEXT_KIND_PLAIN: &str = "plain";
pub(crate) const STARTUP_MESSAGE: &str = "Starting clipboard watcher";
pub(crate) const OPEN_HISTORY_STORE_FAILED: &str = "Failed to open history store";
pub(crate) const TEXT_EMPTY_SKIPPED: &str = "(text:empty) skipped";
pub(crate) const TEXT_IMAGE_DATA_URI_SKIPPED: &str = "(text:image-data-uri) skipped";
pub(crate) const TEXT_CAPTURED: &str = "captured";
pub(crate) const APPEND_TEXT_HISTORY_FAILED: &str = "Failed to append text history";
pub(crate) const IMAGE_SAVED: &str = "(image) saved";
pub(crate) const APPEND_IMAGE_HISTORY_FAILED: &str = "Failed to append image history";
pub(crate) const CREATE_ENTRIES_TABLE_SQL: &str = "\
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
);
CREATE INDEX IF NOT EXISTS idx_entries_created_at ON entries(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_entries_content_hash ON entries(content_hash);
";
pub(crate) const SELECT_EXISTING_TEXT_ENTRY_SQL: &str =
    "SELECT id FROM entries WHERE entry_type = 'text' AND content_hash = ?1 LIMIT 1";
pub(crate) const INSERT_TEXT_ENTRY_SQL: &str = "\
INSERT INTO entries (
    created_at,
    entry_type,
    text_kind,
    text_ciphertext,
    text_nonce,
    content_hash
) VALUES (?1, 'text', ?2, ?3, ?4, ?5)";
pub(crate) const INSERT_IMAGE_ENTRY_SQL: &str = "\
INSERT INTO entries (
    created_at,
    entry_type,
    image_path,
    image_png,
    image_hash
) VALUES (?1, 'image', ?2, ?3, ?4)";
pub(crate) const DELETE_PRUNABLE_ENTRIES_SQL: &str = "\
DELETE FROM entries
WHERE id IN (
    SELECT id FROM entries
    ORDER BY created_at DESC, id DESC
    LIMIT -1 OFFSET ?1
)";
