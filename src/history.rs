use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::SystemTime;

use crate::common::{PasteEntry, GPG_BINARY};

fn io_error(message: impl Into<String>) -> io::Error {
    io::Error::other(message.into())
}

fn decrypt_history(history_path: &Path) -> io::Result<Vec<PasteEntry>> {
    let output = Command::new(GPG_BINARY)
        .arg("--decrypt")
        .arg(history_path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(io_error(format!(
            "failed to decrypt {} with {}: {}",
            history_path.display(),
            GPG_BINARY,
            stderr.trim()
        )));
    }

    serde_json::from_slice(&output.stdout).map_err(|err| {
        io_error(format!(
            "failed to parse decrypted history {}: {err}",
            history_path.display()
        ))
    })
}

fn load_history(history_path: &Path) -> io::Result<Vec<PasteEntry>> {
    if history_path.exists() {
        decrypt_history(history_path)
    } else {
        Ok(Vec::new())
    }
}

fn write_encrypted_history(
    history: &[PasteEntry],
    history_path: &Path,
    recipient: &str,
) -> io::Result<()> {
    let json = serde_json::to_vec_pretty(history)
        .map_err(|err| io_error(format!("failed to serialize history: {err}")))?;

    let mut child = Command::new(GPG_BINARY)
        .arg("--encrypt")
        .arg("--recipient")
        .arg(recipient)
        .arg("--output")
        .arg(history_path)
        .arg("--yes")
        .arg("--batch")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| io_error(format!("failed to open {} stdin", GPG_BINARY)))?;
        stdin.write_all(&json)?;
    }

    let output = child.wait_with_output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(io_error(format!(
            "failed to encrypt {} with {} for recipient {}: {}",
            history_path.display(),
            GPG_BINARY,
            recipient,
            stderr.trim()
        )));
    }

    Ok(())
}

fn push_history_entry(history: &mut Vec<PasteEntry>, entry: &PasteEntry) -> bool {
    if let PasteEntry::Text {
        content: new_content,
        ..
    } = entry
    {
        let is_duplicate = history.iter().any(|existing| {
            matches!(
                existing,
                PasteEntry::Text { content, .. } if content == new_content
            )
        });

        if is_duplicate {
            return false;
        }
    }

    history.push(entry.clone());
    true
}

pub(crate) fn append_history(
    entry: &PasteEntry,
    history_path: &Path,
    recipient: &str,
) -> io::Result<()> {
    let mut history = load_history(history_path)?;

    if !push_history_entry(&mut history, entry) {
        return Ok(());
    }

    write_encrypted_history(&history, history_path, recipient)
}

pub(crate) fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::{current_timestamp, push_history_entry};
    use crate::common::PasteEntry;
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
    fn push_history_entry_deduplicates_text_entries() {
        let entry = PasteEntry::Text {
            timestamp: 1,
            content: "hello".to_string(),
            kind: Some("plain".to_string()),
        };

        let mut history = Vec::new();
        assert!(push_history_entry(&mut history, &entry));
        assert!(!push_history_entry(&mut history, &entry));

        assert_eq!(history.len(), 1);
        match &history[0] {
            PasteEntry::Text { content, kind, .. } => {
                assert_eq!(content, "hello");
                assert_eq!(kind.as_deref(), Some("plain"));
            }
            _ => panic!("expected text entry"),
        }
    }
}
