//! 共用附件：元数据在 `sys_attachments`，文件落盘 `attachments/{todo|muse}/{owner_id}/`。

use std::path::{Path, PathBuf};

use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::{FromRow, SqlitePool};
use tauri::{AppHandle, Manager};

use crate::{
    error::AppError,
    id::{self, now_ms},
    sqlite::{self, Db},
};

pub const MAX_ATTACHMENT_BYTES: u64 = 10 * 1024 * 1024;
pub const MAX_ATTACHMENTS_PER_OWNER: i64 = 5;

const ALLOWED_EXT: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "pdf", "md", "markdown", "docx", "zip", "7z", "rar",
];

pub fn normalize_ext(filename: &str) -> Option<String> {
    let name = Path::new(filename).file_name()?.to_str()?;
    let ext = Path::new(name).extension()?.to_str()?.to_ascii_lowercase();
    if ALLOWED_EXT.contains(&ext.as_str()) {
        Some(ext)
    } else {
        None
    }
}

pub fn mime_for_ext(ext: &str) -> &'static str {
    match ext {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "pdf" => "application/pdf",
        "md" | "markdown" => "text/markdown",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "zip" => "application/zip",
        "7z" => "application/x-7z-compressed",
        "rar" => "application/vnd.rar",
        _ => "application/octet-stream",
    }
}

pub fn is_previewable(ext: &str) -> bool {
    matches!(
        ext,
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "pdf" | "md" | "markdown"
    )
}

/// 拒绝含 `..` 或路径分隔符的 stored_name
pub fn validate_stored_name(name: &str) -> Result<(), AppError> {
    if name.is_empty() || name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err(AppError::Invalid("invalid stored_name".into()));
    }
    Ok(())
}

fn parse_owner_type(s: &str) -> Result<&str, AppError> {
    match s {
        "todo" | "muse" => Ok(s),
        _ => Err(AppError::Invalid(format!("unknown owner_type: {s}"))),
    }
}

async fn owner_exists(pool: &SqlitePool, owner_type: &str, owner_id: i64) -> Result<(), AppError> {
    let sql = match owner_type {
        "todo" => "SELECT 1 FROM todo_tasks WHERE id = ? AND deleted_at IS NULL",
        "muse" => "SELECT 1 FROM muse_notes WHERE id = ? AND deleted_at IS NULL",
        _ => return Err(AppError::Invalid(format!("unknown owner_type: {owner_type}"))),
    };
    let ok: Option<i64> = sqlx::query_scalar(sql)
        .bind(owner_id)
        .fetch_optional(pool)
        .await?;
    if ok.is_none() {
        return Err(AppError::NotFound(format!("{owner_type} {owner_id}")));
    }
    Ok(())
}

pub fn owner_dir(app_data: &Path, owner_type: &str, owner_id: i64) -> PathBuf {
    sqlite::attachments_dir(app_data)
        .join(owner_type)
        .join(owner_id.to_string())
}

fn app_data_from_db(db: &Db) -> Result<PathBuf, AppError> {
    db.path
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| AppError::Invalid("db parent".into()))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentDto {
    pub id: String,
    pub owner_type: String,
    pub owner_id: String,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub hash_sha256: String,
    pub created_at: i64,
    pub missing: bool,
}

#[derive(Debug, FromRow)]
struct AttachmentRow {
    id: i64,
    owner_type: String,
    owner_id: i64,
    filename: String,
    stored_name: String,
    mime_type: String,
    size_bytes: i64,
    hash_sha256: String,
    created_at: i64,
}

fn row_to_dto(row: AttachmentRow, app_data: &Path) -> AttachmentDto {
    let path = owner_dir(app_data, &row.owner_type, row.owner_id).join(&row.stored_name);
    let missing = !path.is_file();
    AttachmentDto {
        id: id::id_to_string(row.id),
        owner_type: row.owner_type,
        owner_id: id::id_to_string(row.owner_id),
        filename: row.filename,
        mime_type: row.mime_type,
        size_bytes: row.size_bytes,
        hash_sha256: row.hash_sha256,
        created_at: row.created_at,
        missing,
    }
}

async fn fetch_attachment_row(
    pool: &SqlitePool,
    attachment_id: i64,
) -> Result<AttachmentRow, AppError> {
    let row: Option<AttachmentRow> = sqlx::query_as(
        "SELECT id, owner_type, owner_id, filename, stored_name, mime_type, size_bytes, hash_sha256, created_at \
         FROM sys_attachments WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(attachment_id)
    .fetch_optional(pool)
    .await?;
    row.ok_or_else(|| AppError::NotFound(format!("attachment {attachment_id}")))
}

fn resolve_file_path(app_data: &Path, row: &AttachmentRow) -> Result<PathBuf, AppError> {
    validate_stored_name(&row.stored_name)?;
    let path = owner_dir(app_data, &row.owner_type, row.owner_id).join(&row.stored_name);
    // 确保解析后仍在 owner 目录内
    let parent = owner_dir(app_data, &row.owner_type, row.owner_id);
    if !path.starts_with(&parent) {
        return Err(AppError::Invalid("path traversal".into()));
    }
    Ok(path)
}

fn open_path_with_system(path: &Path) -> Result<(), AppError> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &path.display().to_string()])
            .spawn()
            .map_err(AppError::from)?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(AppError::from)?;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(AppError::from)?;
    }
    Ok(())
}

/// 软删某 owner 下全部附件元数据（保留磁盘文件，供回收站恢复）
pub async fn soft_delete_owner_attachments(
    pool: &SqlitePool,
    owner_type: &str,
    owner_id: i64,
    now: i64,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE sys_attachments SET deleted_at = ? \
         WHERE owner_type = ? AND owner_id = ? AND deleted_at IS NULL",
    )
    .bind(now)
    .bind(owner_type)
    .bind(owner_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// 恢复某 owner 下软删的附件元数据
pub async fn restore_owner_attachments(
    pool: &SqlitePool,
    owner_type: &str,
    owner_id: i64,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE sys_attachments SET deleted_at = NULL \
         WHERE owner_type = ? AND owner_id = ? AND deleted_at IS NOT NULL",
    )
    .bind(owner_type)
    .bind(owner_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// 硬删某 owner 下全部附件行并删除磁盘目录
pub async fn hard_delete_owner_attachments(
    pool: &SqlitePool,
    app_data: &Path,
    owner_type: &str,
    owner_id: i64,
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM sys_attachments WHERE owner_type = ? AND owner_id = ?")
        .bind(owner_type)
        .bind(owner_id)
        .execute(pool)
        .await?;
    let dir = owner_dir(app_data, owner_type, owner_id);
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn list_attachments(
    db: tauri::State<'_, Db>,
    owner_type: String,
    owner_id: String,
) -> Result<Vec<AttachmentDto>, AppError> {
    let ot = parse_owner_type(&owner_type)?;
    let oid = id::parse_id(&owner_id)?;
    let app_data = app_data_from_db(&db)?;

    let rows: Vec<AttachmentRow> = sqlx::query_as(
        "SELECT id, owner_type, owner_id, filename, stored_name, mime_type, size_bytes, hash_sha256, created_at \
         FROM sys_attachments \
         WHERE owner_type = ? AND owner_id = ? AND deleted_at IS NULL \
         ORDER BY created_at ASC",
    )
    .bind(ot)
    .bind(oid)
    .fetch_all(&db.pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| row_to_dto(r, &app_data))
        .collect())
}

#[tauri::command]
pub async fn add_attachment(
    app: AppHandle,
    db: tauri::State<'_, Db>,
    owner_type: String,
    owner_id: String,
    source_path: String,
) -> Result<AttachmentDto, AppError> {
    let ot = parse_owner_type(&owner_type)?;
    let oid = id::parse_id(&owner_id)?;
    owner_exists(&db.pool, ot, oid).await?;

    let src = PathBuf::from(&source_path);
    let meta = std::fs::metadata(&src)?;
    if !meta.is_file() {
        return Err(AppError::Invalid("source is not a file".into()));
    }
    if meta.len() > MAX_ATTACHMENT_BYTES {
        return Err(AppError::Invalid("file exceeds 10MB".into()));
    }

    let filename = src
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| AppError::Invalid("invalid filename".into()))?
        .to_string();
    let ext = normalize_ext(&filename)
        .ok_or_else(|| AppError::Invalid("unsupported file type".into()))?;
    let mime = mime_for_ext(&ext).to_string();

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sys_attachments \
         WHERE owner_type = ? AND owner_id = ? AND deleted_at IS NULL",
    )
    .bind(ot)
    .bind(oid)
    .fetch_one(&db.pool)
    .await?;
    if count >= MAX_ATTACHMENTS_PER_OWNER {
        return Err(AppError::Invalid("attachment limit is 5".into()));
    }

    let bytes = std::fs::read(&src)?;
    let hash = hex::encode(Sha256::digest(&bytes));

    let aid = id::next_id()?;
    let stored_name = format!("{aid}.{ext}");
    validate_stored_name(&stored_name)?;

    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Invalid(format!("app_data_dir: {e}")))?;
    let dir = owner_dir(&app_data, ot, oid);
    std::fs::create_dir_all(&dir)?;
    let dest = dir.join(&stored_name);
    if let Err(e) = std::fs::write(&dest, &bytes) {
        return Err(e.into());
    }

    let now = now_ms();
    let size = bytes.len() as i64;
    let insert = sqlx::query(
        "INSERT INTO sys_attachments \
         (id, owner_type, owner_id, filename, stored_name, mime_type, size_bytes, hash_sha256, created_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(aid)
    .bind(ot)
    .bind(oid)
    .bind(&filename)
    .bind(&stored_name)
    .bind(&mime)
    .bind(size)
    .bind(&hash)
    .bind(now)
    .execute(&db.pool)
    .await;

    if let Err(e) = insert {
        let _ = std::fs::remove_file(&dest);
        return Err(e.into());
    }

    Ok(AttachmentDto {
        id: id::id_to_string(aid),
        owner_type: ot.to_string(),
        owner_id: id::id_to_string(oid),
        filename,
        mime_type: mime,
        size_bytes: size,
        hash_sha256: hash,
        created_at: now,
        missing: false,
    })
}

#[tauri::command]
pub async fn remove_attachment(
    db: tauri::State<'_, Db>,
    id: String,
) -> Result<(), AppError> {
    let aid = id::parse_id(&id)?;
    let app_data = app_data_from_db(&db)?;
    let row = fetch_attachment_row(&db.pool, aid).await?;
    let path = resolve_file_path(&app_data, &row)?;

    let now = now_ms();
    sqlx::query("UPDATE sys_attachments SET deleted_at = ? WHERE id = ? AND deleted_at IS NULL")
        .bind(now)
        .bind(aid)
        .execute(&db.pool)
        .await?;

    if path.exists() {
        let _ = std::fs::remove_file(&path);
    }
    Ok(())
}

#[tauri::command]
pub async fn get_attachment_path(
    db: tauri::State<'_, Db>,
    id: String,
) -> Result<String, AppError> {
    let aid = id::parse_id(&id)?;
    let app_data = app_data_from_db(&db)?;
    let row = fetch_attachment_row(&db.pool, aid).await?;
    let path = resolve_file_path(&app_data, &row)?;
    if !path.is_file() {
        return Err(AppError::NotFound(format!("attachment file {id}")));
    }
    Ok(path.display().to_string())
}

#[tauri::command]
pub async fn open_attachment(db: tauri::State<'_, Db>, id: String) -> Result<(), AppError> {
    let aid = id::parse_id(&id)?;
    let app_data = app_data_from_db(&db)?;
    let row = fetch_attachment_row(&db.pool, aid).await?;
    let path = resolve_file_path(&app_data, &row)?;
    if !path.is_file() {
        return Err(AppError::NotFound(format!("attachment file {id}")));
    }
    open_path_with_system(&path)
}

#[tauri::command]
pub async fn read_attachment_text(
    db: tauri::State<'_, Db>,
    id: String,
) -> Result<String, AppError> {
    let aid = id::parse_id(&id)?;
    let app_data = app_data_from_db(&db)?;
    let row = fetch_attachment_row(&db.pool, aid).await?;
    let ext = normalize_ext(&row.filename)
        .ok_or_else(|| AppError::Invalid("unsupported file type".into()))?;
    if ext != "md" && ext != "markdown" {
        return Err(AppError::Invalid("not a markdown attachment".into()));
    }
    let path = resolve_file_path(&app_data, &row)?;
    let text = std::fs::read_to_string(&path)?;
    Ok(text)
}

#[tauri::command]
pub async fn read_attachment_bytes(
    db: tauri::State<'_, Db>,
    id: String,
) -> Result<Vec<u8>, AppError> {
    let aid = id::parse_id(&id)?;
    let app_data = app_data_from_db(&db)?;
    let row = fetch_attachment_row(&db.pool, aid).await?;
    let path = resolve_file_path(&app_data, &row)?;
    let bytes = std::fs::read(&path)?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_ext() {
        assert!(normalize_ext("a.exe").is_none());
        assert_eq!(normalize_ext("a.PNG").as_deref(), Some("png"));
    }

    #[test]
    fn rejects_path_traversal_stored_name() {
        assert!(validate_stored_name("../x").is_err());
        assert!(validate_stored_name("abc.dat").is_ok());
    }

    #[test]
    fn previewable_set() {
        assert!(is_previewable("pdf"));
        assert!(!is_previewable("docx"));
    }
}
