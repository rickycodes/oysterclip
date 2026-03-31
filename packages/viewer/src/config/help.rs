#[derive(Clone, Copy, Debug)]
pub struct HelpEntry {
    pub code: &'static str,
    pub description: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct HelpSection {
    pub title: &'static str,
    pub entries: &'static [HelpEntry],
}

pub const HELP_NAVIGATION: &[HelpEntry] = &[
    HelpEntry { code: "↑ / k", description: "Previous entry" },
    HelpEntry { code: "↓ / j", description: "Next entry" },
    HelpEntry { code: "Home", description: "First entry" },
    HelpEntry { code: "End", description: "Last entry" },
];

pub const HELP_SELECTION: &[HelpEntry] = &[
    HelpEntry { code: "Space", description: "Toggle selection" },
    HelpEntry { code: "Shift+↑ / Shift+↓", description: "Extend selection" },
    HelpEntry { code: "Escape", description: "Clear selection" },
];

pub const HELP_ACTIONS: &[HelpEntry] = &[
    HelpEntry { code: "Enter / y", description: "Copy to clipboard" },
    HelpEntry { code: "Delete / Backspace / d", description: "Delete entry (or selection)" },
    HelpEntry { code: "Escape", description: "Close overlay / clear selection / clear search" },
    HelpEntry { code: "p", description: "Pause / resume watcher" },
    HelpEntry { code: "?", description: "Show this help" },
];

pub const HELP_SEARCH: &[HelpEntry] = &[
    HelpEntry { code: "/ or Ctrl+F", description: "Focus search" },
    HelpEntry { code: "type:image", description: "Show only images" },
    HelpEntry { code: "type:text", description: "Show only text entries" },
    HelpEntry { code: "type:password", description: "Show only passwords" },
    HelpEntry { code: "kind:url", description: "Show only URLs" },
    HelpEntry { code: "kind:json", description: "Show only JSON" },
    HelpEntry { code: "kind:path", description: "Show only file paths" },
    HelpEntry { code: "kind:pass", description: "Show only passwords" },
    HelpEntry { code: "since:1h", description: "Last hour (also: 24h, 7d, 30d, today, yesterday)" },
];

pub const HELP_KEYBOARD_SECTIONS: &[HelpSection] = &[
    HelpSection { title: "Navigation", entries: HELP_NAVIGATION },
    HelpSection { title: "Selection", entries: HELP_SELECTION },
    HelpSection { title: "Actions", entries: HELP_ACTIONS },
    HelpSection { title: "Search", entries: HELP_SEARCH },
];

pub mod modal {
    pub const TITLE: &str = "Keyboard Shortcuts";
    pub const CLOSE_LABEL: &str = "Close help";
    pub const FILTER_TIP: &str = "Combine filters with free-text search";

    pub mod controls {
        pub const SECTION_THEME: &str = "Theme";
        pub const SECTION_WATCHER: &str = "Watcher";

        pub mod watcher {
            pub const RESUME: &str = "Resume";
            pub const PAUSE: &str = "Pause";
            pub const UNAVAILABLE: &str = "Unavailable";
            pub const LAST_CAPTURE: &str = "Last capture";
            pub const NO_CAPTURES_YET: &str = "No captures yet";
            pub const WAITING_FOR_STATUS: &str = "Waiting for watcher status";
            pub const LAST_ERROR_PREFIX: &str = "Last error: ";
        }
    }
}
