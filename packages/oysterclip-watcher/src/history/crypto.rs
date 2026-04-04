use base64::{engine::general_purpose, Engine as _};
use chacha20poly1305::aead::{Aead, AeadCore, KeyInit};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use keyring::Entry;
use rand::rngs::OsRng;
use rand::RngCore;
use std::io;

use crate::config::constants::{KEYRING_ACCOUNT, KEYRING_SERVICE};

pub(crate) struct EncryptedText {
    pub(crate) ciphertext: Vec<u8>,
    pub(crate) nonce: Vec<u8>,
}

pub(crate) fn load_or_create_encryption_key() -> io::Result<[u8; 32]> {
    let entry = Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)
        .map_err(|err| io::Error::other(format!("failed to access OS keychain entry: {err}")))?;

    match entry.get_password() {
        Ok(encoded) => decode_encryption_key(&encoded),
        Err(keyring::Error::NoEntry) => {
            let mut key = [0u8; 32];
            OsRng.fill_bytes(&mut key);
            entry
                .set_password(&general_purpose::STANDARD.encode(key))
                .map_err(|err| {
                    io::Error::other(format!(
                        "failed to save encryption key to OS keychain: {err}"
                    ))
                })?;
            Ok(key)
        }
        Err(err) => Err(io::Error::other(format!(
            "failed to read encryption key from OS keychain: {err}"
        ))),
    }
}

fn decode_encryption_key(encoded: &str) -> io::Result<[u8; 32]> {
    let decoded = general_purpose::STANDARD.decode(encoded).map_err(|err| {
        io::Error::other(format!("failed to decode keychain encryption key: {err}"))
    })?;

    if decoded.len() != 32 {
        return Err(io::Error::other(format!(
            "invalid key length in keychain: expected 32 bytes, got {}",
            decoded.len()
        )));
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&decoded);
    Ok(key)
}

pub(crate) fn encrypt_text(content: &str, key: &[u8; 32]) -> io::Result<EncryptedText> {
    let cipher = XChaCha20Poly1305::new(key.into());
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, content.as_bytes())
        .map_err(|err| io::Error::other(format!("failed to encrypt clipboard text: {err}")))?;

    Ok(EncryptedText {
        ciphertext,
        nonce: nonce.to_vec(),
    })
}

#[allow(dead_code)]
pub(crate) fn decrypt_text(ciphertext: &[u8], nonce: &[u8], key: &[u8; 32]) -> io::Result<String> {
    let cipher = XChaCha20Poly1305::new(key.into());
    let nonce = XNonce::from_slice(nonce);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|err| io::Error::other(format!("failed to decrypt clipboard text: {err}")))?;
    String::from_utf8(plaintext).map_err(|err| {
        io::Error::other(format!("failed to decode decrypted clipboard text: {err}"))
    })
}

pub(crate) fn text_content_hash(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub(crate) fn current_timestamp() -> u64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
