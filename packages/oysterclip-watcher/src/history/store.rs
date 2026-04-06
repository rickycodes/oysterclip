use rusqlite::{params, Connection, OptionalExtension};
use std::io;
use std::path::{Path, PathBuf};

use super::crypto::{encrypt_text, load_or_create_encryption_key, text_content_hash};
use crate::config::constants::{
    CREATE_ENTRIES_TABLE_SQL, CREATE_INDICES, DELETE_PRUNABLE_ENTRIES_SQL, INSERT_IMAGE_ENTRY_SQL,
    INSERT_TEXT_ENTRY_SQL, SELECT_EXISTING_TEXT_ENTRY_SQL,
};
use crate::data::entry::PasteEntry;

pub(crate) struct HistoryStore {
    db_path: PathBuf,
    encryption_key: [u8; 32],
    max_history_entries: usize,
}

fn io_error(message: impl Into<String>) -> io::Error {
    io::Error::other(message.into())
}

impl HistoryStore {
    pub(crate) fn open(db_path: &Path, max_history_entries: usize) -> io::Result<Self> {
        let store = Self {
            db_path: db_path.to_path_buf(),
            encryption_key: load_or_create_encryption_key()?,
            max_history_entries,
        };
        let conn = store.connection()?;
        conn.execute_batch(CREATE_ENTRIES_TABLE_SQL)
            .map_err(|err| io_error(format!("failed to initialize history database: {err}")))?;
        for index_sql in CREATE_INDICES {
            conn.execute_batch(index_sql)
                .map_err(|err| io_error(format!("failed to create index: {err}")))?;
        }
        ensure_image_blob_column(&conn)?;
        Ok(store)
    }

    pub(crate) fn append_entry(&self, entry: &PasteEntry) -> io::Result<()> {
        let conn = self.connection()?;

        match entry {
            PasteEntry::Text {
                timestamp,
                content,
                kind,
            } => self.insert_text_entry(&conn, *timestamp, content, kind.as_deref())?,
            PasteEntry::Image {
                timestamp,
                png_bytes,
                path,
                hash,
            } => self.insert_image_entry(&conn, *timestamp, path.as_deref(), png_bytes, *hash)?,
        };

        prune_history(&conn, &self.db_path, self.max_history_entries)
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
        path: Option<&str>,
        png_bytes: &[u8],
        hash: u64,
    ) -> io::Result<()> {
        conn.execute(
            INSERT_IMAGE_ENTRY_SQL,
            params![timestamp as i64, path, png_bytes, hash as i64],
        )
        .map_err(|err| io_error(format!("failed to insert image history entry: {err}")))?;

        Ok(())
    }
}

fn ensure_image_blob_column(conn: &Connection) -> io::Result<()> {
    let mut stmt = conn
        .prepare("PRAGMA table_info(entries)")
        .map_err(|err| io_error(format!("failed to inspect history schema: {err}")))?;
    let mut rows = stmt
        .query([])
        .map_err(|err| io_error(format!("failed to query history schema: {err}")))?;

    while let Some(row) = rows
        .next()
        .map_err(|err| io_error(format!("failed to iterate history schema: {err}")))?
    {
        let name: String = row
            .get(1)
            .map_err(|err| io_error(format!("failed to read history schema column: {err}")))?;
        if name == "image_png" {
            return Ok(());
        }
    }

    conn.execute("ALTER TABLE entries ADD COLUMN image_png BLOB", [])
        .map_err(|err| io_error(format!("failed to add image blob column: {err}")))?;
    Ok(())
}

fn prune_history(conn: &Connection, _db_path: &Path, max_entries: usize) -> io::Result<()> {
    conn.execute(DELETE_PRUNABLE_ENTRIES_SQL, params![max_entries as i64])
        .map_err(|err| io_error(format!("failed to prune old history entries: {err}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::crypto::{decrypt_text, encrypt_text, text_content_hash};
    use super::ensure_image_blob_column;
    use super::prune_history;
    use rusqlite::{params, Connection};
    use std::path::Path;
    use std::time::SystemTime;

    use crate::config::constants::CREATE_ENTRIES_TABLE_SQL;

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

    #[test]
    fn prune_history_keeps_latest_entries() {
        let mut temp_dir = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        temp_dir.push(format!(
            "clipboard-watcher-history-prune-{}-{}",
            std::process::id(),
            nanos
        ));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let db_path = temp_dir.join(".clipboard_history.db");
        let conn = Connection::open(&db_path).unwrap();
        conn.execute_batch(CREATE_ENTRIES_TABLE_SQL).unwrap();
        ensure_image_blob_column(&conn).unwrap();

        let old_path = "clipboard_images/old.png";
        let kept_path = "clipboard_images/kept.png";
        let png_bytes = vec![1u8, 2, 3];

        conn.execute(
            "INSERT INTO entries (created_at, entry_type, image_path, image_png, image_hash) VALUES (1, 'image', ?1, ?2, 1)",
            params![old_path, png_bytes.clone()],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO entries (created_at, entry_type, image_path, image_png, image_hash) VALUES (2, 'image', ?1, ?2, 2)",
            params![kept_path, png_bytes],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO entries (created_at, entry_type, text_kind, text_ciphertext, text_nonce, content_hash) VALUES (3, 'text', 'plain', X'01', X'020202020202020202020202020202020202020202020202', 'hash')",
            [],
        )
        .unwrap();

        prune_history(&conn, Path::new(&db_path), 2).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM entries", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 2);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
