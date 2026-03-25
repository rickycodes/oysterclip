use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::constants::CONTROL_SOCKET_FILE;

#[cfg(unix)]
use std::os::unix::net::{UnixListener, UnixStream};

#[derive(Debug, Clone)]
pub(crate) struct ControlState {
    pub(crate) paused: bool,
    pub(crate) started_at: u64,
    pub(crate) last_capture_at: Option<u64>,
    pub(crate) last_error: Option<String>,
    pub(crate) db_path: String,
    pub(crate) image_dir: String,
}

pub(crate) type SharedControlState = Arc<Mutex<ControlState>>;

#[derive(Debug, Serialize, Deserialize)]
struct ControlRequest {
    cmd: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ControlResponse {
    pub(crate) ok: bool,
    pub(crate) message: String,
    pub(crate) paused: bool,
    pub(crate) started_at: u64,
    pub(crate) last_capture_at: Option<u64>,
    pub(crate) last_error: Option<String>,
    pub(crate) db_path: String,
    pub(crate) image_dir: String,
}

pub(crate) struct ControlSocketGuard {
    socket_path: PathBuf,
}

impl Drop for ControlSocketGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.socket_path);
    }
}

pub(crate) fn new_control_state(db_path: &Path, image_dir: &Path, started_at: u64) -> SharedControlState {
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
pub(crate) fn start_control_server(state: SharedControlState, db_path: &Path) -> io::Result<ControlSocketGuard> {
    let socket_path = control_socket_path(db_path)?;

    if socket_path.exists() {
        match UnixStream::connect(&socket_path) {
            Ok(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::AddrInUse,
                    format!(
                        "control socket already active at {}",
                        socket_path.display()
                    ),
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
            "status" => build_response(&guard, true, "status"),
            "pause" => {
                guard.paused = true;
                build_response(&guard, true, "watcher paused")
            }
            "resume" => {
                guard.paused = false;
                build_response(&guard, true, "watcher resumed")
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

#[cfg(unix)]
pub(crate) fn send_control_command(db_path: &Path, cmd: &str) -> io::Result<ControlResponse> {
    let socket_path = control_socket_path(db_path)?;
    let mut stream = UnixStream::connect(&socket_path).map_err(|err| {
        io::Error::new(
            err.kind(),
            format!(
                "failed to connect to watcher control socket at {}: {err}",
                socket_path.display()
            ),
        )
    })?;

    let request = ControlRequest {
        cmd: cmd.to_string(),
    };
    serde_json::to_writer(&mut stream, &request).map_err(io::Error::other)?;
    stream.write_all(b"\n")?;
    stream.flush()?;

    let mut response_line = String::new();
    BufReader::new(stream).read_line(&mut response_line)?;
    serde_json::from_str(response_line.trim()).map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid control response: {err}"),
        )
    })
}

#[cfg(not(unix))]
pub(crate) fn send_control_command(_db_path: &Path, _cmd: &str) -> io::Result<ControlResponse> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "control commands are only implemented for unix targets right now",
    ))
}

pub(crate) fn print_control_response(response: &ControlResponse) {
    println!("ok: {}", response.ok);
    println!("message: {}", response.message);
    println!("paused: {}", response.paused);
    println!("db_path: {}", response.db_path);
    println!("image_dir: {}", response.image_dir);
    println!("started_at: {}", response.started_at);
    match response.last_capture_at {
        Some(value) => println!("last_capture_at: {}", value),
        None => println!("last_capture_at: none"),
    }
    match &response.last_error {
        Some(value) => println!("last_error: {}", value),
        None => println!("last_error: none"),
    }
}

fn control_socket_path(db_path: &Path) -> io::Result<PathBuf> {
    let absolute_db_path = if db_path.is_absolute() {
        db_path.to_path_buf()
    } else {
        std::env::current_dir()?.join(db_path)
    };

    let parent = absolute_db_path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("database path has no parent: {}", absolute_db_path.display()),
        )
    })?;

    Ok(parent.join(CONTROL_SOCKET_FILE))
}
