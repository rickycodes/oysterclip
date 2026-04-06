use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

use super::constants::MAX_HISTORY_ENTRIES;

pub(crate) struct WatcherConfig {
    pub(crate) max_history_entries: usize,
    pub(crate) save_images_to_disk: bool,
    pub(crate) image_export_dir: PathBuf,
}

#[derive(Deserialize)]
struct RawWatcherConfig {
    max_history_entries: Option<usize>,
    save_images_to_disk: Option<bool>,
    image_export_dir: Option<String>,
}

pub(crate) fn load_config(config_path: &Path, default_image_dir: &Path) -> WatcherConfig {
    fs::read_to_string(config_path)
        .ok()
        .and_then(|contents| toml::from_str::<RawWatcherConfig>(&contents).ok())
        .map(|raw| {
            WatcherConfig::from_raw(
                raw,
                config_path.parent().unwrap_or(default_image_dir),
                default_image_dir,
            )
        })
        .unwrap_or_else(|| WatcherConfig::default(default_image_dir))
}

impl WatcherConfig {
    fn from_raw(raw: RawWatcherConfig, config_dir: &Path, default_image_dir: &Path) -> Self {
        Self {
            max_history_entries: raw
                .max_history_entries
                .filter(|value| *value > 0)
                .map(|value| value.min(MAX_HISTORY_ENTRIES))
                .unwrap_or(MAX_HISTORY_ENTRIES),
            save_images_to_disk: raw.save_images_to_disk.unwrap_or(false),
            image_export_dir: raw
                .image_export_dir
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .map(PathBuf::from)
                .map(|path| {
                    if path.is_absolute() {
                        path
                    } else {
                        config_dir.join(path)
                    }
                })
                .unwrap_or_else(|| default_image_dir.to_path_buf()),
        }
    }

    fn default(default_image_dir: &Path) -> Self {
        Self {
            max_history_entries: MAX_HISTORY_ENTRIES,
            save_images_to_disk: false,
            image_export_dir: default_image_dir.to_path_buf(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{RawWatcherConfig, WatcherConfig};
    use crate::config::constants::MAX_HISTORY_ENTRIES;
    use common::IMAGE_DIR;
    use std::path::Path;

    #[test]
    fn watcher_config_reads_max_history_entries() {
        let config: RawWatcherConfig = toml::from_str("max_history_entries = 250").unwrap();
        let config = WatcherConfig::from_raw(
            config,
            Path::new("/tmp/config-dir"),
            Path::new(IMAGE_DIR),
        );
        assert_eq!(config.max_history_entries, 250);
    }

    #[test]
    fn watcher_config_defaults_image_export_settings() {
        let config = WatcherConfig::from_raw(
            RawWatcherConfig {
                max_history_entries: None,
                save_images_to_disk: None,
                image_export_dir: None,
            },
            Path::new("/tmp/config-dir"),
            Path::new(IMAGE_DIR),
        );

        assert_eq!(config.max_history_entries, MAX_HISTORY_ENTRIES);
        assert!(!config.save_images_to_disk);
        assert_eq!(config.image_export_dir, Path::new(IMAGE_DIR));
    }

    #[test]
    fn watcher_config_resolves_relative_image_export_dir_from_config_dir() {
        let config: RawWatcherConfig =
            toml::from_str("save_images_to_disk = true\nimage_export_dir = \"exports\"").unwrap();
        let config = WatcherConfig::from_raw(
            config,
            Path::new("/tmp/config-dir"),
            Path::new(IMAGE_DIR),
        );

        assert!(config.save_images_to_disk);
        assert_eq!(
            config.image_export_dir,
            Path::new("/tmp/config-dir/exports")
        );
    }
}
