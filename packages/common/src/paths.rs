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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_app_paths_returns_valid_paths() {
        let result = resolve_app_paths();
        assert!(result.is_ok());
        let paths = result.unwrap();
        assert!(!paths.base_dir.as_os_str().is_empty());
        assert!(!paths.db_path.as_os_str().is_empty());
        assert!(!paths.config_path.as_os_str().is_empty());
        assert!(!paths.image_dir.as_os_str().is_empty());
    }

    #[test]
    fn test_app_paths_db_path_contains_history_file() {
        let paths = resolve_app_paths().unwrap();
        assert!(paths
            .db_path
            .to_string_lossy()
            .contains(HISTORY_FILE.as_str()));
    }

    #[test]
    fn test_app_paths_config_path_contains_config_file() {
        let paths = resolve_app_paths().unwrap();
        assert!(paths
            .config_path
            .to_string_lossy()
            .contains(CONFIG_FILE.as_str()));
    }

    #[test]
    fn test_app_paths_image_dir_contains_image_dir_name() {
        let paths = resolve_app_paths().unwrap();
        assert!(paths.image_dir.to_string_lossy().contains(IMAGE_DIR));
    }

    #[test]
    fn test_app_paths_consistency() {
        let paths = resolve_app_paths().unwrap();
        // All paths should be under base_dir
        assert!(paths.db_path.starts_with(&paths.base_dir));
        assert!(paths.config_path.starts_with(&paths.base_dir));
        assert!(paths.image_dir.starts_with(&paths.base_dir));
    }

    #[test]
    fn test_config_dir_returns_valid_path() {
        let result = config_dir();
        assert!(result.is_ok());
        let dir = result.unwrap();
        assert!(!dir.as_os_str().is_empty());
    }
}
