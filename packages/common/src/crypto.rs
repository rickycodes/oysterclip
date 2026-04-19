use base64::engine::general_purpose;
use base64::Engine as _;
use chacha20poly1305::aead::{Aead, AeadCore, KeyInit};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use keyring::Entry;
use rand::RngCore;
use std::io;

use super::constants::{APP_NAME, KEYRING_ACCOUNT};

pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

/// Get or create the encryption key from the OS keyring.
pub fn get_or_create_key() -> io::Result<[u8; 32]> {
    let entry = Entry::new(APP_NAME, KEYRING_ACCOUNT.as_str())
        .map_err(|e| io::Error::other(format!("Failed to access OS keychain entry: {e}")))?;

    match entry.get_password() {
        Ok(encoded) => decode_key(&encoded),
        Err(keyring::Error::NoEntry) => {
            let mut key = [0u8; 32];
            rand::thread_rng().fill_bytes(&mut key);
            entry
                .set_password(&general_purpose::STANDARD.encode(key))
                .map_err(|e| {
                    io::Error::other(format!("Failed to save encryption key to OS keychain: {e}"))
                })?;
            Ok(key)
        }
        Err(err) => Err(io::Error::other(format!(
            "Failed to read encryption key from OS keychain: {err}"
        ))),
    }
}

fn decode_key(encoded: &str) -> io::Result<[u8; 32]> {
    let decoded = general_purpose::STANDARD
        .decode(encoded)
        .map_err(|e| io::Error::other(format!("Failed to decode keychain encryption key: {e}")))?;

    if decoded.len() != 32 {
        return Err(io::Error::other(format!(
            "Invalid key length in keychain: expected 32 bytes, got {}",
            decoded.len()
        )));
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&decoded);
    Ok(key)
}

/// Encrypt plaintext using XChaCha20Poly1305 with a random nonce.
pub fn encrypt_text(plaintext: &str, key: &[u8; 32]) -> io::Result<EncryptedData> {
    let cipher = XChaCha20Poly1305::new(key.into());
    let nonce = XChaCha20Poly1305::generate_nonce(&mut rand::rngs::OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|e| io::Error::other(format!("Failed to encrypt clipboard text: {e}")))?;

    Ok(EncryptedData {
        ciphertext,
        nonce: nonce.to_vec(),
    })
}

/// Decrypt ciphertext using XChaCha20Poly1305.
pub fn decrypt_text(ciphertext: &[u8], nonce: &[u8], key: &[u8; 32]) -> io::Result<String> {
    if nonce.len() != 24 {
        return Err(io::Error::other(format!(
            "Invalid text nonce length: expected 24 bytes, got {}",
            nonce.len()
        )));
    }

    let cipher = XChaCha20Poly1305::new(key.into());
    let nonce_obj = XNonce::from_slice(nonce);
    let plaintext = cipher
        .decrypt(nonce_obj, ciphertext)
        .map_err(|e| io::Error::other(format!("Failed to decrypt clipboard text: {e}")))?;

    String::from_utf8(plaintext)
        .map_err(|e| io::Error::other(format!("Failed to decode decrypted clipboard text: {e}")))
}

/// Generate a hash of text content for deduplication.
pub fn text_content_hash(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; 32] {
        [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ]
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let plaintext = "Hello, World!";
        let key = test_key();

        let encrypted = encrypt_text(plaintext, &key).unwrap();
        let decrypted = decrypt_text(&encrypted.ciphertext, &encrypted.nonce, &key).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_encrypt_decrypt_empty_string() {
        let plaintext = "";
        let key = test_key();

        let encrypted = encrypt_text(plaintext, &key).unwrap();
        let decrypted = decrypt_text(&encrypted.ciphertext, &encrypted.nonce, &key).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_encrypt_decrypt_unicode() {
        let plaintext = "Hello 世界 🌍 ñoño";
        let key = test_key();

        let encrypted = encrypt_text(plaintext, &key).unwrap();
        let decrypted = decrypt_text(&encrypted.ciphertext, &encrypted.nonce, &key).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_encrypt_decrypt_large_text() {
        let plaintext = "x".repeat(100_000);
        let key = test_key();

        let encrypted = encrypt_text(&plaintext, &key).unwrap();
        let decrypted = decrypt_text(&encrypted.ciphertext, &encrypted.nonce, &key).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_decrypt_with_wrong_key_fails() {
        let plaintext = "Secret message";
        let key1 = test_key();
        let mut key2 = test_key();
        key2[0] ^= 0xFF; // Flip bits in first byte

        let encrypted = encrypt_text(plaintext, &key1).unwrap();
        let result = decrypt_text(&encrypted.ciphertext, &encrypted.nonce, &key2);

        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_with_tampered_ciphertext_fails() {
        let plaintext = "Original message";
        let key = test_key();

        let mut encrypted = encrypt_text(plaintext, &key).unwrap();
        // Tamper with ciphertext
        encrypted.ciphertext[0] ^= 0xFF;

        let result = decrypt_text(&encrypted.ciphertext, &encrypted.nonce, &key);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_with_invalid_nonce_length() {
        let key = test_key();
        let ciphertext = vec![1, 2, 3];
        let invalid_nonce = vec![1, 2, 3]; // Too short, should be 24

        let result = decrypt_text(&ciphertext, &invalid_nonce, &key);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid text nonce length"));
    }

    #[test]
    fn test_text_content_hash_deterministic() {
        let content = "Same content";
        let hash1 = text_content_hash(content);
        let hash2 = text_content_hash(content);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_text_content_hash_different_content() {
        let content1 = "Content A";
        let content2 = "Content B";

        let hash1 = text_content_hash(content1);
        let hash2 = text_content_hash(content2);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_text_content_hash_empty_string() {
        let hash = text_content_hash("");
        assert!(!hash.is_empty());
    }

    #[test]
    fn test_encrypted_data_has_nonce() {
        let plaintext = "Test";
        let key = test_key();

        let encrypted = encrypt_text(plaintext, &key).unwrap();

        assert_eq!(encrypted.nonce.len(), 24);
        assert!(!encrypted.ciphertext.is_empty());
    }

    #[test]
    fn test_different_encryptions_have_different_nonces() {
        let plaintext = "Same plaintext";
        let key = test_key();

        let encrypted1 = encrypt_text(plaintext, &key).unwrap();
        let encrypted2 = encrypt_text(plaintext, &key).unwrap();

        // Different random nonces (extremely unlikely to be equal)
        assert_ne!(encrypted1.nonce, encrypted2.nonce);
        // But both decrypt to same plaintext
        let decrypted1 = decrypt_text(&encrypted1.ciphertext, &encrypted1.nonce, &key).unwrap();
        let decrypted2 = decrypt_text(&encrypted2.ciphertext, &encrypted2.nonce, &key).unwrap();
        assert_eq!(decrypted1, decrypted2);
    }

    #[test]
    fn test_decode_key_valid() {
        let original_key = test_key();
        let encoded = general_purpose::STANDARD.encode(original_key);
        let decoded = decode_key(&encoded).unwrap();

        assert_eq!(original_key, decoded);
    }

    #[test]
    fn test_decode_key_invalid_base64() {
        let result = decode_key("not valid base64 !!!!");
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_key_wrong_length() {
        let encoded = general_purpose::STANDARD.encode([1, 2, 3]);
        let result = decode_key(&encoded);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid key length"));
    }
}
