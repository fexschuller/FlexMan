mod config;
mod devices;
mod error;
mod fsutil;
mod install;
mod logging;
mod paths;
mod process;
mod report;
mod selftest;
mod settings;

use error::AppError;
use std::path::Path;

/// Runs blocking work on a worker thread instead of the async runtime's
/// main/event thread - without this, every command below would freeze the
/// window for as long as it takes (self-tests can take up to 8s).
async fn blocking<T, F>(f: F) -> Result<T, AppError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, AppError> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(f)
        .await
        .unwrap_or_else(|e| Err(AppError::ProcessFailed(format!("background task panicked: {e}"))))
}

#[tauri::command]
async fn list_devices(app: tauri::AppHandle) -> Result<Vec<devices::AudioDevice>, AppError> {
    blocking(move || devices::list(&app)).await
}

#[tauri::command]
async fn read_config() -> Result<config::Config, AppError> {
    blocking(config::load).await
}

#[tauri::command]
async fn write_config(update: config::ConfigUpdate) -> Result<(), AppError> {
    blocking(move || config::save(&update)).await
}

#[tauri::command]
async fn read_raw_config() -> Result<String, AppError> {
    blocking(config::read_raw).await
}

#[tauri::command]
async fn preview_config(update: config::ConfigUpdate) -> Result<String, AppError> {
    blocking(move || config::merged_text(&update)).await
}

#[tauri::command]
async fn write_raw_config(raw: String) -> Result<(), AppError> {
    blocking(move || config::write_raw(&raw)).await
}

#[tauri::command]
async fn find_toml_syntax_error(raw: String) -> Option<config::SyntaxError> {
    blocking(move || Ok(config::find_syntax_error(&raw))).await.unwrap_or(None)
}

#[tauri::command]
async fn run_self_test(app: tauri::AppHandle, update: config::ConfigUpdate) -> Result<String, AppError> {
    blocking(move || {
        let text = config::merged_text(&update)?;
        selftest::test_draft(&app, &text)
    })
    .await
}

#[tauri::command]
async fn run_self_test_raw(app: tauri::AppHandle, raw: String) -> Result<String, AppError> {
    blocking(move || selftest::test_draft(&app, &raw)).await
}

#[tauri::command]
async fn run_self_test_saved(app: tauri::AppHandle) -> Result<String, AppError> {
    blocking(move || selftest::test_saved(&app)).await
}

#[tauri::command]
async fn is_logging_enabled() -> Result<bool, AppError> {
    blocking(logging::is_enabled).await
}

#[tauri::command]
async fn set_logging_enabled(enabled: bool) -> Result<(), AppError> {
    blocking(move || logging::set_enabled(enabled)).await
}

#[tauri::command]
async fn read_log_tail() -> Result<String, AppError> {
    blocking(logging::read_log_tail).await
}

/// Remembers the chosen folder in settings; returns None if the user cancelled.
#[tauri::command]
async fn pick_flexasio_dir(app: tauri::AppHandle) -> Result<Option<std::path::PathBuf>, AppError> {
    blocking(move || {
        use tauri_plugin_dialog::DialogExt;

        let Some(folder) = app.dialog().file().blocking_pick_folder() else {
            return Ok(None);
        };
        let dir = folder.into_path().map_err(|e| AppError::Io(e.to_string()))?;

        let mut settings = settings::Settings::load(&app)?;
        settings.flexasio_dir = Some(dir.clone());
        settings.save(&app)?;

        Ok(Some(dir))
    })
    .await
}

#[tauri::command]
async fn get_system_report(app: tauri::AppHandle) -> Result<report::SystemReport, AppError> {
    blocking(move || report::gather(&app)).await
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
enum RevealTarget {
    FlexasioInstallDir,
    Config,
    Log,
}

/// The frontend never handles filesystem paths directly (see capabilities/default.json,
/// which grants no `opener:allow-open-path` scope): it names one of a fixed
/// set of targets, and only this command resolves and opens the real path.
#[tauri::command]
async fn reveal_path(app: tauri::AppHandle, target: RevealTarget) -> Result<(), AppError> {
    use tauri_plugin_opener::OpenerExt;

    blocking(move || match target {
        RevealTarget::FlexasioInstallDir => {
            let dir = install::find_tool(&app, "PortAudioDevices.exe")
                .and_then(|p| p.parent().map(Path::to_path_buf))
                .ok_or(AppError::FlexAsioNotFound)?;
            app.opener()
                .open_path(dir.display().to_string(), None::<String>)
                .map_err(|e| AppError::Io(e.to_string()))
        }
        RevealTarget::Config => {
            let path = config::path()?;
            app.opener()
                .reveal_item_in_dir(path)
                .map_err(|e| AppError::Io(e.to_string()))
        }
        RevealTarget::Log => {
            let path = logging::path()?;
            app.opener()
                .reveal_item_in_dir(path)
                .map_err(|e| AppError::Io(e.to_string()))
        }
    })
    .await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Finish restoring the config if a previous self-test never got to (the
    // app was killed or crashed mid-test); see selftest::recover_orphaned_backup.
    if let Ok(path) = config::path() {
        let _ = selftest::recover_orphaned_backup(&path);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            list_devices,
            read_config,
            write_config,
            read_raw_config,
            write_raw_config,
            preview_config,
            find_toml_syntax_error,
            run_self_test,
            run_self_test_raw,
            run_self_test_saved,
            is_logging_enabled,
            set_logging_enabled,
            read_log_tail,
            pick_flexasio_dir,
            get_system_report,
            reveal_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
