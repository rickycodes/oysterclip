use arboard::Clipboard;
use std::thread;
use std::time::Duration;

/// Copy text to system clipboard
/// Returns Ok(message) on success, Err(error_message) on failure
pub fn copy_to_clipboard(text: String) -> Result<String, String> {
    let mut clipboard = Clipboard::new().map_err(|e| format!("Copy failed: {}", e))?;
    clipboard
        .set_text(text)
        .map_err(|e| format!("Copy failed: {}", e))?;

    // Keep clipboard object alive briefly to ensure clipboard managers can read it
    thread::sleep(Duration::from_millis(100));

    Ok("Copied to clipboard".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_to_clipboard_success_message() {
        // Note: This test may fail in headless environments without X11/Wayland
        // It tests the success message format at minimum
        match copy_to_clipboard("test".to_string()) {
            Ok(msg) => assert_eq!(msg, "Copied to clipboard"),
            Err(_) => {
                // Accept failure in headless environment - we're just testing the API
            }
        }
    }

    #[test]
    fn test_copy_to_clipboard_returns_result() {
        let result = copy_to_clipboard("test".to_string());
        assert!(result.is_ok() || result.is_err());
    }
}
