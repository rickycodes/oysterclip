use serde::Deserialize;
use std::fs;
use std::path::Path;

use crate::constants::{CONFIG_FILE, MAX_HISTORY_ENTRIES};

pub(crate) struct WatcherConfig {
    pub(crate) max_history_entries: usize,
    pub(crate) save_images_to_disk: bool,
    pub(crate) image_export_dir: String,
}

#[derive(Deserialize)]
struct RawWatcherConfig {
    max_history_entries: Option<usize>,
    save_images_to_disk: Option<bool>,
    image_export_dir: Option<String>,
}

pub(crate) fn load_config() -> WatcherConfig {
    fs::read_to_string(Path::new(CONFIG_FILE))
        .ok()
        .and_then(|contents| toml::from_str::<RawWatcherConfig>(&contents).ok())
        .map(WatcherConfig::from_raw)
        .unwrap_or_default()
}

impl WatcherConfig {
    fn from_raw(raw: RawWatcherConfig) -> Self {
        Self {
            max_history_entries: raw
                .max_history_entries
                .filter(|value| *value > 0)
                .map(|value| value.min(MAX_HISTORY_ENTRIES))
                .unwrap_or(MAX_HISTORY_ENTRIES),
            save_images_to_disk: raw.save_images_to_disk.unwrap_or(false),
            image_export_dir: raw
                .image_export_dir
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| "clipboard_images".to_string()),
        }
    }
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            max_history_entries: MAX_HISTORY_ENTRIES,
            save_images_to_disk: false,
            image_export_dir: "clipboard_images".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{RawWatcherConfig, WatcherConfig};
    use crate::constants::MAX_HISTORY_ENTRIES;

    #[test]
    fn watcher_config_reads_max_history_entries() {
        let config: RawWatcherConfig = toml::from_str("max_history_entries = 250").unwrap();
        let config = WatcherConfig::from_raw(config);
        assert_eq!(config.max_history_entries, 250);
    }

    #[test]
    fn watcher_config_defaults_image_export_settings() {
        let config = WatcherConfig::from_raw(RawWatcherConfig {
            max_history_entries: None,
            save_images_to_disk: None,
            image_export_dir: None,
        });

        assert_eq!(config.max_history_entries, MAX_HISTORY_ENTRIES);
        assert!(!config.save_images_to_disk);
        assert_eq!(config.image_export_dir, "clipboard_images");
    }

    #[test]
    fn watcher_config_reads_image_export_settings() {
        let config: RawWatcherConfig =
            toml::from_str("save_images_to_disk = true\nimage_export_dir = \"exports\"").unwrap();
        let config = WatcherConfig::from_raw(config);

        assert!(config.save_images_to_disk);
        assert_eq!(config.image_export_dir, "exports");
    }
}
