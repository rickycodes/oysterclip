use image::{ImageBuffer, ImageFormat, Rgba};
use std::fs;
use std::path::Path;

use crate::common::FAILED_IMAGE_BUFFER;

pub(crate) fn simple_image_hash(bytes: &[u8]) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

pub(crate) fn save_image(
    bytes: &[u8],
    width: usize,
    height: usize,
    hash: u64,
    image_dir: &Path,
) -> Result<String, Box<dyn std::error::Error>> {
    fs::create_dir_all(image_dir)?;

    let filename = format!("{}/img_{}.png", image_dir.display(), hash);
    let path = Path::new(&filename);

    let buffer: ImageBuffer<Rgba<u8>, _> =
        ImageBuffer::from_raw(width as u32, height as u32, bytes.to_vec())
            .ok_or(FAILED_IMAGE_BUFFER)?;

    buffer.save_with_format(path, ImageFormat::Png)?;

    Ok(filename.to_string())
}

#[cfg(test)]
mod tests {
    use super::save_image;
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
        let filename = save_image(&bytes, 1, 1, 42, &temp_dir).unwrap();

        let path = std::path::Path::new(&filename);
        assert!(path.exists(), "expected image file to exist");
        assert!(filename.ends_with("img_42.png"));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
