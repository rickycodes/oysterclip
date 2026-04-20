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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_timestamp_is_positive() {
        let ts = current_timestamp();
        assert!(ts > 0);
    }

    #[test]
    fn test_current_timestamp_increases() {
        let ts1 = current_timestamp();
        let ts2 = current_timestamp();
        assert!(ts2 >= ts1);
    }

    #[test]
    fn test_current_timestamp_reasonable_range() {
        let ts = current_timestamp();
        // Should be after year 2000 (946684800) and before year 2050
        assert!(ts > 946684800);
        assert!(ts < 2524608000);
    }
}
