use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

#[cfg(unix)]
use std::os::unix::net::UnixStream;

use super::server::control_socket_path;
use super::ControlResponse;

#[derive(Debug, Serialize, Deserialize)]
struct ControlRequest {
    cmd: String,
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
