use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub theme: ThemeConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub mode: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            mode: "dark".to_string(),
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        let Ok(path) = crate::config::paths::config_path() else {
            return Self::default();
        };
        let Ok(text) = std::fs::read_to_string(&path) else {
            return Self::default();
        };
        toml::from_str(&text).unwrap_or_default()
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
}
