use crate::settings::Settings;
use std::path::PathBuf;
use tauri::AppHandle;

/// Checks a user-picked folder (see `Settings`) before the usual install locations.
/// Each base is tried both directly and via its `x64`/`x86` subfolder, since a
/// user might point the folder picker straight at either.
pub fn find_tool(app: &AppHandle, filename: &str) -> Option<PathBuf> {
    let custom_dir = Settings::load(app).ok().and_then(|s| s.flexasio_dir);

    candidate_dirs(custom_dir)
        .into_iter()
        .map(|dir| dir.join(filename))
        .find(|p| p.exists())
}

fn candidate_dirs(custom_dir: Option<PathBuf>) -> Vec<PathBuf> {
    let program_files =
        std::env::var("ProgramFiles").unwrap_or_else(|_| r"C:\Program Files".to_string());
    let program_files_x86 = std::env::var("ProgramFiles(x86)")
        .unwrap_or_else(|_| r"C:\Program Files (x86)".to_string());

    let mut dirs = Vec::new();
    if let Some(dir) = custom_dir {
        dirs.push(dir.clone());
        dirs.push(dir.join("x64"));
        dirs.push(dir.join("x86"));
    }
    dirs.push(PathBuf::from(&program_files).join("FlexASIO").join("x64"));
    dirs.push(
        PathBuf::from(&program_files_x86)
            .join("FlexASIO")
            .join("x64"),
    );
    dirs.push(
        PathBuf::from(&program_files_x86)
            .join("FlexASIO")
            .join("x86"),
    );
    dirs
}
