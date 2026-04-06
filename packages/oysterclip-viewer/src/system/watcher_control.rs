use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

use crate::config::source::ClipboardSource;
use common::{ControlCommand, ControlRequest, ControlResponse};

#[derive(Clone, PartialEq)]
pub struct WatcherStatus {
    pub available: bool,
    pub paused: bool,
    pub label: String,
    pub detail: String,
    pub last_capture_at: Option<u64>,
    pub last_error: Option<String>,
}

impl WatcherStatus {
    pub fn unavailable(detail: impl Into<String>) -> Self {
        Self {
            available: false,
            paused: false,
            label: "Watcher offline".to_string(),
            detail: detail.into(),
            last_capture_at: None,
            last_error: None,
        }
    }
}

pub fn get_status(source: &ClipboardSource) -> WatcherStatus {
    match send_control_command(source, ControlCommand::Status) {
        Ok(response) => from_response(response),
        Err(err) => WatcherStatus::unavailable(err.to_string()),
    }
}

pub fn pause(source: &ClipboardSource) -> Result<WatcherStatus, String> {
    send_control_command(source, ControlCommand::Pause)
        .map(from_response)
        .map_err(|err| err.to_string())
}

pub fn resume(source: &ClipboardSource) -> Result<WatcherStatus, String> {
    send_control_command(source, ControlCommand::Resume)
        .map(from_response)
        .map_err(|err| err.to_string())
}

fn from_response(response: ControlResponse) -> WatcherStatus {
    let label = if response.paused {
        "Watcher paused"
    } else {
        "Watcher running"
    };

    WatcherStatus {
        available: response.ok,
        paused: response.paused,
        label: label.to_string(),
        detail: response.message,
        last_capture_at: response.last_capture_at,
        last_error: response.last_error,
    }
}

fn send_control_command(
    source: &ClipboardSource,
    cmd: ControlCommand,
) -> io::Result<ControlResponse> {
    let db_path = source.file_path().map_err(io::Error::other)?;
    let socket_path = get_socket_path(db_path)?;

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

        let request = ControlRequest::new(cmd);
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

/// Get socket path from database path (handles both absolute and relative paths)
fn get_socket_path(db_path: &Path) -> io::Result<std::path::PathBuf> {
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

    Ok(parent.join(".clipboard-watcher.sock"))
}
