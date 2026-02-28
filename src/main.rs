mod app;
mod common;

use app::App;
use dioxus::prelude::LaunchBuilder;

fn main() {
    LaunchBuilder::desktop()
        .launch(App);
}
