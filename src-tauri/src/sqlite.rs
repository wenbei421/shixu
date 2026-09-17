use std::path::{Path, PathBuf};

use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};
use tauri::{App, Manager};

use crate::error::AppError;

#[derive(Debug)]
pub struct Db {
    pub pool: SqlitePool,
    pub path: PathBuf,
}

impl Db {
    pub fn new(pool: SqlitePool, path: PathBuf) -> Self {
        Self { pool, path }
    }
}

async fn init_db(db_path: &Path) -> Result<SqlitePool, AppError> {
    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

/// 若存在待恢复副本，在打开库之前覆盖正式库文件（FR-13 恢复流程）
fn apply_pending_restore(app_data_dir: &Path) -> Result<(), AppError> {
    let pending = app_data_dir.join("shixu.db.pending_restore");
    if !pending.exists() {
        return Ok(());
    }

    let db_path = app_data_dir.join("shixu.db");
    for suffix in ["", "-wal", "-shm"] {
        let path = PathBuf::from(format!("{}{suffix}", db_path.display()));
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
    }

    std::fs::rename(&pending, &db_path)?;
    log::info!("Applied pending Muse/DB restore → {}", db_path.display());
    Ok(())
}

pub fn set_db(app: &mut App) -> Result<(), AppError> {
    let app_data_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&app_data_dir)?;

    apply_pending_restore(&app_data_dir)?;

    let db_path = app_data_dir.join("shixu.db");
    log::info!("SQLite database: {}", db_path.display());

    let handle = app.handle().clone();
    let pool = tauri::async_runtime::block_on(async { init_db(&db_path).await })?;
    handle.manage(Db::new(pool, db_path));
    Ok(())
}
