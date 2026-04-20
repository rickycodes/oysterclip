use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::config::constants::SOCKET_FILE;
use common::{ControlRequest, MSG_WATCHER_PAUSED, MSG_WATCHER_RESUMED};

#[cfg(unix)]
use std::os::unix::net::{UnixListener, UnixStream};

use super::{ControlResponse, ControlState, SharedControlState};

pub(crate) struct ControlSocketGuard {
    socket_path: PathBuf,
}

impl Drop for ControlSocketGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.socket_path);
    }
}

pub(crate) fn new_control_state(
    db_path: &Path,
    image_dir: &Path,
    started_at: u64,
) -> SharedControlState {
    Arc::new(Mutex::new(ControlState {
        paused: false,
        started_at,
        last_capture_at: None,
        last_error: None,
        db_path: db_path.display().to_string(),
        image_dir: image_dir.display().to_string(),
    }))
}

#[cfg(unix)]
pub(crate) fn start_control_server(
    state: SharedControlState,
    db_path: &Path,
) -> io::Result<ControlSocketGuard> {
    let socket_path = control_socket_path(db_path)?;

    if socket_path.exists() {
        match UnixStream::connect(&socket_path) {
            Ok(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::AddrInUse,
                    format!("control socket already active at {}", socket_path.display()),
                ));
            }
            Err(_) => {
                let _ = fs::remove_file(&socket_path);
            }
        }
    }

    let listener = UnixListener::bind(&socket_path)?;
    let state_for_thread = state.clone();

    thread::spawn(move || loop {
        match listener.accept() {
            Ok((stream, _)) => {
                if let Err(err) = handle_client(stream, &state_for_thread) {
                    if let Ok(mut guard) = state_for_thread.lock() {
                        guard.last_error = Some(format!("IPC client handling failed: {err}"));
                    }
                }
            }
            Err(err) => {
                if let Ok(mut guard) = state_for_thread.lock() {
                    guard.last_error = Some(format!("IPC accept failed: {err}"));
                }
                thread::sleep(Duration::from_millis(100));
            }
        }
    });

    Ok(ControlSocketGuard { socket_path })
}

#[cfg(not(unix))]
pub(crate) fn start_control_server(
    _state: SharedControlState,
    _db_path: &Path,
) -> io::Result<ControlSocketGuard> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "control server is only implemented for unix targets right now",
    ))
}

#[cfg(unix)]
fn handle_client(stream: UnixStream, state: &SharedControlState) -> io::Result<()> {
    let mut reader = BufReader::new(stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;

    let request: ControlRequest = serde_json::from_str(request_line.trim()).map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid control request: {err}"),
        )
    })?;

    let response = {
        let mut guard = state
            .lock()
            .map_err(|_| io::Error::other("failed to acquire control state lock"))?;
        match request.cmd.as_str() {
            "ping" => build_response(&guard, true, "pong"),
            "status" => {
                let status_msg = if guard.paused {
                    "Watcher is paused"
                } else {
                    "Watcher is running"
                };
                build_response(&guard, true, status_msg)
            }
            "pause" => {
                guard.paused = true;
                build_response(&guard, true, MSG_WATCHER_PAUSED)
            }
            "resume" => {
                guard.paused = false;
                build_response(&guard, true, MSG_WATCHER_RESUMED)
            }
            other => build_response(&guard, false, format!("unknown command: {other}")),
        }
    };

    let mut stream = reader.into_inner();
    serde_json::to_writer(&mut stream, &response).map_err(io::Error::other)?;
    stream.write_all(b"\n")?;
    stream.flush()?;
    Ok(())
}

fn build_response(state: &ControlState, ok: bool, message: impl Into<String>) -> ControlResponse {
    ControlResponse {
        ok,
        message: message.into(),
        paused: state.paused,
        started_at: state.started_at,
        last_capture_at: state.last_capture_at,
        last_error: state.last_error.clone(),
        db_path: state.db_path.clone(),
        image_dir: state.image_dir.clone(),
    }
}

pub(crate) fn control_socket_path(db_path: &Path) -> io::Result<PathBuf> {
    let absolute_db_path = if db_path.is_absolute() {
        db_path.to_path_buf()
    } else {
        std::env::current_dir()?.join(db_path)
    };

    let parent = absolute_db_path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "database path has no parent: {}",
                absolute_db_path.display()
            ),
        )
    })?;

    Ok(parent.join(SOCKET_FILE.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_control_state_initialization() {
        let db_path = Path::new("/tmp/test.db");
        let image_dir = Path::new("/tmp/images");
        let started_at = 123456;

        let state = new_control_state(db_path, image_dir, started_at);
        let guard = state.lock().unwrap();

        assert!(!guard.paused);
        assert_eq!(guard.started_at, 123456);
        assert!(guard.last_capture_at.is_none());
        assert!(guard.last_error.is_none());
    }

    #[test]
    fn test_control_socket_path_absolute() {
        let db_path = Path::new("/home/user/.oysterclip.db");
        let result = control_socket_path(db_path);

        assert!(result.is_ok());
        let socket_path = result.unwrap();
        assert!(socket_path.to_string_lossy().contains("home/user"));
        assert!(socket_path.to_string_lossy().contains(".oysterclip.sock"));
    }

    #[test]
    fn test_control_socket_path_relative() {
        let db_path = Path::new(".oysterclip.db");
        let result = control_socket_path(db_path);

        assert!(result.is_ok());
        let socket_path = result.unwrap();
        assert!(socket_path.to_string_lossy().contains(".oysterclip.sock"));
    }

    #[test]
    fn test_control_socket_path_nested() {
        let db_path = Path::new("/some/deeply/nested/path/.oysterclip.db");
        let result = control_socket_path(db_path);

        assert!(result.is_ok());
        let socket_path = result.unwrap();
        assert!(socket_path.to_string_lossy().contains("deeply"));
        assert!(socket_path.to_string_lossy().ends_with(".oysterclip.sock"));
    }

    #[test]
    fn test_build_response_success() {
        let state = ControlState {
            paused: false,
            started_at: 100,
            last_capture_at: Some(200),
            last_error: None,
            db_path: "/tmp/db".to_string(),
            image_dir: "/tmp/images".to_string(),
        };

        let response = build_response(&state, true, "test message");

        assert!(response.ok);
        assert_eq!(response.message, "test message");
        assert!(!response.paused);
        assert_eq!(response.started_at, 100);
        assert!(response.last_error.is_none());
    }

    #[test]
    fn test_build_response_failure() {
        let state = ControlState {
            paused: true,
            started_at: 100,
            last_capture_at: None,
            last_error: Some("error message".to_string()),
            db_path: "/tmp/db".to_string(),
            image_dir: "/tmp/images".to_string(),
        };

        let response = build_response(&state, false, "failure");

        assert!(!response.ok);
        assert_eq!(response.message, "failure");
        assert!(response.paused);
        assert_eq!(response.last_error, Some("error message".to_string()));
    }

    #[test]
    fn test_control_socket_guard_path() {
        let guard = ControlSocketGuard {
            socket_path: PathBuf::from("/tmp/test.sock"),
        };
        
        // Guard stores path correctly
        assert_eq!(guard.socket_path, PathBuf::from("/tmp/test.sock"));
    }
}
