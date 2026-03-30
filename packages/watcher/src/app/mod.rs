pub mod error;

use crate::config::{Cli, Commands, ControlAction};
use crate::config::settings::load_config;
use crate::config::paths::{ensure_app_dir, resolve_app_paths};
use crate::config::constants::STARTUP_MESSAGE;
use crate::history::HistoryStore;
use crate::history::current_timestamp;
use crate::ipc::{new_control_state, print_control_response, send_control_command, start_control_server};
use crate::watcher::start_watching;
use error::{AppError, Result};

const INTERVAL_MS: u64 = 500;

pub fn run(cli: Cli) -> Result<()> {
    let app_paths = resolve_app_paths()?;
    ensure_app_dir(&app_paths)?;

    if let Some(command) = cli.command {
        handle_command(command, &app_paths)?;
        return Ok(());
    }

    let config = load_config(&app_paths.config_path, &app_paths.image_dir);
    let history_store = HistoryStore::open(&app_paths.db_path, config.max_history_entries)
        .map_err(|err| AppError::HistoryDbFailed(err.to_string()))?;

    println!("{STARTUP_MESSAGE} - interval: {INTERVAL_MS}ms");
    println!("History DB: {}", app_paths.db_path.display());

    let control_state = new_control_state(
        &app_paths.db_path,
        &config.image_export_dir,
        current_timestamp(),
    );

    let _control_guard = start_control_server(control_state.clone(), &app_paths.db_path)
        .map_err(|err| AppError::ControlSocketFailed(err.to_string()))?;

    start_watching(
        history_store,
        control_state,
        config.save_images_to_disk,
        &config.image_export_dir,
    ).map_err(AppError::IoError)?;

    Ok(())
}

fn handle_command(command: Commands, app_paths: &crate::config::paths::AppPaths) -> Result<()> {
    match command {
        Commands::Version => {
            println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Commands::Control(control) => {
            let cmd = match control.action {
                ControlAction::Pause => "pause",
                ControlAction::Resume => "resume",
                ControlAction::Status => "status",
            };
            send_control_command(&app_paths.db_path, cmd)
                .map(|response| print_control_response(&response))
                .map_err(|err| AppError::ControlSocketFailed(err.to_string()))
        }
        Commands::Watch(args) => {
            let config = load_config(&app_paths.config_path, &app_paths.image_dir);
            let history_store = HistoryStore::open(&app_paths.db_path, config.max_history_entries)
                .map_err(|err| AppError::HistoryDbFailed(err.to_string()))?;

            println!("{STARTUP_MESSAGE} - interval: {INTERVAL_MS}ms");
            println!("History DB: {}", app_paths.db_path.display());

            let control_state = new_control_state(
                &app_paths.db_path,
                &config.image_export_dir,
                current_timestamp(),
            );

            if args.paused {
                if let Ok(mut guard) = control_state.lock() {
                    guard.paused = true;
                }
            }

            let _control_guard = start_control_server(control_state.clone(), &app_paths.db_path)
                .map_err(|err| AppError::ControlSocketFailed(err.to_string()))?;

            start_watching(
                history_store,
                control_state,
                config.save_images_to_disk,
                &config.image_export_dir,
            ).map_err(AppError::IoError)?;

            Ok(())
        }
    }
}
