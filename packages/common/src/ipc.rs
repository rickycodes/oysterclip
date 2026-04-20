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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
    Ok(app_paths.base_dir.join(SOCKET_FILE.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_command_status_as_str() {
        assert_eq!(ControlCommand::Status.as_str(), "status");
    }

    #[test]
    fn test_control_command_pause_as_str() {
        assert_eq!(ControlCommand::Pause.as_str(), "pause");
    }

    #[test]
    fn test_control_command_resume_as_str() {
        assert_eq!(ControlCommand::Resume.as_str(), "resume");
    }

    #[test]
    fn test_control_command_parse_status() {
        assert_eq!(
            ControlCommand::parse("status"),
            Some(ControlCommand::Status)
        );
    }

    #[test]
    fn test_control_command_parse_pause() {
        assert_eq!(ControlCommand::parse("pause"), Some(ControlCommand::Pause));
    }

    #[test]
    fn test_control_command_parse_resume() {
        assert_eq!(
            ControlCommand::parse("resume"),
            Some(ControlCommand::Resume)
        );
    }

    #[test]
    fn test_control_command_parse_invalid() {
        assert_eq!(ControlCommand::parse("invalid"), None);
        assert_eq!(ControlCommand::parse(""), None);
        assert_eq!(ControlCommand::parse("STATUS"), None);
    }

    #[test]
    fn test_control_request_new_status() {
        let req = ControlRequest::new(ControlCommand::Status);
        assert_eq!(req.cmd, "status");
    }

    #[test]
    fn test_control_request_new_pause() {
        let req = ControlRequest::new(ControlCommand::Pause);
        assert_eq!(req.cmd, "pause");
    }

    #[test]
    fn test_control_request_new_resume() {
        let req = ControlRequest::new(ControlCommand::Resume);
        assert_eq!(req.cmd, "resume");
    }

    #[test]
    fn test_control_request_serialization() {
        let req = ControlRequest::new(ControlCommand::Pause);
        let json = serde_json::to_string(&req).unwrap();
        let deserialized: ControlRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.cmd, "pause");
    }

    #[test]
    fn test_control_response_serialization() {
        let resp = ControlResponse {
            ok: true,
            message: "test".to_string(),
            paused: false,
            started_at: 1000,
            last_capture_at: Some(2000),
            last_error: None,
            db_path: "/tmp/test.db".to_string(),
            image_dir: "/tmp/images".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: ControlResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.ok, true);
        assert_eq!(deserialized.message, "test");
        assert_eq!(deserialized.paused, false);
        assert_eq!(deserialized.started_at, 1000);
    }

    #[test]
    fn test_control_response_with_error() {
        let resp = ControlResponse {
            ok: false,
            message: "failed".to_string(),
            paused: true,
            started_at: 1000,
            last_capture_at: None,
            last_error: Some("file not found".to_string()),
            db_path: "/tmp/test.db".to_string(),
            image_dir: "/tmp/images".to_string(),
        };
        assert_eq!(resp.ok, false);
        assert_eq!(resp.last_error, Some("file not found".to_string()));
    }

    #[test]
    fn test_socket_path_returns_path() {
        let result = socket_path();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(!path.as_os_str().is_empty());
    }
}
