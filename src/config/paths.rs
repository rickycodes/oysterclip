use directories::ProjectDirs;
use std::io;
use std::path::PathBuf;

const HISTORY_FILE: &str = ".clipboard_history.db";
const CONFIG_FILE: &str = "config.toml";

fn project_dirs() -> io::Result<ProjectDirs> {
    ProjectDirs::from("", "", "clipboard-manager").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "failed to resolve application data directory",
        )
    })
}

pub fn default_history_path() -> io::Result<PathBuf> {
    Ok(project_dirs()?.data_local_dir().join(HISTORY_FILE))
}

pub fn config_path() -> io::Result<PathBuf> {
    Ok(project_dirs()?.config_dir().join(CONFIG_FILE))
}
