mod app;
mod app_actions;
mod app_state;
mod auth;
mod components;
mod entry;
mod format;
mod history;
mod source;

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
