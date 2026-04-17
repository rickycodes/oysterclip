use common::constants::CONFIG_FILE;
use common::paths as common_paths;
use std::io;
use std::path::PathBuf;

pub fn default_history_path() -> io::Result<PathBuf> {
    Ok(common_paths::resolve_app_paths()?.db_path)
}

pub fn config_path() -> io::Result<PathBuf> {
    Ok(common_paths::config_dir()?.join(CONFIG_FILE.as_str()))
}
