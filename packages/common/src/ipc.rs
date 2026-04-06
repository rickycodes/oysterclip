use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::constants::SOCKET_FILE;
use super::paths;

// Command string constants
const CMD_STATUS: &str = "status";
const CMD_PAUSE: &str = "pause";
const CMD_RESUME: &str = "resume";

// Status message constants
pub const MSG_WATCHER_PAUSED: &str = "Watcher paused";
pub const MSG_WATCHER_RESUMED: &str = "Watcher resumed";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ControlCommand {
    Status,
    Pause,
    Resume,
}

impl ControlCommand {
    pub fn as_str(&self) -> &'static str {
        match self {
            ControlCommand::Status => CMD_STATUS,
            ControlCommand::Pause => CMD_PAUSE,
            ControlCommand::Resume => CMD_RESUME,
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            CMD_STATUS => Some(ControlCommand::Status),
            CMD_PAUSE => Some(ControlCommand::Pause),
            CMD_RESUME => Some(ControlCommand::Resume),
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
