/// Tracks clipboard state to detect changes and avoid duplicate captures.
#[derive(Debug, Clone)]
pub struct ChangeDetectionState {
    pub last_text: Option<String>,
    pub last_image_hash: Option<u64>,
}

impl ChangeDetectionState {
    /// Creates a new empty state.
    pub fn new() -> Self {
        Self {
            last_text: None,
            last_image_hash: None,
        }
    }

    /// Checks if text has changed and updates state if it has.
    /// Returns true if the text is new/different.
    pub fn has_text_changed(&mut self, text: &str) -> bool {
        if Some(text) != self.last_text.as_deref() {
            self.last_text = Some(text.to_string());
            true
        } else {
            false
        }
    }

    /// Checks if image hash has changed and updates state if it has.
    /// Returns true if the image is new/different.
    pub fn has_image_changed(&mut self, hash: u64) -> bool {
        if Some(hash) != self.last_image_hash {
            self.last_image_hash = Some(hash);
            true
        } else {
            false
        }
    }
}

impl Default for ChangeDetectionState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_change_detection() {
        let mut state = ChangeDetectionState::new();
        assert!(state.has_text_changed("hello"));
        assert!(!state.has_text_changed("hello"));
        assert!(state.has_text_changed("world"));
    }

    #[test]
    fn test_image_change_detection() {
        let mut state = ChangeDetectionState::new();
        assert!(state.has_image_changed(12345));
        assert!(!state.has_image_changed(12345));
        assert!(state.has_image_changed(67890));
    }
}
