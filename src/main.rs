use clap::Parser;
use image::{ImageBuffer, Rgba};
use std::path::PathBuf;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

/// Watch the clipboard and print changes (text or images).
#[derive(Parser, Debug)]
struct Args {
    /// Polling interval in milliseconds
    #[arg(short = 'i', long = "interval", default_value_t = 500)]
    interval_ms: u64,

    /// Directory to save new clipboard images
    #[arg(short = 'o', long = "output", default_value = "clipboard_images")]
    output_dir: String,
}

fn main() {
    let args = Args::parse();

    println!(
        "Starting clipboard watcher — interval: {}ms — image output: {}",
        args.interval_ms, args.output_dir
    );

    let mut clipboard = match arboard::Clipboard::new() {
        Ok(cb) => cb,
        Err(e) => {
            eprintln!("Failed to open clipboard: {:#?}", e);
            return;
        }
    };

    let mut last_text: Option<String> = None;
    let mut last_image_hash: Option<u64> = None;

    std::fs::create_dir_all(&args.output_dir).ok();

    let interval = Duration::from_millis(args.interval_ms);

    loop {
        // TEXT CHECK
        if let Ok(text) = clipboard.get_text() {
            let normalized = text.replace('\r', "").trim().to_string();
            if last_text.as_ref() != Some(&normalized) {
                on_text_change(&normalized);
                last_text = Some(normalized);
            }
        }

        // IMAGE CHECK
        if let Ok(img) = clipboard.get_image() {
            let hash = simple_image_hash(&img.bytes);
            if last_image_hash != Some(hash) {
                if let Err(e) = on_image_change(
                    &img.bytes,
                    img.width as u32,
                    img.height as u32,
                    &args.output_dir,
                ) {
                    eprintln!("Failed to save image: {}", e);
                }
                last_image_hash = Some(hash);
            }
        }

        sleep(interval);
    }
}

fn on_text_change(text: &str) {
    let ts = humantime::format_rfc3339_seconds(SystemTime::now());
    println!("[{}] Clipboard TEXT changed:\n{}\n---", ts, text);
}

fn on_image_change(raw: &[u8], w: u32, h: u32, dir: &str) -> Result<(), String> {
    let ts = humantime::format_rfc3339_seconds(SystemTime::now());

    println!("[{}] Clipboard IMAGE changed: {}x{} (saved)", ts, w, h);

    // Convert raw RGBA bytes to ImageBuffer
    let buffer: ImageBuffer<Rgba<u8>, _> =
        ImageBuffer::from_raw(w, h, raw.to_vec()).ok_or("Failed to create image buffer")?;

    let filename = format!(
        "clipboard_{}.png",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );
    let mut path = PathBuf::from(dir);
    path.push(filename);

    buffer
        .save(&path)
        .map_err(|e| format!("Failed to save PNG: {}", e))?;

    Ok(())
}

/// Very cheap non-cryptographic byte hash
fn simple_image_hash(bytes: &[u8]) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for &b in bytes {
        h = h ^ (b as u64);
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}
