mod app;
mod common;

use app::App;
use dioxus::desktop::Config as DesktopConfig;
use dioxus::prelude::LaunchBuilder;

fn main() {
    LaunchBuilder::desktop()
        .with_cfg(DesktopConfig::new().with_menu(None))
        .launch(App);
}
