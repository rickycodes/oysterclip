use image::{ImageBuffer, ImageFormat, Rgba};
use serde::Deserialize;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::SystemTime;

use crate::common::{PasteEntry, CONFIG_FILE, FAILED_IMAGE_BUFFER, GPG_BINARY, GPG_RECIPIENT_ENV};

#[derive(Deserialize)]
struct AppConfig {
    gpg_recipient: Option<String>,
}

pub(crate) fn detect_text_kind(text: &str) -> &'static str {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return "empty";
    }

    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return "url";
    }

    if serde_json::from_str::<serde_json::Value>(trimmed).is_ok() {
        return "json";
    }

    if trimmed.contains('\n') || trimmed.contains('\r') {
        return "multiline";
    }

    "plain"
}

pub(crate) fn simple_image_hash(bytes: &[u8]) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

pub(crate) fn save_image(
    bytes: &[u8],
    width: usize,
    height: usize,
    hash: u64,
    image_dir: &Path,
) -> Result<String, Box<dyn std::error::Error>> {
    fs::create_dir_all(image_dir)?;

    let filename = format!("{}/img_{}.png", image_dir.display(), hash);
    let path = Path::new(&filename);

    let buffer: ImageBuffer<Rgba<u8>, _> =
        ImageBuffer::from_raw(width as u32, height as u32, bytes.to_vec())
            .ok_or(FAILED_IMAGE_BUFFER)?;

    buffer.save_with_format(path, ImageFormat::Png)?;

    Ok(filename.to_string())
}

fn io_error(message: impl Into<String>) -> io::Error {
    io::Error::other(message.into())
}

fn normalize_config_value(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn parse_gpg_recipient_config(contents: &str) -> io::Result<Option<String>> {
    let config: AppConfig = toml::from_str(contents)
        .map_err(|err| io_error(format!("failed to parse {}: {err}", CONFIG_FILE)))?;
    Ok(normalize_config_value(config.gpg_recipient))
}

pub(crate) fn resolve_gpg_recipient() -> io::Result<String> {
    if let Some(recipient) = normalize_config_value(std::env::var(GPG_RECIPIENT_ENV).ok()) {
        return Ok(recipient);
    }

    let config_path = Path::new(CONFIG_FILE);
    if config_path.exists() {
        let contents = fs::read_to_string(config_path)?;
        if let Some(recipient) = parse_gpg_recipient_config(&contents)? {
            return Ok(recipient);
        }
    }

    Err(io_error(format!(
        "missing GPG recipient; set {} or add gpg_recipient = \"your-key-id-or-email\" to {}",
        GPG_RECIPIENT_ENV, CONFIG_FILE
    )))
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
    use super::{
        current_timestamp, detect_text_kind, parse_gpg_recipient_config, push_history_entry,
        save_image,
    };
    use crate::common::PasteEntry;
    use std::fs;
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

    #[test]
    fn save_image_writes_png_to_dir() {
        let mut temp_dir = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        temp_dir.push(format!(
            "clipboard-watcher-test-{}-{}",
            std::process::id(),
            nanos
        ));
        fs::create_dir_all(&temp_dir).unwrap();

        let bytes = [0u8, 0u8, 0u8, 255u8];
        let filename = save_image(&bytes, 1, 1, 42, &temp_dir).unwrap();

        let path = std::path::Path::new(&filename);
        assert!(path.exists(), "expected image file to exist");
        assert!(filename.ends_with("img_42.png"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn detect_text_kind_classifies_common_types() {
        assert_eq!(detect_text_kind("https://example.com"), "url");
        assert_eq!(detect_text_kind("{\"a\":1}"), "json");
        assert_eq!(detect_text_kind("line1\nline2"), "multiline");
        assert_eq!(detect_text_kind("hello"), "plain");
        assert_eq!(detect_text_kind("   "), "empty");
    }

    #[test]
    fn parse_gpg_recipient_config_reads_value() {
        let config = "gpg_recipient = \"ricky@example.com\"\n";
        assert_eq!(
            parse_gpg_recipient_config(config).unwrap().as_deref(),
            Some("ricky@example.com")
        );
    }

    #[test]
    fn parse_gpg_recipient_config_treats_blank_as_missing() {
        let config = "gpg_recipient = \"   \"\n";
        assert_eq!(parse_gpg_recipient_config(config).unwrap(), None);
    }
}
