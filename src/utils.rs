use image::{ImageBuffer, ImageFormat, Rgba};
use std::fs;
use std::path::Path;
use std::time::SystemTime;

use crate::common::{PasteEntry, FAILED_IMAGE_BUFFER};

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

pub(crate) fn append_history(entry: &PasteEntry, history_path: &Path) {
    let mut history: Vec<PasteEntry> = if history_path.exists() {
        fs::read_to_string(history_path)
            .ok()
            .and_then(|data| serde_json::from_str(&data).ok())
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    if let PasteEntry::Text { content: new_content, .. } = entry {
        let is_duplicate = history.iter().any(|existing| {
            matches!(
                existing,
                PasteEntry::Text { content, .. } if content == new_content
            )
        });

        if is_duplicate {
            return;
        }
    }

    history.push(entry.clone());

    if let Ok(json) = serde_json::to_string_pretty(&history) {
        let _ = fs::write(history_path, json);
    }
}

pub(crate) fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::{append_history, current_timestamp, save_image};
    use crate::common::{PasteEntry, HISTORY_FILE};
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
    fn append_history_deduplicates_text_entries() {
        let mut temp_dir = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        temp_dir.push(format!("clipboard-watcher-test-{}-{}", std::process::id(), nanos));
        fs::create_dir_all(&temp_dir).unwrap();

        let entry = PasteEntry::Text {
            timestamp: 1,
            content: "hello".to_string(),
        };

        let history_path = temp_dir.join(HISTORY_FILE);
        append_history(&entry, &history_path);
        append_history(&entry, &history_path);

        let data = fs::read_to_string(&history_path).unwrap();
        let history: Vec<PasteEntry> = serde_json::from_str(&data).unwrap();

        assert_eq!(history.len(), 1);
        match &history[0] {
            PasteEntry::Text { content, .. } => assert_eq!(content, "hello"),
            _ => panic!("expected text entry"),
        }

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn save_image_writes_png_to_dir() {
        let mut temp_dir = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        temp_dir.push(format!("clipboard-watcher-test-{}-{}", std::process::id(), nanos));
        fs::create_dir_all(&temp_dir).unwrap();

        let bytes = [0u8, 0u8, 0u8, 255u8];
        let filename = save_image(&bytes, 1, 1, 42, &temp_dir).unwrap();

        let path = std::path::Path::new(&filename);
        assert!(path.exists(), "expected image file to exist");
        assert!(filename.ends_with("img_42.png"));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
