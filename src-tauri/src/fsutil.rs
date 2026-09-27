use crate::error::AppError;
use std::path::Path;

/// Writes via a temp file + rename so a crash or concurrent read (FlexASIO
/// watches this file for changes) never observes a half-written file.
pub fn write_atomic(path: &Path, contents: &str) -> Result<(), AppError> {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = Path::new(&tmp);

    std::fs::write(tmp, contents).map_err(|e| AppError::io(tmp, e))?;
    std::fs::rename(tmp, path).map_err(|e| AppError::io(path, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_new_file_and_cleans_up_tmp() {
        let dir = std::env::temp_dir().join(format!("flexasio-fsutil-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("out.toml");

        write_atomic(&path, "backend = \"MME\"\n").unwrap();

        assert_eq!(std::fs::read_to_string(&path).unwrap(), "backend = \"MME\"\n");
        assert!(!path.with_extension("toml.tmp").exists());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn overwrites_existing_file() {
        let dir = std::env::temp_dir().join(format!("flexasio-fsutil-test2-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("out.toml");
        std::fs::write(&path, "old").unwrap();

        write_atomic(&path, "new").unwrap();

        assert_eq!(std::fs::read_to_string(&path).unwrap(), "new");

        std::fs::remove_dir_all(&dir).ok();
    }
}
