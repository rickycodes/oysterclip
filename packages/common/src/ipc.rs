use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::constants::SOCKET_FILE;
use super::paths;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ControlCommand {
    Status,
    Pause,
    Resume,
}

impl ControlCommand {
    pub fn as_str(&self) -> &'static str {
        match self {
            ControlCommand::Status => "status",
            ControlCommand::Pause => "pause",
            ControlCommand::Resume => "resume",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "status" => Some(ControlCommand::Status),
            "pause" => Some(ControlCommand::Pause),
            "resume" => Some(ControlCommand::Resume),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ControlRequest {
    pub cmd: String,
}

impl ControlRequest {
    pub fn new(cmd: ControlCommand) -> Self {
        Self {
            cmd: cmd.as_str().to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ControlResponse {
    pub ok: bool,
    pub message: String,
    pub paused: bool,
    pub started_at: u64,
    pub last_capture_at: Option<u64>,
    pub last_error: Option<String>,
    pub db_path: String,
    pub image_dir: String,
}

pub fn socket_path() -> std::io::Result<PathBuf> {
    let app_paths = paths::resolve_app_paths()?;
    Ok(app_paths.base_dir.join(SOCKET_FILE))
}
