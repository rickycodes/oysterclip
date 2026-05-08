use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

use super::constants::MAX_HISTORY_ENTRIES;

#[derive(Debug, Clone, Default)]
pub(crate) struct StorageExclusions {
    values: Vec<String>,
}

pub(crate) struct WatcherConfig {
    pub(crate) max_history_entries: usize,
    pub(crate) save_images_to_disk: bool,
    pub(crate) image_export_dir: PathBuf,
    pub(crate) storage_exclusions: StorageExclusions,
}

#[derive(Deserialize, Default)]
struct RawWatcherConfig {
    #[serde(default)]
    max_history_entries: Option<usize>,
    #[serde(default)]
    save_images_to_disk: Option<bool>,
    #[serde(default)]
    image_export_dir: Option<String>,
    #[serde(default)]
    storage: RawStorageConfig,
}

#[derive(Deserialize, Default)]
struct RawStorageConfig {
    #[serde(default)]
    exclude: Vec<String>,
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
            storage_exclusions: StorageExclusions::from_raw(raw.storage.exclude),
        }
    }

    fn default(default_image_dir: &Path) -> Self {
        Self {
            max_history_entries: MAX_HISTORY_ENTRIES,
            save_images_to_disk: false,
            image_export_dir: default_image_dir.to_path_buf(),
            storage_exclusions: StorageExclusions::default(),
        }
    }
}

impl StorageExclusions {
    fn from_raw(values: Vec<String>) -> Self {
        let mut normalized = Vec::new();

        for value in values {
            let value = value.trim().to_lowercase();
            if !value.is_empty() && !normalized.iter().any(|existing| existing == &value) {
                normalized.push(value);
            }
        }

        Self { values: normalized }
    }

    pub(crate) fn excludes_image(&self) -> bool {
        self.contains("image")
    }

    pub(crate) fn excludes_password(&self) -> bool {
        self.contains("password")
    }

    fn contains(&self, value: &str) -> bool {
        self.values.iter().any(|excluded| excluded == value)
    }
}

#[cfg(test)]
mod tests {
    use super::{RawStorageConfig, RawWatcherConfig, WatcherConfig};
    use crate::config::constants::MAX_HISTORY_ENTRIES;
    use common::IMAGE_DIR;
    use std::path::Path;

    #[test]
    fn watcher_config_reads_max_history_entries() {
        let config: RawWatcherConfig = toml::from_str("max_history_entries = 250").unwrap();
        let config =
            WatcherConfig::from_raw(config, Path::new("/tmp/config-dir"), Path::new(IMAGE_DIR));
        assert_eq!(config.max_history_entries, 250);
    }

    #[test]
    fn watcher_config_defaults_image_export_settings() {
        let config = WatcherConfig::from_raw(
            RawWatcherConfig {
                max_history_entries: None,
                save_images_to_disk: None,
                image_export_dir: None,
                storage: RawStorageConfig::default(),
            },
            Path::new("/tmp/config-dir"),
            Path::new(IMAGE_DIR),
        );

        assert_eq!(config.max_history_entries, MAX_HISTORY_ENTRIES);
        assert!(!config.save_images_to_disk);
        assert_eq!(config.image_export_dir, Path::new(IMAGE_DIR));
        assert!(!config.storage_exclusions.excludes_image());
        assert!(!config.storage_exclusions.excludes_password());
    }

    #[test]
    fn watcher_config_resolves_relative_image_export_dir_from_config_dir() {
        let config: RawWatcherConfig =
            toml::from_str("save_images_to_disk = true\nimage_export_dir = \"exports\"").unwrap();
        let config =
            WatcherConfig::from_raw(config, Path::new("/tmp/config-dir"), Path::new(IMAGE_DIR));

        assert!(config.save_images_to_disk);
        assert_eq!(
            config.image_export_dir,
            Path::new("/tmp/config-dir/exports")
        );
    }

    #[test]
    fn watcher_config_clamps_max_history_entries() {
        let config: RawWatcherConfig = toml::from_str("max_history_entries = 9999").unwrap();
        let config =
            WatcherConfig::from_raw(config, Path::new("/tmp/config-dir"), Path::new(IMAGE_DIR));

        assert_eq!(config.max_history_entries, MAX_HISTORY_ENTRIES);
    }

    #[test]
    fn watcher_config_ignores_zero_max_history_entries() {
        let config: RawWatcherConfig = toml::from_str("max_history_entries = 0").unwrap();
        let config =
            WatcherConfig::from_raw(config, Path::new("/tmp/config-dir"), Path::new(IMAGE_DIR));

        assert_eq!(config.max_history_entries, MAX_HISTORY_ENTRIES);
    }

    #[test]
    fn watcher_config_handles_whitespace_in_image_export_dir() {
        let config: RawWatcherConfig =
            toml::from_str("image_export_dir = \"  exports  \"").unwrap();
        let config =
            WatcherConfig::from_raw(config, Path::new("/tmp/config-dir"), Path::new(IMAGE_DIR));

        assert_eq!(
            config.image_export_dir,
            Path::new("/tmp/config-dir/exports")
        );
    }

    #[test]
    fn watcher_config_uses_absolute_image_export_dir() {
        let config: RawWatcherConfig =
            toml::from_str("save_images_to_disk = true\nimage_export_dir = \"/absolute/path\"")
                .unwrap();
        let config =
            WatcherConfig::from_raw(config, Path::new("/tmp/config-dir"), Path::new(IMAGE_DIR));

        assert_eq!(config.image_export_dir, Path::new("/absolute/path"));
    }

    #[test]
    fn watcher_config_empty_image_export_dir_uses_default() {
        let config: RawWatcherConfig = toml::from_str("image_export_dir = \"\"").unwrap();
        let config =
            WatcherConfig::from_raw(config, Path::new("/tmp/config-dir"), Path::new(IMAGE_DIR));

        assert_eq!(config.image_export_dir, Path::new(IMAGE_DIR));
    }

    #[test]
    fn watcher_config_false_save_images_to_disk() {
        let config: RawWatcherConfig = toml::from_str("save_images_to_disk = false").unwrap();
        let config =
            WatcherConfig::from_raw(config, Path::new("/tmp/config-dir"), Path::new(IMAGE_DIR));

        assert!(!config.save_images_to_disk);
    }

    #[test]
    fn watcher_config_reads_storage_exclusions() {
        let config: RawWatcherConfig =
            toml::from_str("[storage]\nexclude = [\"image\", \"password\"]").unwrap();
        let config =
            WatcherConfig::from_raw(config, Path::new("/tmp/config-dir"), Path::new(IMAGE_DIR));

        assert!(config.storage_exclusions.excludes_image());
        assert!(config.storage_exclusions.excludes_password());
    }

    #[test]
    fn watcher_config_normalizes_storage_exclusions() {
        let config = WatcherConfig::from_raw(
            RawWatcherConfig {
                max_history_entries: None,
                save_images_to_disk: None,
                image_export_dir: None,
                storage: RawStorageConfig {
                    exclude: vec![
                        " Image ".to_string(),
                        "PASSWORD".to_string(),
                        "".to_string(),
                    ],
                },
            },
            Path::new("/tmp/config-dir"),
            Path::new(IMAGE_DIR),
        );

        assert!(config.storage_exclusions.excludes_image());
        assert!(config.storage_exclusions.excludes_password());
    }
}
