use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
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
