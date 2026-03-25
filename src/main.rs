use arboard::Clipboard;
use clap::Parser;
use std::thread::sleep;
use std::time::Duration;

mod cli;
mod config;
mod constants;
mod entry;
mod history;
mod image_store;
mod text;

use crate::cli::{Cli, Commands};
use crate::config::load_max_history_entries;
use crate::constants::{
    APPEND_IMAGE_HISTORY_FAILED, APPEND_TEXT_HISTORY_FAILED, CLIPBOARD_NOT_AVAILABLE, HISTORY_FILE,
    IMAGE_DIR, IMAGE_SAVED, INTERVAL_MS, OPEN_HISTORY_STORE_FAILED, STARTUP_MESSAGE, TEXT_CAPTURED,
    TEXT_EMPTY_SKIPPED, TEXT_IMAGE_DATA_URI_SKIPPED, TEXT_KIND_EMPTY, TEXT_KIND_IMAGE_DATA_URI,
};
use crate::entry::PasteEntry;
use crate::history::{current_timestamp, HistoryStore};
use crate::image_store::{save_image, simple_image_hash};
use crate::text::detect_text_kind;
use std::path::Path;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Version) => {
            println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            return;
        }
        Some(Commands::Watch) | None => {
            // Fall through to the main watch logic
        }
    }

    println!("{STARTUP_MESSAGE} — interval: {INTERVAL_MS}ms");
    let max_history_entries = load_max_history_entries();
    let history_store = HistoryStore::open(Path::new(HISTORY_FILE), max_history_entries)
        .unwrap_or_else(|err| {
            eprintln!("{OPEN_HISTORY_STORE_FAILED}: {err}");
            std::process::exit(1);
        });

    let mut clipboard = Clipboard::new().expect(CLIPBOARD_NOT_AVAILABLE);
    let mut last_text: Option<String> = None;
    let mut last_image_hash: Option<u64> = None;

    loop {
        if let Ok(text) = clipboard.get_text() {
            if Some(&text) != last_text.as_ref() {
                let kind = detect_text_kind(&text);
                if kind == TEXT_KIND_EMPTY {
                    println!("{TEXT_EMPTY_SKIPPED}");
                } else if kind == TEXT_KIND_IMAGE_DATA_URI {
                    println!(
                        "{TEXT_IMAGE_DATA_URI_SKIPPED} {} chars for review",
                        text.chars().count()
                    );
                } else {
                    println!(
                        "(text:{kind}) {TEXT_CAPTURED} {} chars",
                        text.chars().count()
                    );
                    if let Err(err) = history_store.append_entry(&PasteEntry::Text {
                        timestamp: current_timestamp(),
                        content: text.clone(),
                        kind: Some(kind.to_string()),
                    }) {
                        eprintln!("{APPEND_TEXT_HISTORY_FAILED}: {err}");
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
                    println!("{IMAGE_SAVED}: {path}");
                    if let Err(err) = history_store.append_entry(&PasteEntry::Image {
                        timestamp: current_timestamp(),
                        path,
                        hash,
                    }) {
                        eprintln!("{APPEND_IMAGE_HISTORY_FAILED}: {err}");
                    }
                }
                last_image_hash = Some(hash);
            }
        }

        sleep(Duration::from_millis(INTERVAL_MS));
    }
}
