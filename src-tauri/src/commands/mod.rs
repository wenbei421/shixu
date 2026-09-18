mod muse;
mod todo;

pub use muse::*;
pub use todo::*;

use serde::Serialize;

use crate::{error::AppError, sqlite::Db};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DbStatus {
    pub ok: bool,
    pub fragment_count: i64,
}

#[tauri::command]
pub async fn get_db_status(db: tauri::State<'_, Db>) -> Result<DbStatus, AppError> {
    let fragment_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM fragments")
        .fetch_one(&db.pool)
        .await?;

    Ok(DbStatus {
        ok: true,
        fragment_count,
    })
}
