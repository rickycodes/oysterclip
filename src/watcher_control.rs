use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use crate::source::ClipboardSource;

const CONTROL_SOCKET_FILE: &str = ".clipboard-watcher.sock";

#[derive(Clone, PartialEq)]
pub struct WatcherStatus {
    pub available: bool,
    pub paused: bool,
    pub label: String,
    pub detail: String,
}

impl WatcherStatus {
    pub fn unavailable(detail: impl Into<String>) -> Self {
        Self {
            available: false,
            paused: false,
            label: "Watcher offline".to_string(),
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ControlRequest {
    cmd: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ControlResponse {
    ok: bool,
    message: String,
    paused: bool,
    started_at: u64,
    last_capture_at: Option<u64>,
    last_error: Option<String>,
    db_path: String,
    image_dir: String,
}

pub fn get_status(source: &ClipboardSource) -> WatcherStatus {
    match send_control_command(source, "status") {
        Ok(response) => from_response(response),
        Err(err) => WatcherStatus::unavailable(err.to_string()),
    }
}

pub fn pause(source: &ClipboardSource) -> Result<WatcherStatus, String> {
    send_control_command(source, "pause")
        .map(from_response)
        .map_err(|err| err.to_string())
}

pub fn resume(source: &ClipboardSource) -> Result<WatcherStatus, String> {
    send_control_command(source, "resume")
        .map(from_response)
        .map_err(|err| err.to_string())
}

fn from_response(response: ControlResponse) -> WatcherStatus {
    let label = if response.paused {
        "Watcher paused"
    } else {
        "Watcher running"
    };

    let detail = match response.last_error {
        Some(last_error) if !last_error.is_empty() => {
            format!("{} | last error: {}", response.message, last_error)
        }
        _ => response.message,
    };

    WatcherStatus {
        available: response.ok,
        paused: response.paused,
        label: label.to_string(),
        detail,
    }
}

fn send_control_command(source: &ClipboardSource, cmd: &str) -> io::Result<ControlResponse> {
    let db_path = source.file_path().map_err(io::Error::other)?;
    let socket_path = control_socket_path(db_path)?;

    #[cfg(unix)]
    {
        use std::os::unix::net::UnixStream;

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
    {
        let _ = socket_path;
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "watcher control is only implemented for unix targets right now",
        ))
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
