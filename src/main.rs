mod app;
mod common;

use app::App;
use dioxus::desktop::Config as DesktopConfig;
use dioxus::desktop::tao::window::WindowBuilder;
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
