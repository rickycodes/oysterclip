use std::io;

// Re-export common crypto functions
#[allow(unused_imports)]
pub(crate) use common::crypto::{decrypt_text, encrypt_text, get_or_create_key, text_content_hash};

pub(crate) fn load_or_create_encryption_key() -> io::Result<[u8; 32]> {
    get_or_create_key()
}

pub(crate) fn current_timestamp() -> u64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
