mod app;
mod ui;
mod data;
mod config;
mod system;

use app::App;
use dioxus::desktop::tao::window::WindowBuilder;
use dioxus::desktop::Config as DesktopConfig;
use dioxus::prelude::LaunchBuilder;

fn main() {
    config::parse();
    LaunchBuilder::desktop()
        .with_cfg(
            DesktopConfig::new()
                .with_window(WindowBuilder::new().with_title("Clipboard Viewer"))
                .with_menu(None),
        )
        .launch(App);
}
