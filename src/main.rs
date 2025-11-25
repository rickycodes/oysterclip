use arboard::Clipboard;
use std::thread::sleep;
use std::time::{Duration};

mod constants;
mod utils;
use crate::constants::{INTERVAL_MS};
use crate::utils::{save_image, append_history, current_timestamp,simple_image_hash,PasteEntry};

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
