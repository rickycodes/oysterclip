mod poll;
mod signal;
mod state;

pub use poll::poll_clipboard;
pub use signal::setup_signal_handler;

use crate::history::HistoryStore;
use crate::ipc::SharedControlState;

/// Starts the clipboard watcher with signal handling.
///
/// This orchestrates:
/// - Signal handler setup for graceful shutdown
/// - Main clipboard polling loop
pub fn start_watching(
    history_store: HistoryStore,
    control_state: SharedControlState,
    save_images_to_disk: bool,
    image_export_dir: &std::path::Path,
) -> std::io::Result<()> {
    let shutdown = setup_signal_handler()?;
    poll_clipboard(
        history_store,
        control_state,
        save_images_to_disk,
        image_export_dir,
        &shutdown,
    )
}
