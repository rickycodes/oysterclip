pub(crate) const INTERVAL_MS: u64 = 500;
pub(crate) const MAX_HISTORY_ENTRIES: usize = 500;
pub(crate) const CONTROL_SOCKET_FILE: &str = ".clipboard-watcher.sock";
pub(crate) const CLIPBOARD_NOT_AVAILABLE: &str = "Clipboard not available";
pub(crate) const FAILED_IMAGE_BUFFER: &str = "Failed to create image buffer";
pub(crate) const TEXT_KIND_EMPTY: &str = "empty";
pub(crate) const TEXT_KIND_IMAGE_DATA_URI: &str = "image-data-uri";
pub(crate) const TEXT_KIND_URL: &str = "url";
pub(crate) const TEXT_KIND_JSON: &str = "json";
pub(crate) const TEXT_KIND_MULTILINE: &str = "multiline";
pub(crate) const TEXT_KIND_PATH: &str = "path";
pub(crate) const TEXT_KIND_PLAIN: &str = "plain";
pub(crate) const STARTUP_MESSAGE: &str = "Starting clipboard watcher";
pub(crate) const TEXT_EMPTY_SKIPPED: &str = "(text:empty) skipped";
pub(crate) const TEXT_IMAGE_DATA_URI_SKIPPED: &str = "(text:image-data-uri) skipped";
pub(crate) const TEXT_CAPTURED: &str = "captured";
pub(crate) const APPEND_TEXT_HISTORY_FAILED: &str = "Failed to append text history";
pub(crate) const IMAGE_SAVED: &str = "(image) saved";
pub(crate) const APPEND_IMAGE_HISTORY_FAILED: &str = "Failed to append image history";

// Re-export database queries from common
pub(crate) use common::{
    CREATE_ENTRIES_TABLE as CREATE_ENTRIES_TABLE_SQL, CREATE_INDICES,
    DELETE_PRUNABLE_ENTRIES as DELETE_PRUNABLE_ENTRIES_SQL,
    INSERT_IMAGE_ENTRY as INSERT_IMAGE_ENTRY_SQL, INSERT_TEXT_ENTRY as INSERT_TEXT_ENTRY_SQL,
    SELECT_EXISTING_TEXT_ENTRY as SELECT_EXISTING_TEXT_ENTRY_SQL,
};
