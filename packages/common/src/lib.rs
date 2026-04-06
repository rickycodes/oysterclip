pub mod constants;
pub mod crypto;
pub mod entry;
pub mod ipc;
pub mod paths;
pub mod storage;

pub use constants::{
    APP_NAME, APP_ORGANIZATION, APP_QUALIFIER, CONFIG_FILE, CREATE_ENTRIES_TABLE, CREATE_INDICES,
    DELETE_PRUNABLE_ENTRIES, HISTORY_FILE, INSERT_IMAGE_ENTRY, INSERT_TEXT_ENTRY, KEYRING_ACCOUNT,
    SELECT_EXISTING_TEXT_ENTRY, SOCKET_FILE,
};
pub use crypto::{decrypt_text, encrypt_text, get_or_create_key, text_content_hash, EncryptedData};
pub use entry::{CommonEntry, EntryType, StorageEntry};
pub use ipc::{socket_path, ControlCommand, ControlRequest, ControlResponse};
pub use paths::{ensure_app_dir, resolve_app_paths, AppPaths};
pub use storage::StorageConfig;
