use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use keyring::Entry;
use rand::RngCore;
use std::io;
use base64::engine::general_purpose;
use base64::Engine as _;

use super::constants::ENCRYPTION_KEY_ID;

pub struct Crypto;

impl Crypto {
    pub fn get_or_create_key() -> io::Result<[u8; 32]> {
        let entry = Entry::new(ENCRYPTION_KEY_ID, ENCRYPTION_KEY_ID)
            .map_err(|e| io::Error::other(e.to_string()))?;

        match entry.get_password() {
            Ok(password) => {
                let key_bytes = general_purpose::STANDARD
                    .decode(&password)
                    .map_err(|e| io::Error::other(e.to_string()))?;
                if key_bytes.len() != 32 {
                    return Err(io::Error::other(
                        "Invalid key length".to_string(),
                    ));
                }
                let mut key = [0u8; 32];
                key.copy_from_slice(&key_bytes);
                Ok(key)
            }
            Err(_) => {
                let mut key = [0u8; 32];
                rand::thread_rng().fill_bytes(&mut key);
                let encoded = general_purpose::STANDARD.encode(key);
                entry
                    .set_password(&encoded)
                    .map_err(|e| io::Error::other(e.to_string()))?;
                Ok(key)
            }
        }
    }

    pub fn encrypt(plaintext: &str, key: &[u8; 32]) -> io::Result<(Vec<u8>, [u8; 20])> {
        let cipher = ChaCha20Poly1305::new_from_slice(key)
            .map_err(|e| io::Error::other(e.to_string()))?;

        let mut nonce_bytes = [0u8; 20];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);

        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| io::Error::other(e.to_string()))?;

        Ok((ciphertext, nonce_bytes))
    }

    pub fn decrypt(ciphertext: &[u8], nonce: &[u8; 20], key: &[u8; 32]) -> io::Result<String> {
        let cipher = ChaCha20Poly1305::new_from_slice(key)
            .map_err(|e| io::Error::other(e.to_string()))?;

        let nonce_obj = Nonce::from_slice(nonce);
        let plaintext = cipher
            .decrypt(nonce_obj, ciphertext)
            .map_err(|e| io::Error::other(e.to_string()))?;

        String::from_utf8(plaintext)
            .map_err(|e| io::Error::other(e.to_string()))
    }
}
