mod app;
mod app_actions;
mod app_state;
mod auth;
mod components;
mod entry;
mod format;
mod help_modal;
mod history;
mod link_preview;
mod paths;
mod source;
mod theme;
mod watcher_control;

use app::App;
use dioxus::desktop::tao::window::WindowBuilder;
use dioxus::desktop::Config as DesktopConfig;
use dioxus::prelude::LaunchBuilder;

fn main() {
    LaunchBuilder::desktop()
        .with_cfg(
            DesktopConfig::new()
                .with_window(WindowBuilder::new().with_title("Clipboard Viewer"))
                .with_menu(None),
        )
        .launch(App);
}
