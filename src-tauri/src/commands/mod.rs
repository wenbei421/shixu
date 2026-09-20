mod attachments;
mod deepseek;
mod muse;
mod todo;

pub use attachments::*;
pub use deepseek::*;
pub use muse::*;
pub use todo::*;

use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::{error::AppError, sqlite::Db};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DbStatus {
    pub ok: bool,
    pub fragment_count: i64,
    pub data_dir: String,
    pub db_path: String,
}

#[tauri::command]
pub async fn get_db_status(db: tauri::State<'_, Db>) -> Result<DbStatus, AppError> {
    let fragment_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM fragments")
        .fetch_one(&db.pool)
        .await?;

    let data_dir = db
        .path
        .parent()
        .map(|p| p.display().to_string())
        .unwrap_or_default();

    Ok(DbStatus {
        ok: true,
        fragment_count,
        data_dir,
        db_path: db.path.display().to_string(),
    })
}

/// 用系统资源管理器 / 文件管理器打开应用数据目录（状态栏点击 SQLite 状态用）
#[tauri::command]
pub async fn reveal_data_dir(app: AppHandle) -> Result<(), AppError> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Invalid(format!("resolve app_data_dir: {e}")))?;

    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&dir)
            .spawn()
            .map_err(AppError::from)?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&dir)
            .spawn()
            .map_err(AppError::from)?;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open")
            .arg(&dir)
            .spawn()
            .map_err(AppError::from)?;
    }

    Ok(())
}

/// 清空当前环境的业务数据。不删除库文件，因此不会重新灌入种子数据。
#[tauri::command]
pub async fn reset_system(app: AppHandle, db: tauri::State<'_, Db>) -> Result<(), AppError> {
    crate::sqlite::clear_user_data(&db.pool).await?;

    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Invalid(format!("resolve app_data_dir: {e}")))?;
    crate::sqlite::remove_runtime_data(&dir)?;

    if let Ok(config) = app.path().app_config_dir() {
        let settings = config.join("settings.json");
        if settings.exists() {
            std::fs::remove_file(settings)?;
        }
    }
    Ok(())
}
