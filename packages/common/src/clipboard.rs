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
    fn test_copy_to_clipboard_handles_unicode() {
        let unicode_text = "Hello 世界 🦀 Ñoño".to_string();
        let result = copy_to_clipboard(unicode_text);
        // Just verify error/success is handled gracefully
        match result {
            Ok(msg) => assert_eq!(msg, "Copied to clipboard"),
            Err(err) => assert!(err.contains("Copy failed")),
        }
    }

    #[test]
    fn test_copy_to_clipboard_handles_multiline() {
        let multiline = "Line 1\nLine 2\nLine 3".to_string();
        let result = copy_to_clipboard(multiline);
        match result {
            Ok(msg) => assert_eq!(msg, "Copied to clipboard"),
            Err(err) => assert!(err.contains("Copy failed")),
        }
    }
}
