use directories::ProjectDirs;
use std::io;
use std::path::PathBuf;

const HISTORY_FILE: &str = ".clipboard_history.db";

pub(crate) fn default_history_path() -> io::Result<PathBuf> {
    let project_dirs = ProjectDirs::from("", "", "clipboard-manager").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "failed to resolve application data directory",
        )
    })?;

    Ok(project_dirs.data_local_dir().join(HISTORY_FILE))
}
