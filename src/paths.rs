use directories::ProjectDirs;
use std::io;
use std::path::PathBuf;

use crate::constants::{CONFIG_FILE, HISTORY_FILE};

pub(crate) struct AppPaths {
    pub(crate) base_dir: PathBuf,
    pub(crate) db_path: PathBuf,
    pub(crate) config_path: PathBuf,
    pub(crate) image_dir: PathBuf,
}

pub(crate) fn resolve_app_paths() -> io::Result<AppPaths> {
    let project_dirs = ProjectDirs::from("", "", "clipboard-manager").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "failed to resolve application data directory",
        )
    })?;

    let base_dir = project_dirs.data_local_dir().to_path_buf();
    Ok(AppPaths {
        db_path: base_dir.join(HISTORY_FILE),
        config_path: base_dir.join(CONFIG_FILE),
        image_dir: base_dir.join("clipboard_images"),
        base_dir,
    })
}

pub(crate) fn ensure_app_dir(paths: &AppPaths) -> io::Result<()> {
    std::fs::create_dir_all(&paths.base_dir)
}
