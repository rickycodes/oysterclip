// clipboard_watcher.rs
// Single-file Rust clipboard watcher without TTS functionality.
// Instructions: save as src/main.rs in a new cargo project, or run with `cargo run` after creating Cargo.toml as indicated below.

/*
Cargo.toml (add this to your project):

[package]
name = "clipboard-watcher"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = { version = "4", features = ["derive"] }
arboard = "2"
humantime = "2"
*/

use std::thread::sleep;
use std::time::{Duration, SystemTime};
use clap::Parser;

/// Watch the clipboard and print new clipboard text.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Polling interval in milliseconds
    #[arg(short = 'i', long = "interval", default_value_t = 500)]
    interval_ms: u64,
}

fn main() {
    let args = Args::parse();

    println!("Starting clipboard watcher — interval: {}ms", args.interval_ms);

    let mut clipboard = match arboard::Clipboard::new() {
        Ok(cb) => cb,
        Err(e) => {
            eprintln!("Failed to open clipboard: {:#?}", e);
            return;
        }
    };

    let mut last: Option<String> = None;
    let interval = Duration::from_millis(args.interval_ms);

    loop {
        match clipboard.get_text() {
            Ok(text) => {
                // Normalize newlines and trim.
                let normalized = text.replace('\r', "").trim().to_string();
                if let Some(prev) = &last {
                    if &normalized != prev {
                        on_change(&normalized);
                        last = Some(normalized);
                    }
                } else {
                    // First read; treat as initial state but still announce it
                    on_change(&normalized);
                    last = Some(normalized);
                }
            }
            Err(e) => {
                eprintln!("Error reading clipboard: {:#?}", e);
                // On some platforms the clipboard can be temporarily inaccessible; continue.
            }
        }

        sleep(interval);
    }
}

fn on_change(text: &str) {
    let now = SystemTime::now();
    let ts = humantime::format_rfc3339_seconds(now);

    println!("[{}] Clipboard changed:\n{}\n---", ts, text);
}
