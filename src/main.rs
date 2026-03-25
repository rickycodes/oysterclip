use arboard::Clipboard;
use clap::Parser;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

mod cli;
mod config;
mod constants;
mod entry;
mod history;
mod image_store;
mod ipc;
mod text;

use crate::cli::{Cli, Commands, ControlAction};
use crate::config::load_config;
use crate::constants::{
    APPEND_IMAGE_HISTORY_FAILED, APPEND_TEXT_HISTORY_FAILED, CLIPBOARD_NOT_AVAILABLE, HISTORY_FILE,
    IMAGE_SAVED, INTERVAL_MS, OPEN_HISTORY_STORE_FAILED, STARTUP_MESSAGE, TEXT_CAPTURED,
    TEXT_EMPTY_SKIPPED, TEXT_IMAGE_DATA_URI_SKIPPED, TEXT_KIND_EMPTY, TEXT_KIND_IMAGE_DATA_URI,
};
use crate::entry::PasteEntry;
use crate::history::{current_timestamp, HistoryStore};
use crate::image_store::{encode_png, save_png, simple_image_hash};
use crate::ipc::{
    new_control_state, print_control_response, send_control_command, start_control_server,
    SharedControlState,
};
use crate::text::detect_text_kind;

fn main() {
    let cli = Cli::parse();
    let db_path = Path::new(HISTORY_FILE);
    let mut start_paused = false;

    match cli.command {
        Some(Commands::Version) => {
            println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            return;
        }
        Some(Commands::Control(control)) => {
            let cmd = match control.action {
                ControlAction::Pause => "pause",
                ControlAction::Resume => "resume",
                ControlAction::Status => "status",
            };
            run_control_command(db_path, cmd);
            return;
        }
        Some(Commands::Watch(args)) => {
            start_paused = args.paused;
        }
        None => {}
    }

    println!("{STARTUP_MESSAGE} - interval: {INTERVAL_MS}ms");
    let config = load_config();
    let image_export_dir = Path::new(&config.image_export_dir);
    let history_store =
        HistoryStore::open(db_path, config.max_history_entries).unwrap_or_else(|err| {
            eprintln!("{OPEN_HISTORY_STORE_FAILED}: {err}");
            std::process::exit(1);
        });

    let control_state = new_control_state(db_path, image_export_dir, current_timestamp());
    if start_paused {
        if let Ok(mut guard) = control_state.lock() {
            guard.paused = true;
        }
    }
    let _control_guard =
        start_control_server(control_state.clone(), db_path).unwrap_or_else(|err| {
            eprintln!("Failed to start watcher control socket: {err}");
            std::process::exit(1);
        });

    let mut clipboard = Clipboard::new().expect(CLIPBOARD_NOT_AVAILABLE);
    let mut last_text: Option<String> = None;
    let mut last_image_hash: Option<u64> = None;

    loop {
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
                        let exported_path = if config.save_images_to_disk {
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
}

fn run_control_command(db_path: &Path, cmd: &str) {
    match send_control_command(db_path, cmd) {
        Ok(response) => print_control_response(&response),
        Err(err) => {
            eprintln!("Failed to send `{cmd}` command: {err}");
            std::process::exit(1);
        }
    }
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
