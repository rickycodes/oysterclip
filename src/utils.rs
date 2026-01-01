use image::{ImageBuffer, ImageFormat, Rgba};
use std::fs;
use std::path::Path;
use std::time::SystemTime;

use crate::common::{PasteEntry, FAILED_IMAGE_BUFFER, HISTORY_FILE, IMAGE_DIR};

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
) -> Result<String, Box<dyn std::error::Error>> {
    fs::create_dir_all(IMAGE_DIR)?;

    let filename = format!("{}/img_{}.png", IMAGE_DIR, hash);
    let path = Path::new(&filename);

    let buffer: ImageBuffer<Rgba<u8>, _> =
        ImageBuffer::from_raw(width as u32, height as u32, bytes.to_vec())
            .ok_or(FAILED_IMAGE_BUFFER)?;

    buffer.save_with_format(path, ImageFormat::Png)?;

    Ok(filename.to_string())
}

pub(crate) fn append_history(entry: &PasteEntry) {
    let mut history = if Path::new(HISTORY_FILE).exists() {
        let data = fs::read_to_string(HISTORY_FILE).unwrap_or_default();
        serde_json::from_str::<Vec<PasteEntry>>(&data).unwrap_or_else(|_| vec![])
    } else {
        vec![]
    };

    history.push(entry.clone());

    if let Ok(json) = serde_json::to_string_pretty(&history) {
        let _ = fs::write(HISTORY_FILE, json);
    }
}

pub(crate) fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
