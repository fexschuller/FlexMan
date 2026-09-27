use std::fmt;
use std::path::Path;

/// Tagged so the frontend can match on `err.kind` instead of comparing error
/// strings (see AppError::FlexAsioNotFound's one call site in ConfigForm.tsx).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "kind", content = "message", rename_all = "camelCase")]
pub enum AppError {
    FlexAsioNotFound,
    Io(String),
    InvalidConfig(String),
    ProcessFailed(String),
    SelfTestTimeout(String),
}

impl AppError {
    pub fn io(path: &Path, e: std::io::Error) -> Self {
        AppError::Io(format!("{}: {}", path.display(), e))
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::FlexAsioNotFound => {
                write!(f, "FlexASIO not found in any known install location")
            }
            AppError::Io(m)
            | AppError::InvalidConfig(m)
            | AppError::ProcessFailed(m)
            | AppError::SelfTestTimeout(m) => write!(f, "{m}"),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e.to_string())
    }
}
