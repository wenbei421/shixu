use std::path::Path;

use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};
use tauri::{App, Manager};

use crate::error::AppError;

#[derive(Debug)]
pub struct Db {
    pub pool: SqlitePool,
}

impl Db {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
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

pub fn set_db(app: &mut App) -> Result<(), AppError> {
    let app_data_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&app_data_dir)?;

    let db_path = app_data_dir.join("shixu.db");
    log::info!("SQLite database: {}", db_path.display());

    let handle = app.handle().clone();
    let pool = tauri::async_runtime::block_on(async { init_db(&db_path).await })?;
    handle.manage(Db::new(pool));
    Ok(())
}
