use serde::Deserialize;
use std::fs;
use std::path::Path;

use crate::constants::{CONFIG_FILE, MAX_HISTORY_ENTRIES};

#[derive(Deserialize)]
struct WatcherConfig {
    max_history_entries: Option<usize>,
}

pub(crate) fn load_max_history_entries() -> usize {
    fs::read_to_string(Path::new(CONFIG_FILE))
        .ok()
        .and_then(|contents| toml::from_str::<WatcherConfig>(&contents).ok())
        .and_then(|config| config.max_history_entries)
        .filter(|value| *value > 0)
        .map(|value| value.min(MAX_HISTORY_ENTRIES))
        .unwrap_or(MAX_HISTORY_ENTRIES)
}

#[cfg(test)]
mod tests {
    use super::WatcherConfig;

    #[test]
    fn watcher_config_reads_max_history_entries() {
        let config: WatcherConfig = toml::from_str("max_history_entries = 250").unwrap();
        assert_eq!(config.max_history_entries, Some(250));
    }

    #[test]
    fn watcher_config_ignores_unrelated_keys() {
        let config: WatcherConfig =
            toml::from_str("gpg_recipient = \"someone@example.com\"\nmax_history_entries = 250")
                .unwrap();
        assert_eq!(config.max_history_entries, Some(250));
    }
}
