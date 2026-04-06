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
    let entry = Entry::new(APP_NAME, KEYRING_ACCOUNT)
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
