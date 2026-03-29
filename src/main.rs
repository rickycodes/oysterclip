use clap::Parser;

mod cli;
mod config;
mod data;
mod history;
mod ipc;
mod watcher;

use crate::cli::{Cli, Commands, ControlAction};
use crate::config::config::load_config;
use crate::config::paths::{ensure_app_dir, resolve_app_paths};
use crate::config::constants::{
    OPEN_HISTORY_STORE_FAILED, STARTUP_MESSAGE, INTERVAL_MS,
};
use crate::history::HistoryStore;
use crate::history::current_timestamp;
use crate::ipc::{
    new_control_state, print_control_response, send_control_command, start_control_server,
};
use crate::watcher::start_watching;

fn main() {
    let cli = Cli::parse();
    let app_paths = resolve_app_paths().unwrap_or_else(|err| {
        eprintln!("Failed to resolve application storage paths: {err}");
        std::process::exit(1);
    });
    ensure_app_dir(&app_paths).unwrap_or_else(|err| {
        eprintln!("Failed to create application storage directory: {err}");
        std::process::exit(1);
    });

    let mut start_paused = false;

    match cli.command {
        Some(Commands::Version) => {
            println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            return;
        }
        Some(Commands::Control(control)) => {
            let cmd = match control.action {
                ControlAction::Pause => "pause",
                ControlAction::Resume => "resume",
                ControlAction::Status => "status",
            };
            run_control_command(&app_paths.db_path, cmd);
            return;
        }
        Some(Commands::Watch(args)) => {
            start_paused = args.paused;
        }
        None => {}
    }

    println!("{STARTUP_MESSAGE} - interval: {INTERVAL_MS}ms");
    println!("History DB: {}", app_paths.db_path.display());
    let config = load_config(&app_paths.config_path, &app_paths.image_dir);
    let history_store =
        HistoryStore::open(&app_paths.db_path, config.max_history_entries).unwrap_or_else(|err| {
            eprintln!("{OPEN_HISTORY_STORE_FAILED}: {err}");
            std::process::exit(1);
        });

    let control_state = new_control_state(
        &app_paths.db_path,
        &config.image_export_dir,
        current_timestamp(),
    );
    if start_paused {
        if let Ok(mut guard) = control_state.lock() {
            guard.paused = true;
        }
    }
    let _control_guard =
        start_control_server(control_state.clone(), &app_paths.db_path).unwrap_or_else(|err| {
            eprintln!("Failed to start watcher control socket: {err}");
            std::process::exit(1);
        });

    start_watching(
        history_store,
        control_state,
        config.save_images_to_disk,
        &config.image_export_dir,
    );
}

fn run_control_command(db_path: &std::path::Path, cmd: &str) {
    match send_control_command(db_path, cmd) {
        Ok(response) => print_control_response(&response),
        Err(err) => {
            eprintln!("Failed to send `{cmd}` command: {err}");
            std::process::exit(1);
        }
    }
}

