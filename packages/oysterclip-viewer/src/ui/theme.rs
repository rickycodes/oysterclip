#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Theme {
    Dark,
    Light,
}

use common::{THEME_DARK, THEME_LIGHT};

impl Theme {
    pub fn class_name(&self) -> &'static str {
        match self {
            Theme::Dark => "",
            Theme::Light => "light-mode",
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Theme::Dark => "Dark",
            Theme::Light => "Light",
        }
    }
}

pub fn load_theme() -> Theme {
    #[cfg(target_arch = "wasm32")]
    {
        if let Ok(Some(stored)) = web_sys::window()
            .and_then(|w| w.local_storage().ok())
            .and_then(|s| s.and_then(|storage| storage.get_item("theme").ok()))
        {
            if stored == THEME_LIGHT {
                return Theme::Light;
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // CLI arg takes priority over config file, but doesn't persist
        if let Some(t) = &crate::config::cli::args().theme {
            return if t == THEME_LIGHT {
                Theme::Light
            } else {
                Theme::Dark
            };
        }

        if crate::config::AppConfig::load().theme.mode == THEME_LIGHT {
            return Theme::Light;
        }
    }

    Theme::Dark
}

pub fn save_theme(theme: Theme) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Ok(Some(storage)) = web_sys::window().and_then(|w| w.local_storage().ok()) {
            let _ = storage.set_item(
                "theme",
                if theme == Theme::Light {
                    THEME_LIGHT
                } else {
                    THEME_DARK
                },
            );
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut config = crate::config::AppConfig::load();
        config.theme.mode = if theme == Theme::Light {
            THEME_LIGHT
        } else {
            THEME_DARK
        }
        .to_string();
        config.save();
    }
}
