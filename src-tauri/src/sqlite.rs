use std::path::{Path, PathBuf};

use sqlx::{
    Acquire, SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};
use tauri::{App, Manager};

use crate::error::AppError;

/// 开发 / 生产运行时标记（与 `cfg(debug_assertions)` 对齐：`tauri dev` vs `tauri build`）
pub fn is_dev() -> bool {
    cfg!(debug_assertions)
}

/// 主库文件名：开发用 `shixu.dev.db`，生产用 `shixu.db`
pub fn db_file_name() -> &'static str {
    if is_dev() { "shixu.dev.db" } else { "shixu.db" }
}

/// 待恢复副本文件名（与主库同前缀，避免跨环境互相覆盖）
pub fn pending_restore_file_name() -> &'static str {
    if is_dev() {
        "shixu.dev.db.pending_restore"
    } else {
        "shixu.db.pending_restore"
    }
}

/// Muse 备份目录名：开发 / 生产隔离
pub fn muse_backup_dir_name() -> &'static str {
    if is_dev() {
        "muse-backups-dev"
    } else {
        "muse-backups"
    }
}

/// `app_data_dir` 下的主库绝对路径
pub fn db_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(db_file_name())
}

/// `app_data_dir` 下的待恢复文件绝对路径
pub fn pending_restore_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(pending_restore_file_name())
}

/// `app_data_dir` 下的 Muse 备份目录
pub fn muse_backup_dir(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(muse_backup_dir_name())
}

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

/// 删除当前环境的待恢复副本和 Muse 备份。主库文件保留。
pub fn remove_runtime_data(app_data_dir: &Path) -> Result<(), AppError> {
    let pending = pending_restore_path(app_data_dir);
    if pending.exists() {
        std::fs::remove_file(&pending)?;
    }

    let backups = muse_backup_dir(app_data_dir);
    if backups.exists() {
        std::fs::remove_dir_all(&backups)?;
    }
    Ok(())
}

/// 清空业务表。保留库文件和迁移记录，避免下次启动重跑种子数据。
pub async fn clear_user_data(pool: &SqlitePool) -> Result<(), AppError> {
    const TABLES: &[&str] = &[
        "muse_reviews",
        "muse_note_tags",
        "todo_task_tags",
        "muse_notes",
        "todo_tasks",
        "sys_tags",
        "sys_projects",
        "sys_settings",
        "fragments",
        "inspiration_tag_map",
        "inspirations",
        "inspiration_tags",
        "inspiration_categories",
    ];

    let mut conn = pool.acquire().await?;
    sqlx::query("PRAGMA foreign_keys = OFF")
        .execute(&mut *conn)
        .await?;
    let cleared = async {
        let mut tx = conn.begin().await?;
        for table in TABLES {
            let statement = format!("DELETE FROM {table}");
            sqlx::query(&statement).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok::<(), AppError>(())
    }
    .await;
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&mut *conn)
        .await?;
    cleared
}

fn apply_pending_restore(app_data_dir: &Path) -> Result<(), AppError> {
    let pending = pending_restore_path(app_data_dir);
    if !pending.exists() {
        return Ok(());
    }

    let target = db_path(app_data_dir);
    for suffix in ["", "-wal", "-shm"] {
        let path = PathBuf::from(format!("{}{suffix}", target.display()));
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
    }

    std::fs::rename(&pending, &target)?;
    log::info!("Applied pending Muse/DB restore → {}", target.display());
    Ok(())
}

pub fn set_db(app: &mut App) -> Result<(), AppError> {
    let app_data_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&app_data_dir)?;

    apply_pending_restore(&app_data_dir)?;

    let path = db_path(&app_data_dir);
    let env_label = if is_dev() {
        "development"
    } else {
        "production"
    };
    log::info!("SQLite database ({env_label}): {}", path.display());

    let handle = app.handle().clone();
    let pool = tauri::async_runtime::block_on(async { init_db(&path).await })?;
    handle.manage(Db::new(pool, path));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn db_paths_are_environment_scoped() {
        let root = Path::new("/tmp/shixu-app-data");
        let db = db_path(root);
        let pending = pending_restore_path(root);
        let backups = muse_backup_dir(root);

        if is_dev() {
            assert_eq!(
                db.file_name().and_then(|n| n.to_str()),
                Some("shixu.dev.db")
            );
            assert_eq!(
                pending.file_name().and_then(|n| n.to_str()),
                Some("shixu.dev.db.pending_restore")
            );
            assert_eq!(
                backups.file_name().and_then(|n| n.to_str()),
                Some("muse-backups-dev")
            );
        } else {
            assert_eq!(db.file_name().and_then(|n| n.to_str()), Some("shixu.db"));
            assert_eq!(
                pending.file_name().and_then(|n| n.to_str()),
                Some("shixu.db.pending_restore")
            );
            assert_eq!(
                backups.file_name().and_then(|n| n.to_str()),
                Some("muse-backups")
            );
        }

        assert_eq!(db, root.join(db_file_name()));
        assert_eq!(pending, root.join(pending_restore_file_name()));
        assert_eq!(backups, root.join(muse_backup_dir_name()));
    }
}
