use image::{ImageBuffer, ImageFormat, Rgba};
use std::fs;
use std::io::Cursor;
use std::path::Path;

use crate::constants::FAILED_IMAGE_BUFFER;

pub(crate) fn simple_image_hash(bytes: &[u8]) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn image_buffer(
    bytes: &[u8],
    width: usize,
    height: usize,
) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Box<dyn std::error::Error>> {
    let buffer: ImageBuffer<Rgba<u8>, _> =
        ImageBuffer::from_raw(width as u32, height as u32, bytes.to_vec())
            .ok_or(FAILED_IMAGE_BUFFER)?;
    Ok(buffer)
}

pub(crate) fn encode_png(
    bytes: &[u8],
    width: usize,
    height: usize,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let buffer = image_buffer(bytes, width, height)?;
    let mut cursor = Cursor::new(Vec::new());
    buffer.write_to(&mut cursor, ImageFormat::Png)?;
    Ok(cursor.into_inner())
}

pub(crate) fn save_png(
    png_bytes: &[u8],
    hash: u64,
    image_dir: &Path,
) -> Result<String, Box<dyn std::error::Error>> {
    fs::create_dir_all(image_dir)?;

    let filename = format!("{}/img_{}.png", image_dir.display(), hash);
    let path = Path::new(&filename);
    fs::write(path, png_bytes)?;

    Ok(filename.to_string())
}

#[cfg(test)]
mod tests {
    use super::{encode_png, save_png};
    use std::fs;
    use std::time::SystemTime;

    #[test]
    fn save_image_writes_png_to_dir() {
        let mut temp_dir = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        temp_dir.push(format!(
            "clipboard-watcher-test-{}-{}",
            std::process::id(),
            nanos
        ));
        fs::create_dir_all(&temp_dir).unwrap();

        let bytes = [0u8, 0u8, 0u8, 255u8];
        let png_bytes = encode_png(&bytes, 1, 1).unwrap();
        let filename = save_png(&png_bytes, 42, &temp_dir).unwrap();

        let path = std::path::Path::new(&filename);
        assert!(path.exists(), "expected image file to exist");
        assert!(filename.ends_with("img_42.png"));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
