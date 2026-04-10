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
        // Priority 1: CLI arg takes priority over config file, but doesn't persist
        if let Some(t) = &crate::config::cli::args().theme {
            return if t == THEME_LIGHT {
                Theme::Light
            } else {
                Theme::Dark
            };
        }

        // Priority 2: User config file setting
        let config = crate::config::AppConfig::load();
        if let Some(mode) = &config.theme.mode {
            if mode == THEME_LIGHT {
                return Theme::Light;
            } else if mode == THEME_DARK {
                return Theme::Dark;
            }
        }

        // Priority 3: OS appearance setting
        let detected = dark_light::detect();
        match detected {
            dark_light::Mode::Dark => return Theme::Dark,
            dark_light::Mode::Light => return Theme::Light,
            dark_light::Mode::Default => {
                // Fallback: Try to detect GNOME settings directly
                if let Ok(output) = std::process::Command::new("gsettings")
                    .args(&["get", "org.gnome.desktop.interface", "color-scheme"])
                    .output()
                {
                    let result = String::from_utf8_lossy(&output.stdout);
                    if result.contains("prefer-light") {
                        return Theme::Light;
                    } else if result.contains("prefer-dark") {
                        return Theme::Dark;
                    }
                }
            }
        }
    }

    // Priority 4: Default to Dark
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
        config.theme.mode = Some(if theme == Theme::Light {
            THEME_LIGHT.to_string()
        } else {
            THEME_DARK.to_string()
        });
        config.save();
    }
}

/// Detect only the OS theme preference (pure detection, no config/CLI checks).
/// Used internally by the polling loop to detect OS changes.
/// Note: The polling loop in App ensures CLI args and config take precedence—
/// this function only runs if both are unset (None).
pub fn detect_os_theme() -> Theme {
    #[cfg(target_arch = "wasm32")]
    {
        Theme::Dark
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let detected = dark_light::detect();
        match detected {
            dark_light::Mode::Dark => Theme::Dark,
            dark_light::Mode::Light => Theme::Light,
            dark_light::Mode::Default => {
                // Fallback: Try to detect GNOME settings directly
                if let Ok(output) = std::process::Command::new("gsettings")
                    .args(&["get", "org.gnome.desktop.interface", "color-scheme"])
                    .output()
                {
                    let result = String::from_utf8_lossy(&output.stdout);
                    if result.contains("prefer-light") {
                        return Theme::Light;
                    } else if result.contains("prefer-dark") {
                        return Theme::Dark;
                    }
                }
                Theme::Dark
            }
        }
    }
}
