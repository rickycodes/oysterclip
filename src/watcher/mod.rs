use arboard::Clipboard;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use crate::config::constants::{
    APPEND_IMAGE_HISTORY_FAILED, APPEND_TEXT_HISTORY_FAILED, CLIPBOARD_NOT_AVAILABLE, IMAGE_SAVED,
    INTERVAL_MS, TEXT_CAPTURED, TEXT_EMPTY_SKIPPED, TEXT_IMAGE_DATA_URI_SKIPPED, TEXT_KIND_EMPTY,
    TEXT_KIND_IMAGE_DATA_URI,
};
use crate::data::entry::PasteEntry;
use crate::data::text::detect_text_kind;
use crate::data::image_store::{encode_png, save_png, simple_image_hash};
use crate::history::{current_timestamp, HistoryStore};
use crate::ipc::SharedControlState;

pub fn start_watching(
    history_store: HistoryStore,
    control_state: SharedControlState,
    save_images_to_disk: bool,
    image_export_dir: &std::path::Path,
) -> std::io::Result<()> {
    let shutdown = Arc::new(AtomicBool::new(false));

    // Set up signal handler for graceful shutdown (Unix only)
    #[cfg(unix)]
    {
        let shutdown_clone = shutdown.clone();
        let mut signals = signal_hook::iterator::Signals::new([signal_hook::consts::signal::SIGTERM, signal_hook::consts::signal::SIGINT])?;
        std::thread::spawn(move || {
            for _ in signals.forever() {
                shutdown_clone.store(true, Ordering::SeqCst);
            }
        });
    }

    let mut clipboard = Clipboard::new().expect(CLIPBOARD_NOT_AVAILABLE);
    let mut last_text: Option<String> = None;
    let mut last_image_hash: Option<u64> = None;

    loop {
        if shutdown.load(Ordering::SeqCst) {
            println!("Shutting down gracefully...");
            break;
        }

        if is_paused(&control_state) {
            sleep(Duration::from_millis(INTERVAL_MS));
            continue;
        }

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
                    match history_store.append_entry(&PasteEntry::Text {
                        timestamp: current_timestamp(),
                        content: text.clone(),
                        kind: Some(kind.to_string()),
                    }) {
                        Ok(_) => mark_capture_success(&control_state),
                        Err(err) => {
                            eprintln!("{APPEND_TEXT_HISTORY_FAILED}: {err}");
                            set_last_error(
                                &control_state,
                                format!("{APPEND_TEXT_HISTORY_FAILED}: {err}"),
                            );
                        }
                    }
                }
                last_text = Some(text);
            }
        }

        if let Ok(img) = clipboard.get_image() {
            let bytes: Vec<u8> = img.bytes.to_vec();
            let hash = simple_image_hash(&bytes);

            if Some(hash) != last_image_hash {
                match encode_png(&bytes, img.width, img.height) {
                    Ok(png_bytes) => {
                        let exported_path = if save_images_to_disk {
                            match save_png(&png_bytes, hash, image_export_dir) {
                                Ok(path) => {
                                    println!("{IMAGE_SAVED}: {path}");
                                    Some(path)
                                }
                                Err(err) => {
                                    let message = format!("Failed to export image: {err}");
                                    eprintln!("{message}");
                                    set_last_error(&control_state, message);
                                    None
                                }
                            }
                        } else {
                            None
                        };

                        match history_store.append_entry(&PasteEntry::Image {
                            timestamp: current_timestamp(),
                            png_bytes,
                            path: exported_path,
                            hash,
                        }) {
                            Ok(_) => mark_capture_success(&control_state),
                            Err(err) => {
                                eprintln!("{APPEND_IMAGE_HISTORY_FAILED}: {err}");
                                set_last_error(
                                    &control_state,
                                    format!("{APPEND_IMAGE_HISTORY_FAILED}: {err}"),
                                );
                            }
                        }
                    }
                    Err(err) => {
                        set_last_error(&control_state, format!("Failed to encode image: {err}"));
                    }
                }
                last_image_hash = Some(hash);
            }
        }

        sleep(Duration::from_millis(INTERVAL_MS));
    }

    Ok(())
}

fn is_paused(state: &SharedControlState) -> bool {
    state.lock().map(|guard| guard.paused).unwrap_or(false)
}

fn mark_capture_success(state: &SharedControlState) {
    if let Ok(mut guard) = state.lock() {
        guard.last_capture_at = Some(current_timestamp());
        guard.last_error = None;
    }
}

fn set_last_error(state: &SharedControlState, message: String) {
    if let Ok(mut guard) = state.lock() {
        guard.last_error = Some(message);
    }
}
