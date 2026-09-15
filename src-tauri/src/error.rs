use sqlx::migrate::MigrateError;
use thiserror::Error;

/// 拾序 OS 应用错误
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Tauri error: {0}")]
    Tauri(#[from] tauri::Error),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Migration error: {0}")]
    Migration(#[from] MigrateError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Invalid: {0}")]
    Invalid(String),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
