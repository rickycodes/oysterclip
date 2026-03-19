use arboard::Clipboard;
use std::thread::sleep;
use std::time::Duration;

mod cli;
mod common;
mod history;
mod image_store;
mod text;

use crate::cli::{print_help, print_version};
use crate::common::{PasteEntry, CLIPBOARD_NOT_AVAILABLE, HISTORY_FILE, IMAGE_DIR, INTERVAL_MS};
use crate::history::{current_timestamp, HistoryStore};
use crate::image_store::{save_image, simple_image_hash};
use crate::text::detect_text_kind;
use std::path::Path;

fn main() {
    if let Some(flag) = std::env::args()
        .skip(1)
        .find(|arg| matches!(arg.as_str(), "-h" | "--help" | "-V" | "--version"))
    {
        match flag.as_str() {
            "-h" | "--help" => {
                print_help();
                return;
            }
            "-V" | "--version" => {
                print_version();
                return;
            }
            _ => unreachable!(),
        }
    }

    println!("Starting clipboard watcher — interval: {}ms", INTERVAL_MS);
    let history_store = HistoryStore::open(Path::new(HISTORY_FILE)).unwrap_or_else(|err| {
        eprintln!("Failed to open history store: {}", err);
        std::process::exit(1);
    });

    let mut clipboard = Clipboard::new().expect(CLIPBOARD_NOT_AVAILABLE);
    let mut last_text: Option<String> = None;
    let mut last_image_hash: Option<u64> = None;

    loop {
        if let Ok(text) = clipboard.get_text() {
            if Some(&text) != last_text.as_ref() {
                let kind = detect_text_kind(&text);
                if kind == "empty" {
                    println!("(text:empty) skipped");
                } else if kind == "image-data-uri" {
                    println!(
                        "(text:image-data-uri) skipped {} chars for review",
                        text.chars().count()
                    );
                } else {
                    println!("(text:{}) captured {} chars", kind, text.chars().count());
                    if let Err(err) = history_store.append_entry(&PasteEntry::Text {
                        timestamp: current_timestamp(),
                        content: text.clone(),
                        kind: Some(kind.to_string()),
                    }) {
                        eprintln!("Failed to append text history: {}", err);
                    }
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
                    if let Err(err) = history_store.append_entry(&PasteEntry::Image {
                        timestamp: current_timestamp(),
                        path,
                        hash,
                    }) {
                        eprintln!("Failed to append image history: {}", err);
                    }
                }
                last_image_hash = Some(hash);
            }
        }

        sleep(Duration::from_millis(INTERVAL_MS));
    }
}
