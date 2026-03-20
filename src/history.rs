use base64::{engine::general_purpose, Engine as _};
use chacha20poly1305::aead::{Aead, AeadCore, KeyInit};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use keyring::Entry;
use rand::rngs::OsRng;
use rand::RngCore;
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::constants::{
    CREATE_ENTRIES_TABLE_SQL, INSERT_IMAGE_ENTRY_SQL, INSERT_TEXT_ENTRY_SQL, KEYRING_ACCOUNT,
    KEYRING_SERVICE, SELECT_EXISTING_TEXT_ENTRY_SQL,
};
use crate::entry::PasteEntry;

pub(crate) struct HistoryStore {
    db_path: PathBuf,
    encryption_key: [u8; 32],
}

fn io_error(message: impl Into<String>) -> io::Error {
    io::Error::other(message.into())
}

impl HistoryStore {
    pub(crate) fn open(db_path: &Path) -> io::Result<Self> {
        let store = Self {
            db_path: db_path.to_path_buf(),
            encryption_key: load_or_create_encryption_key()?,
        };
        let conn = store.connection()?;
        conn.execute_batch(CREATE_ENTRIES_TABLE_SQL)
            .map_err(|err| io_error(format!("failed to initialize history database: {err}")))?;
        Ok(store)
    }

    pub(crate) fn append_entry(&self, entry: &PasteEntry) -> io::Result<()> {
        let conn = self.connection()?;

        match entry {
            PasteEntry::Text {
                timestamp,
                content,
                kind,
            } => self.insert_text_entry(&conn, *timestamp, content, kind.as_deref()),
            PasteEntry::Image {
                timestamp,
                path,
                hash,
            } => self.insert_image_entry(&conn, *timestamp, path, *hash),
        }
    }

    fn connection(&self) -> io::Result<Connection> {
        Connection::open(&self.db_path)
            .map_err(|err| io_error(format!("failed to open history database: {err}")))
    }

    fn insert_text_entry(
        &self,
        conn: &Connection,
        timestamp: u64,
        content: &str,
        kind: Option<&str>,
    ) -> io::Result<()> {
        let content_hash = text_content_hash(content);
        let existing: Option<i64> = conn
            .query_row(
                SELECT_EXISTING_TEXT_ENTRY_SQL,
                params![content_hash],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| io_error(format!("failed to query existing text history: {err}")))?;

        if existing.is_some() {
            return Ok(());
        }

        let encrypted = encrypt_text(content, &self.encryption_key)?;
        conn.execute(
            INSERT_TEXT_ENTRY_SQL,
            params![
                timestamp as i64,
                kind,
                encrypted.ciphertext,
                encrypted.nonce,
                content_hash
            ],
        )
        .map_err(|err| io_error(format!("failed to insert text history entry: {err}")))?;

        Ok(())
    }

    fn insert_image_entry(
        &self,
        conn: &Connection,
        timestamp: u64,
        path: &str,
        hash: u64,
    ) -> io::Result<()> {
        conn.execute(
            INSERT_IMAGE_ENTRY_SQL,
            params![timestamp as i64, path, hash as i64],
        )
        .map_err(|err| io_error(format!("failed to insert image history entry: {err}")))?;

        Ok(())
    }
}

struct EncryptedText {
    ciphertext: Vec<u8>,
    nonce: Vec<u8>,
}

fn load_or_create_encryption_key() -> io::Result<[u8; 32]> {
    let entry = Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)
        .map_err(|err| io_error(format!("failed to access OS keychain entry: {err}")))?;

    match entry.get_password() {
        Ok(encoded) => decode_encryption_key(&encoded),
        Err(keyring::Error::NoEntry) => {
            let mut key = [0u8; 32];
            OsRng.fill_bytes(&mut key);
            entry
                .set_password(&general_purpose::STANDARD.encode(key))
                .map_err(|err| {
                    io_error(format!(
                        "failed to save encryption key to OS keychain: {err}"
                    ))
                })?;
            Ok(key)
        }
        Err(err) => Err(io_error(format!(
            "failed to read encryption key from OS keychain: {err}"
        ))),
    }
}

fn decode_encryption_key(encoded: &str) -> io::Result<[u8; 32]> {
    let decoded = general_purpose::STANDARD
        .decode(encoded)
        .map_err(|err| io_error(format!("failed to decode keychain encryption key: {err}")))?;

    if decoded.len() != 32 {
        return Err(io_error(format!(
            "invalid key length in keychain: expected 32 bytes, got {}",
            decoded.len()
        )));
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&decoded);
    Ok(key)
}

fn encrypt_text(content: &str, key: &[u8; 32]) -> io::Result<EncryptedText> {
    let cipher = XChaCha20Poly1305::new(key.into());
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, content.as_bytes())
        .map_err(|err| io_error(format!("failed to encrypt clipboard text: {err}")))?;

    Ok(EncryptedText {
        ciphertext,
        nonce: nonce.to_vec(),
    })
}

fn text_content_hash(content: &str) -> String {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

#[allow(dead_code)]
fn decrypt_text(ciphertext: &[u8], nonce: &[u8], key: &[u8; 32]) -> io::Result<String> {
    let cipher = XChaCha20Poly1305::new(key.into());
    let nonce = XNonce::from_slice(nonce);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|err| io_error(format!("failed to decrypt clipboard text: {err}")))?;
    String::from_utf8(plaintext)
        .map_err(|err| io_error(format!("failed to decode decrypted clipboard text: {err}")))
}

pub(crate) fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::{current_timestamp, decrypt_text, encrypt_text, text_content_hash};
    use std::time::SystemTime;

    #[test]
    fn current_timestamp_returns_unix_seconds_between_bounds() {
        let before = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let ts = current_timestamp();

        let after = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        assert!(ts >= before, "timestamp was earlier than before bound");
        assert!(ts <= after, "timestamp was later than after bound");
    }

    #[test]
    fn text_encryption_round_trips() {
        let key = [7u8; 32];
        let encrypted = encrypt_text("hello", &key).unwrap();
        let decrypted = decrypt_text(&encrypted.ciphertext, &encrypted.nonce, &key).unwrap();
        assert_eq!(decrypted, "hello");
    }

    #[test]
    fn text_content_hash_is_stable() {
        assert_eq!(text_content_hash("hello"), text_content_hash("hello"));
        assert_ne!(text_content_hash("hello"), text_content_hash("world"));
    }
}
