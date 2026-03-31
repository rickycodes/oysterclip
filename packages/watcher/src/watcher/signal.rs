use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Sets up graceful shutdown signal handlers (Unix only).
/// Returns an Arc to the shutdown flag that can be shared across threads.
pub fn setup_signal_handler() -> std::io::Result<Arc<AtomicBool>> {
    let shutdown = Arc::new(AtomicBool::new(false));

    #[cfg(unix)]
    {
        let shutdown_clone = shutdown.clone();
        let mut signals = signal_hook::iterator::Signals::new([
            signal_hook::consts::signal::SIGTERM,
            signal_hook::consts::signal::SIGINT,
        ])?;
        std::thread::spawn(move || {
            for _ in signals.forever() {
                shutdown_clone.store(true, Ordering::SeqCst);
            }
        });
    }

    Ok(shutdown)
}

/// Checks if shutdown signal has been received.
pub fn should_shutdown(shutdown: &Arc<AtomicBool>) -> bool {
    shutdown.load(Ordering::SeqCst)
}
