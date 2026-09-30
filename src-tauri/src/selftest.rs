use crate::config;
use crate::error::AppError;
use crate::fsutil;
use crate::install;
use crate::process;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};
use tauri::AppHandle;

const TIMEOUT: Duration = Duration::from_secs(8);

/// What the config file looked like before a self-test overwrote it, so it
/// can be restored afterwards - persisted to disk (not just kept in memory)
/// so a crash mid-test can still be recovered from on the next app start.
#[derive(serde::Serialize, serde::Deserialize)]
struct Backup {
    config_path: PathBuf,
    existed: bool,
    content: String,
}

fn marker_path(config_path: &Path) -> PathBuf {
    let mut marker = config_path.as_os_str().to_owned();
    marker.push(".flexman-backup");
    PathBuf::from(marker)
}

impl Backup {
    fn capture(config_path: &Path) -> Result<Backup, AppError> {
        match std::fs::read_to_string(config_path) {
            Ok(content) => Ok(Backup {
                config_path: config_path.to_path_buf(),
                existed: true,
                content,
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Backup {
                config_path: config_path.to_path_buf(),
                existed: false,
                content: String::new(),
            }),
            Err(e) => Err(AppError::io(config_path, e)),
        }
    }

    fn write_marker(&self) -> Result<(), AppError> {
        let marker = marker_path(&self.config_path);
        let json = serde_json::to_string(self).map_err(|e| AppError::Io(e.to_string()))?;
        fsutil::write_atomic(&marker, &json)
    }

    /// Restores the config file to exactly what it was before (deleting it
    /// again if it didn't exist yet), then clears the on-disk marker - this
    /// is what makes the whole swap safe even across a crash or kill.
    fn restore(&self) -> Result<(), AppError> {
        if self.existed {
            fsutil::write_atomic(&self.config_path, &self.content)?;
        } else if self.config_path.exists() {
            std::fs::remove_file(&self.config_path).map_err(|e| AppError::io(&self.config_path, e))?;
        }
        let marker = marker_path(&self.config_path);
        if marker.exists() {
            std::fs::remove_file(&marker).map_err(|e| AppError::io(&marker, e))?;
        }
        Ok(())
    }
}

/// Called once at app startup: if a previous self-test never got to restore
/// the config (app was killed, machine lost power, ...), finish the job now.
///
/// This - not a `Drop` guard - is the actual safety net here: the release
/// profile uses `panic = "abort"`, which skips destructors entirely, so a
/// `Drop`-based guard would silently do nothing on the crash paths that
/// matter most.
pub fn recover_orphaned_backup(config_path: &Path) -> Result<(), AppError> {
    let marker = marker_path(config_path);
    if !marker.exists() {
        return Ok(());
    }

    let json = std::fs::read_to_string(&marker).map_err(|e| AppError::io(&marker, e))?;
    let backup: Backup =
        serde_json::from_str(&json).map_err(|e| AppError::InvalidConfig(e.to_string()))?;
    backup.restore()
}

/// Temporarily writes `draft` (unsaved config text, from the form or the raw
/// editor) in place of the real config, runs the test, then restores it.
///
/// Note this briefly makes FlexASIO see `draft` as the real config: FlexASIO
/// watches the file and issues a live reset to any running DAW on every
/// change, so testing while a DAW has FlexASIO open will audibly interrupt it
/// twice (once for the draft, once for the restore).
pub fn test_draft(app: &AppHandle, draft: &str) -> Result<String, AppError> {
    let config_path = config::path()?;
    let backup = Backup::capture(&config_path)?;
    backup.write_marker()?;

    fsutil::write_atomic(&config_path, draft)?;
    let result = test_saved(app);

    match backup.restore() {
        Ok(()) => result,
        Err(restore_err) => match result {
            Ok(_) => Err(AppError::Io(format!(
                "Test ran, but failed to restore the previous config: {restore_err}"
            ))),
            Err(test_err) => Err(AppError::Io(format!(
                "{test_err}\n(Also failed to restore the previous config: {restore_err})"
            ))),
        },
    }
}

/// Runs FlexASIOTest.exe against whatever is currently saved on disk
/// (killing it after a timeout) and returns its output.
pub fn test_saved(app: &AppHandle) -> Result<String, AppError> {
    let exe = install::find_tool(app, "FlexASIOTest.exe").ok_or(AppError::FlexAsioNotFound)?;

    let mut command = process::hidden_command(exe);
    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .map_err(|e| AppError::ProcessFailed(e.to_string()))?;

    // Drain both pipes concurrently: a full pipe buffer would block the child's
    // write, which would deadlock against us waiting for it to exit.
    let mut stdout_pipe = child.stdout.take().expect("stdout was piped");
    let mut stderr_pipe = child.stderr.take().expect("stderr was piped");
    let stdout_thread = std::thread::spawn(move || {
        let mut buf = String::new();
        let _ = stdout_pipe.read_to_string(&mut buf);
        buf
    });
    let stderr_thread = std::thread::spawn(move || {
        let mut buf = String::new();
        let _ = stderr_pipe.read_to_string(&mut buf);
        buf
    });

    let start = Instant::now();
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|e| AppError::ProcessFailed(e.to_string()))?
        {
            break Some(status);
        }
        if start.elapsed() > TIMEOUT {
            let _ = child.kill();
            let _ = child.wait();
            break None;
        }
        std::thread::sleep(Duration::from_millis(100));
    };

    let stdout = stdout_thread.join().unwrap_or_default();
    let stderr = stderr_thread.join().unwrap_or_default();
    let combined = format!("{stdout}{stderr}");

    match status {
        Some(status) if status.success() => Ok(combined),
        Some(_) => Err(AppError::ProcessFailed(combined)),
        None => Err(AppError::SelfTestTimeout(format!(
            "Self-test timed out after {} seconds.\n{combined}",
            TIMEOUT.as_secs()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("flexasio-selftest-test-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn restore_rewrites_original_content_when_file_existed() {
        let dir = temp_dir("restore-existed");
        let config_path = dir.join("FlexASIO.toml");
        std::fs::write(&config_path, "backend = \"MME\"\n").unwrap();

        let backup = Backup::capture(&config_path).unwrap();
        std::fs::write(&config_path, "backend = \"Windows WASAPI\"\n").unwrap();
        backup.restore().unwrap();

        assert_eq!(
            std::fs::read_to_string(&config_path).unwrap(),
            "backend = \"MME\"\n"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn restore_deletes_file_when_it_did_not_exist_before() {
        let dir = temp_dir("restore-absent");
        let config_path = dir.join("FlexASIO.toml");

        let backup = Backup::capture(&config_path).unwrap();
        std::fs::write(&config_path, "backend = \"Windows WASAPI\"\n").unwrap();
        backup.restore().unwrap();

        assert!(!config_path.exists());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn recover_orphaned_backup_finishes_an_interrupted_restore() {
        let dir = temp_dir("recover");
        let config_path = dir.join("FlexASIO.toml");
        std::fs::write(&config_path, "backend = \"MME\"\n").unwrap();

        let backup = Backup::capture(&config_path).unwrap();
        backup.write_marker().unwrap();
        std::fs::write(&config_path, "backend = \"Windows WASAPI\"\n").unwrap();
        // Simulate a crash: the marker is left behind, restore() never ran.

        recover_orphaned_backup(&config_path).unwrap();

        assert_eq!(
            std::fs::read_to_string(&config_path).unwrap(),
            "backend = \"MME\"\n"
        );
        assert!(!marker_path(&config_path).exists());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn recover_orphaned_backup_is_a_no_op_without_a_marker() {
        let dir = temp_dir("recover-noop");
        let config_path = dir.join("FlexASIO.toml");

        assert!(recover_orphaned_backup(&config_path).is_ok());

        std::fs::remove_dir_all(&dir).ok();
    }
}
