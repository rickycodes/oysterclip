use arboard::Clipboard;
use std::thread::sleep;
use std::time::Duration;

mod common;
mod utils;

use crate::common::{PasteEntry, CLIPBOARD_NOT_AVAILABLE, HISTORY_FILE, IMAGE_DIR, INTERVAL_MS};
use crate::utils::{
    append_history, current_timestamp, detect_text_kind, resolve_gpg_recipient, save_image,
    simple_image_hash,
};
use std::path::Path;

fn main() {
    println!("Starting clipboard watcher — interval: {}ms", INTERVAL_MS);
    let gpg_recipient = resolve_gpg_recipient().unwrap_or_else(|err| {
        eprintln!("Failed to resolve GPG recipient: {}", err);
        std::process::exit(1);
    });

    let mut clipboard = Clipboard::new().expect(CLIPBOARD_NOT_AVAILABLE);
    let mut last_text: Option<String> = None;
    let mut last_image_hash: Option<u64> = None;

    loop {
        if let Ok(text) = clipboard.get_text() {
            if Some(&text) != last_text.as_ref() {
                let kind = detect_text_kind(&text);
                println!("(text:{}) captured {} chars", kind, text.chars().count());
                if let Err(err) = append_history(
                    &PasteEntry::Text {
                        timestamp: current_timestamp(),
                        content: text.clone(),
                        kind: Some(kind.to_string()),
                    },
                    Path::new(HISTORY_FILE),
                    &gpg_recipient,
                ) {
                    eprintln!("Failed to append text history: {}", err);
                }
                last_text = Some(text);
            }
        }

        if let Ok(img) = clipboard.get_image() {
            let bytes: Vec<u8> = img.bytes.to_vec();
            let hash = simple_image_hash(&bytes);

            if Some(hash) != last_image_hash {
                if let Ok(path) =
                    save_image(&bytes, img.width, img.height, hash, Path::new(IMAGE_DIR))
                {
                    println!("(image) saved: {}", path);
                    if let Err(err) = append_history(
                        &PasteEntry::Image {
                            timestamp: current_timestamp(),
                            path,
                            hash,
                        },
                        Path::new(HISTORY_FILE),
                        &gpg_recipient,
                    ) {
                        eprintln!("Failed to append image history: {}", err);
                    }
                }
                last_image_hash = Some(hash);
            }
        }

        sleep(Duration::from_millis(INTERVAL_MS));
    }
}
