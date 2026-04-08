pub mod auth;
pub mod classification;
pub mod clipboard;
pub mod constants;
pub mod crypto;
pub mod entry;
pub mod ipc;
pub mod paths;
pub mod storage;

pub use auth::{authenticate_admin_action, AuthCache, AuthResult};
pub use classification::{is_password, mask_password_preview};
pub use clipboard::copy_to_clipboard;
pub use constants::{
    APP_NAME, APP_ORGANIZATION, APP_QUALIFIER, AUTH_FAILED, AUTH_SUCCESS, CONFIG_FILE,
    CREATE_ENTRIES_TABLE, CREATE_INDICES, DELETE_PRUNABLE_ENTRIES, ENTRY_TYPE_IMAGE,
    ENTRY_TYPE_TEXT, ERR_OPEN_HISTORY_DB, ERR_RESOLVE_APP_DIR, HISTORY_FILE, IMAGE_DIR,
    INSERT_IMAGE_ENTRY, INSERT_TEXT_ENTRY, KEYRING_ACCOUNT, ORDER_ENTRIES,
    SELECT_EXISTING_TEXT_ENTRY, SOCKET_FILE, TEMP_BULK_FILE, TEXT_KIND_EMPTY,
    TEXT_KIND_IMAGE_DATA_URI, TEXT_KIND_JSON, TEXT_KIND_MULTILINE, TEXT_KIND_PATH, TEXT_KIND_PLAIN,
    TEXT_KIND_URL, THEME_DARK, THEME_LIGHT, UI_REFRESH_INTERVAL_MS,
};
pub use crypto::{decrypt_text, encrypt_text, get_or_create_key, text_content_hash, EncryptedData};
pub use entry::{CommonEntry, EntryType, StorageEntry};
pub use ipc::{
    socket_path, ControlCommand, ControlRequest, ControlResponse, MSG_WATCHER_PAUSED,
    MSG_WATCHER_RESUMED,
};
pub use paths::{ensure_app_dir, resolve_app_paths, AppPaths};
pub use storage::StorageConfig;
