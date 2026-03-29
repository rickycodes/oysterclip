use clap::Parser;

mod app;
mod config;
mod data;
mod history;
mod ipc;
mod watcher;

use crate::config::Cli;

fn main() {
    let cli = Cli::parse();
    if let Err(err) = app::run(cli) {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}


