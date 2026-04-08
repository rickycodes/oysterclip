use arboard::Clipboard;
use std::thread;
use std::time::Duration;

/// Copy text to system clipboard
/// Returns Ok(message) on success, Err(error_message) on failure
pub fn copy_to_clipboard(text: String) -> Result<String, String> {
    let mut clipboard = Clipboard::new().map_err(|e| format!("Copy failed: {}", e))?;
    clipboard.set_text(text).map_err(|e| format!("Copy failed: {}", e))?;
    
    // Keep clipboard object alive briefly to ensure clipboard managers can read it
    thread::sleep(Duration::from_millis(100));
    
    Ok("Copied to clipboard".to_string())
}
