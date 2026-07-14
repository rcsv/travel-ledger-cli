//! Backend-owned desktop settings (last selected database path).

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::DesktopError;

const SETTINGS_FILE_NAME: &str = "desktop-settings.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DesktopSettings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_database_path: Option<String>,
}

pub fn settings_file_path(config_dir: &Path) -> PathBuf {
    config_dir.join(SETTINGS_FILE_NAME)
}

/// Distinguishes corrupt JSON from missing file for restore semantics.
pub fn load_saved_database_path(config_path: &Path) -> Result<LoadSavedPath, DesktopError> {
    if !config_path.exists() {
        return Ok(LoadSavedPath::Absent);
    }
    let raw = fs::read_to_string(config_path).map_err(|err| {
        DesktopError::database_config_read_failed(format!("Failed to read settings: {err}"))
    })?;
    if raw.trim().is_empty() {
        return Ok(LoadSavedPath::Absent);
    }
    match serde_json::from_str::<DesktopSettings>(&raw) {
        Ok(settings) => match settings.last_database_path {
            Some(path) if !path.trim().is_empty() => {
                Ok(LoadSavedPath::Present(PathBuf::from(path)))
            }
            _ => Ok(LoadSavedPath::Absent),
        },
        Err(_) => Ok(LoadSavedPath::Corrupt),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum LoadSavedPath {
    Absent,
    Present(PathBuf),
    Corrupt,
}

pub fn write_last_database_path(config_path: &Path, db_path: &Path) -> Result<(), DesktopError> {
    let settings = DesktopSettings {
        last_database_path: Some(path_to_stored_string(db_path)),
    };
    write_settings_atomic(config_path, &settings)
}

pub fn clear_settings(config_path: &Path) -> Result<(), DesktopError> {
    if !config_path.exists() {
        return Ok(());
    }
    fs::remove_file(config_path).map_err(|err| {
        DesktopError::database_config_write_failed(format!("Failed to clear settings: {err}"))
    })
}

fn path_to_stored_string(path: &Path) -> String {
    path.to_str()
        .map(str::to_string)
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}

fn write_settings_atomic(
    config_path: &Path,
    settings: &DesktopSettings,
) -> Result<(), DesktopError> {
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            DesktopError::database_config_write_failed(format!(
                "Failed to create config directory: {err}"
            ))
        })?;
    }
    let body = serde_json::to_string_pretty(settings).map_err(|err| {
        DesktopError::database_config_write_failed(format!("Failed to serialize settings: {err}"))
    })?;
    let tmp_path = config_path.with_extension("json.tmp");
    fs::write(&tmp_path, body.as_bytes()).map_err(|err| {
        DesktopError::database_config_write_failed(format!("Failed to write settings: {err}"))
    })?;
    fs::rename(&tmp_path, config_path).map_err(|err| {
        let _ = fs::remove_file(&tmp_path);
        DesktopError::database_config_write_failed(format!("Failed to finalize settings: {err}"))
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn roundtrip_last_database_path() {
        let dir = tempdir().unwrap();
        let path = settings_file_path(dir.path());
        write_last_database_path(&path, Path::new("/tmp/sample.db")).unwrap();
        match load_saved_database_path(&path).unwrap() {
            LoadSavedPath::Present(p) => assert_eq!(p, PathBuf::from("/tmp/sample.db")),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn clear_removes_file() {
        let dir = tempdir().unwrap();
        let path = settings_file_path(dir.path());
        write_last_database_path(&path, Path::new("/tmp/a.db")).unwrap();
        assert!(path.exists());
        clear_settings(&path).unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn corrupt_json_is_corrupt_load() {
        let dir = tempdir().unwrap();
        let path = settings_file_path(dir.path());
        fs::write(&path, "{not-json").unwrap();
        assert_eq!(
            load_saved_database_path(&path).unwrap(),
            LoadSavedPath::Corrupt
        );
    }
}
