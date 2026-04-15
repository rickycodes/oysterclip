use serde::{Deserialize, Serialize};

use common::classification::PASSWORD_LEN;
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    #[serde(default)]
    pub theme: ThemeConfig,
    #[serde(default)]
    pub bulk_actions: BulkActionsConfig,
    #[serde(default)]
    pub password: PasswordConfig,
    #[serde(default)]
    pub watcher: WatcherSettingsConfig,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct BulkActionsConfig {
    #[serde(default)]
    pub handlers: std::collections::HashMap<String, HandlerConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HandlerConfig {
    pub handler_type: String,
    pub command: Option<String>,
    pub app: Option<String>,
    pub target: Option<String>,
    pub separator: Option<String>,
    pub template: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ThemeConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PasswordConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub len: Option<usize>,
    #[serde(default = "PasswordConfig::default_score_threshold")]
    pub score_threshold: u8,
}

impl Default for PasswordConfig {
    fn default() -> Self {
        Self {
            len: Some(Self::default_len()),
            score_threshold: Self::default_score_threshold(),
        }
    }
}

impl PasswordConfig {
    const fn default_len() -> usize {
        PASSWORD_LEN
    }

    const fn default_score_threshold() -> u8 {
        3 // Score::Three from zxcvbn
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct WatcherSettingsConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub save_images_to_disk: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_export_dir: Option<String>,
}

impl AppConfig {
    pub fn load() -> Self {
        let Ok(path) = crate::config::paths::config_path() else {
            return Self::default();
        };
        let Ok(text) = std::fs::read_to_string(&path) else {
            return Self::default();
        };
        let mut config: AppConfig = toml::from_str(&text).unwrap_or_default();

        // Apply CLI overrides
        let cli_args = crate::config::cli::args();
        if let Some(len) = cli_args.password_len {
            config.password.len = Some(len);
        }
        if let Some(score) = cli_args.password_score_threshold {
            config.password.score_threshold = score;
        }

        config
    }

    pub fn save(&self) {
        let Ok(path) = crate::config::paths::config_path() else {
            return;
        };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(text) = toml::to_string_pretty(self) {
            let _ = std::fs::write(&path, text);
        }
    }

    pub fn get_handler(&self, name: &str) -> Option<HandlerConfig> {
        self.bulk_actions.handlers.get(name).cloned()
    }
}
