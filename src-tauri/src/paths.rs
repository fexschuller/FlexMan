use crate::error::AppError;
use std::path::PathBuf;

/// The user's profile folder, where FlexASIO looks for FlexASIO.toml/.log.
pub fn userprofile() -> Result<PathBuf, AppError> {
    std::env::var("USERPROFILE")
        .map(PathBuf::from)
        .map_err(|_| AppError::Io("USERPROFILE is not set".to_string()))
}
