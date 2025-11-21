use arboard::Clipboard;
use image::{ImageBuffer, Rgba, ImageFormat};
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

const HISTORY_FILE: &str = ".paste_history.json";
const IMAGE_DIR: &str = "clipboard_images";
const INTERVAL_MS: u64 = 500;

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
enum PasteEntry {
    Text { timestamp: u64, content: String },
    Image { timestamp: u64, path: String, hash: u64 },
}

fn simple_image_hash(bytes: &[u8]) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn save_image(bytes: &[u8], width: usize, height: usize, hash: u64) -> Result<String, Box<dyn std::error::Error>> {
    fs::create_dir_all(IMAGE_DIR)?;

    let filename = format!("{}/img_{}.png", IMAGE_DIR, hash);
    let path = Path::new(&filename);

    let buffer: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_raw(
        width as u32,
        height as u32,
        bytes.to_vec(),
    ).ok_or("Failed to create image buffer")?;

    buffer.save_with_format(path, ImageFormat::Png)?;

    Ok(filename.to_string())
}

fn append_history(entry: &PasteEntry) {
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

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn main() {
    println!("Starting clipboard watcher — interval: {}ms", INTERVAL_MS);

    let mut clipboard = Clipboard::new().expect("Clipboard not available");
    let mut last_text: Option<String> = None;
    let mut last_image_hash: Option<u64> = None;

    loop {
        // TEXT
        if let Ok(text) = clipboard.get_text() {
            if Some(&text) != last_text.as_ref() {
                println!("(text) {}", text);
                append_history(&PasteEntry::Text {
                    timestamp: current_timestamp(),
                    content: text.clone(),
                });
                last_text = Some(text);
            }
        }

        // IMAGE
        if let Ok(img) = clipboard.get_image() {
            let bytes: Vec<u8> = img.bytes.to_vec();
            let hash = simple_image_hash(&bytes);

            if Some(hash) != last_image_hash {
                if let Ok(path) = save_image(&bytes, img.width, img.height, hash) {
                    println!("(image) saved: {}", path);
                    append_history(&PasteEntry::Image {
                        timestamp: current_timestamp(),
                        path,
                        hash,
                    });
                }
                last_image_hash = Some(hash);
            }
        }

        sleep(Duration::from_millis(INTERVAL_MS));
    }
}
