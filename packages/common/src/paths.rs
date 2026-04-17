use directories::ProjectDirs;
use std::io;
use std::path::PathBuf;

use super::constants::{
    APP_NAME, APP_ORGANIZATION, APP_QUALIFIER, CONFIG_FILE, ERR_RESOLVE_APP_DIR, HISTORY_FILE,
    IMAGE_DIR,
};

#[derive(Clone, Debug)]
pub struct AppPaths {
    pub base_dir: PathBuf,
    pub db_path: PathBuf,
    pub config_path: PathBuf,
    pub image_dir: PathBuf,
}

pub fn resolve_app_paths() -> io::Result<AppPaths> {
    let project_dirs = ProjectDirs::from(APP_QUALIFIER, APP_ORGANIZATION, APP_NAME)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, ERR_RESOLVE_APP_DIR))?;

    let base_dir = project_dirs.data_local_dir().to_path_buf();
    Ok(AppPaths {
        db_path: base_dir.join(HISTORY_FILE.as_str()),
        config_path: base_dir.join(CONFIG_FILE.as_str()),
        image_dir: base_dir.join(IMAGE_DIR),
        base_dir,
    })
}

pub fn ensure_app_dir(paths: &AppPaths) -> io::Result<()> {
    std::fs::create_dir_all(&paths.base_dir)
}

pub fn config_dir() -> io::Result<PathBuf> {
    let project_dirs = ProjectDirs::from(APP_QUALIFIER, APP_ORGANIZATION, APP_NAME)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, ERR_RESOLVE_APP_DIR))?;
    Ok(project_dirs.config_dir().to_path_buf())
}
