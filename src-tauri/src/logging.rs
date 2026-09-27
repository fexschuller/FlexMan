use crate::error::AppError;
use crate::fsutil;
use crate::paths;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

const LOG_TAIL_BYTES: u64 = 64 * 1024;

pub fn path() -> Result<PathBuf, AppError> {
    Ok(paths::userprofile()?.join("FlexASIO.log"))
}

/// FlexASIO logs verbosely whenever this file exists, regardless of its content.
pub fn is_enabled() -> Result<bool, AppError> {
    Ok(path()?.exists())
}

pub fn set_enabled(enabled: bool) -> Result<(), AppError> {
    let path = path()?;

    if enabled {
        fsutil::write_atomic(&path, "")
    } else if path.exists() {
        std::fs::remove_file(&path).map_err(|e| AppError::io(&path, e))
    } else {
        Ok(())
    }
}

/// Reads up to the last 64 KB of FlexASIO.log (empty string if it doesn't exist yet).
pub fn read_log_tail() -> Result<String, AppError> {
    read_log_tail_from(&path()?)
}

fn read_log_tail_from(path: &Path) -> Result<String, AppError> {
    if !path.exists() {
        return Ok(String::new());
    }

    let mut file = std::fs::File::open(path).map_err(|e| AppError::io(path, e))?;
    let len = file.metadata().map_err(|e| AppError::io(path, e))?.len();

    if len > LOG_TAIL_BYTES {
        file.seek(SeekFrom::End(-(LOG_TAIL_BYTES as i64)))
            .map_err(|e| AppError::io(path, e))?;
    }

    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).map_err(|e| AppError::io(path, e))?;

    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_log_tail_from_missing_file_returns_empty_string() {
        let path = std::env::temp_dir().join("flexasio-logging-test-does-not-exist.log");

        assert_eq!(read_log_tail_from(&path).unwrap(), "");
    }

    #[test]
    fn read_log_tail_from_truncates_to_last_bytes() {
        let dir = std::env::temp_dir().join(format!("flexasio-logging-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("FlexASIO.log");
        let contents = "x".repeat(LOG_TAIL_BYTES as usize + 100) + "TAIL";
        std::fs::write(&path, &contents).unwrap();

        let tail = read_log_tail_from(&path).unwrap();

        assert_eq!(tail.len() as u64, LOG_TAIL_BYTES);
        assert!(tail.ends_with("TAIL"));

        std::fs::remove_dir_all(&dir).ok();
    }
}
