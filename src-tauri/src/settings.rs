use crate::error::AppError;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

/// Persisted app settings (currently just a user-picked FlexASIO install folder,
/// used as a fallback when it isn't found at any of the usual install paths).
#[derive(serde::Deserialize, serde::Serialize, Debug, Default, PartialEq)]
pub struct Settings {
    pub flexasio_dir: Option<PathBuf>,
}

impl Settings {
    /// Reads the app's settings, or defaults if the file doesn't exist yet.
    pub fn load(app: &AppHandle) -> Result<Settings, AppError> {
        load_from(&path(app)?)
    }

    /// Saves the settings, creating the app data directory if needed.
    pub fn save(&self, app: &AppHandle) -> Result<(), AppError> {
        let path = path(app)?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| AppError::io(dir, e))?;
        }

        let json = serde_json::to_string_pretty(self).map_err(|e| AppError::Io(e.to_string()))?;
        std::fs::write(&path, json).map_err(|e| AppError::io(&path, e))
    }
}

fn load_from(path: &Path) -> Result<Settings, AppError> {
    if !path.exists() {
        return Ok(Settings::default());
    }

    let raw = std::fs::read_to_string(path).map_err(|e| AppError::io(path, e))?;
    serde_json::from_str(&raw).map_err(|e| AppError::InvalidConfig(e.to_string()))
}

fn path(app: &AppHandle) -> Result<PathBuf, AppError> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| AppError::Io(e.to_string()))?;
    Ok(dir.join("settings.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_from_missing_file_returns_defaults() {
        let path = std::env::temp_dir().join("flexasio-settings-test-does-not-exist.json");

        assert_eq!(load_from(&path).unwrap(), Settings::default());
    }

    #[test]
    fn load_from_existing_file_returns_its_contents() {
        let dir = std::env::temp_dir().join(format!("flexasio-settings-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings.json");
        std::fs::write(&path, r#"{"flexasio_dir":"C:\\FlexASIO"}"#).unwrap();

        let settings = load_from(&path).unwrap();

        assert_eq!(settings.flexasio_dir, Some(PathBuf::from(r"C:\FlexASIO")));

        std::fs::remove_dir_all(&dir).ok();
    }
}
