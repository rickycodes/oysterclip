use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
#[allow(dead_code)]
pub enum AppError {
    ConfigNotFound(PathBuf),
    ConfigInvalid(String),
    HistoryDbFailed(String),
    KeychainFailed(String),
    ClipboardUnavailable,
    ControlSocketFailed(String),
    ImageEncodeFailed(String),
    IoError(io::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::ConfigNotFound(path) => {
                write!(f, "config file not found: {}", path.display())
            }
            AppError::ConfigInvalid(msg) => write!(f, "invalid config: {msg}"),
            AppError::HistoryDbFailed(msg) => write!(f, "history database error: {msg}"),
            AppError::KeychainFailed(msg) => write!(f, "keychain access failed: {msg}"),
            AppError::ClipboardUnavailable => write!(f, "clipboard is not available"),
            AppError::ControlSocketFailed(msg) => {
                write!(f, "failed to start control socket: {msg}")
            }
            AppError::ImageEncodeFailed(msg) => write!(f, "failed to encode image: {msg}"),
            AppError::IoError(err) => write!(f, "io error: {err}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        AppError::IoError(err)
    }
}

pub type Result<T> = std::result::Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_error_config_not_found() {
        let path = PathBuf::from("/etc/app.conf");
        let err = AppError::ConfigNotFound(path.clone());
        let msg = format!("{}", err);
        assert!(msg.contains("config file not found"));
        assert!(msg.contains("app.conf"));
    }

    #[test]
    fn test_app_error_config_invalid() {
        let err = AppError::ConfigInvalid("missing key".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("invalid config"));
        assert!(msg.contains("missing key"));
    }

    #[test]
    fn test_app_error_history_db_failed() {
        let err = AppError::HistoryDbFailed("database locked".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("history database error"));
        assert!(msg.contains("database locked"));
    }

    #[test]
    fn test_app_error_keychain_failed() {
        let err = AppError::KeychainFailed("access denied".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("keychain access failed"));
        assert!(msg.contains("access denied"));
    }

    #[test]
    fn test_app_error_clipboard_unavailable() {
        let err = AppError::ClipboardUnavailable;
        let msg = format!("{}", err);
        assert!(msg.contains("clipboard is not available"));
    }

    #[test]
    fn test_app_error_control_socket_failed() {
        let err = AppError::ControlSocketFailed("port 9000 in use".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("failed to start control socket"));
        assert!(msg.contains("port 9000"));
    }

    #[test]
    fn test_app_error_image_encode_failed() {
        let err = AppError::ImageEncodeFailed("unsupported format".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("failed to encode image"));
        assert!(msg.contains("unsupported format"));
    }

    #[test]
    fn test_app_error_io_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err = AppError::IoError(io_err);
        let msg = format!("{}", err);
        assert!(msg.contains("io error"));
    }

    #[test]
    fn test_app_error_from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
        let app_err: AppError = io_err.into();
        let msg = format!("{}", app_err);
        assert!(msg.contains("io error"));
    }

    #[test]
    fn test_app_error_implements_error_trait() {
        let err: Box<dyn std::error::Error> = Box::new(AppError::ClipboardUnavailable);
        assert!(!format!("{}", err).is_empty());
    }
}
