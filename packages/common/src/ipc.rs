use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::paths;
use super::constants::SOCKET_FILE;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ControlCommand {
    Status,
    Pause,
    Resume,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ControlResponse {
    Status { paused: bool },
    Ok,
    Error(String),
}

pub fn socket_path() -> std::io::Result<PathBuf> {
    let app_paths = paths::resolve_app_paths()?;
    Ok(app_paths.base_dir.join(SOCKET_FILE))
}
