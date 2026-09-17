//! Muse 灵感库命令层。
//!
//! 数据模型见 `migrations/0003_muse.sql`：灵感（`muse_notes`）带状态机与来源，
//! 通过 `muse_note_tags` 关联标签、通过 `project_id` 归入项目，
//! 全文检索走 FTS5 虚表 `muse_notes_fts`（由触发器维护）。

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Sqlite, SqliteConnection, Transaction, query::QueryAs, sqlite::SqliteArguments};
use tauri::Manager;

use crate::{error::AppError, id, sqlite::Db};

/// 正文长度上限（BR-01-02）
const MAX_CONTENT_CHARS: usize = 10_000;
/// 单条灵感标签数上限（BR-01-03）
const MAX_TAGS_PER_NOTE: usize = 20;
/// 回收站保留天数（FR-09.3）
const TRASH_RETENTION_DAYS: i64 = 30;
/// 自动备份保留份数（FR-13.2）
const AUTO_BACKUP_KEEP: usize = 7;
/// 合法来源（附录 10.2）
const VALID_SOURCES: &[&str] = &[
    "quick", "manual", "clip", "book", "pod", "web", "api", "import",
];
/// 合法项目状态（FR-06.1）
const VALID_PROJECT_STATUSES: &[&str] = &["active", "paused", "done", "archived"];

/// 新建标签时按名称哈希取色，与原型 `tagColor()` 的调色板一致
const TAG_PALETTE: [&str; 10] = [
    "#5b5bd6", "#e0872b", "#22a06b", "#e5484d", "#3b82f6", "#8b5cf6", "#0d9488", "#d946ef",
    "#0891b2", "#ca8a04",
];

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default()
}

fn tag_color(name: &str) -> &'static str {
    let mut hash: u32 = 0;
    for ch in name.chars() {
        hash = hash.wrapping_mul(31).wrapping_add(ch as u32);
    }
    let idx = (hash as usize) % TAG_PALETTE.len();
    TAG_PALETTE.get(idx).copied().unwrap_or("#5b5bd6")
}

/// 折叠行内多余空白，但保留换行（捕捉浮层支持 Shift+Enter 换行）
fn normalize_content(raw: &str) -> String {
    raw.lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn truncate_chars(raw: &str, max: usize) -> String {
    raw.chars().take(max).collect()
}

// —— DTO ——

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteDto {
    pub id: String,
    pub content: String,
    pub status: String,
    pub source: String,
    pub source_url: Option<String>,
    pub project_id: Option<String>,
    pub project_name: Option<String>,
    pub tags: Vec<String>,
    pub pinned: bool,
    pub archived: bool,
    pub deleted_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, FromRow)]
struct NoteRow {
    id: i64,
    content: String,
    status: String,
    source: String,
    source_url: Option<String>,
    project_id: Option<i64>,
    project_name: Option<String>,
    tags_text: String,
    pinned: i64,
    archived_at: Option<i64>,
    deleted_at: Option<i64>,
    created_at: i64,
    updated_at: i64,
}

impl From<NoteRow> for NoteDto {
    fn from(row: NoteRow) -> Self {
        Self {
            id: id::id_to_string(row.id),
            content: row.content,
            status: row.status,
            source: row.source,
            source_url: row.source_url,
            project_id: row.project_id.map(id::id_to_string),
            project_name: row.project_name,
            tags: row
                .tags_text
                .split(',')
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect(),
            pinned: row.pinned != 0,
            archived: row.archived_at.is_some(),
            deleted_at: row.deleted_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInfoDto {
    pub path: String,
    pub file_name: String,
    pub size: u64,
    pub created_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub color: Option<String>,
    pub status: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MuseTagDto {
    pub id: String,
    pub name: String,
    pub color: String,
    pub note_count: i64,
}

#[derive(Debug, FromRow)]
struct TagStatRow {
    id: i64,
    name: String,
    color: Option<String>,
    note_count: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDto {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub status: String,
    pub note_count: i64,
    pub done_count: i64,
}

#[derive(Debug, FromRow)]
struct ProjectStatRow {
    id: i64,
    name: String,
    color: Option<String>,
    status: String,
    note_count: i64,
    done_count: i64,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct NoteCountsDto {
    pub inbox: i64,
    pub today: i64,
    pub all: i64,
    pub unsorted: i64,
    pub archive: i64,
    pub trash: i64,
}

// —— 请求体 ——

/// 列表筛选。`kind` 对应左栏导航；`tag` / `project` 通过 `value` 传名称。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteFilter {
    pub kind: String,
    pub value: Option<String>,
    /// 状态 chip：`all` 或 `new` / `check` / `doing` / `done`
    pub status: Option<String>,
    pub keyword: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateNoteRequest {
    pub content: String,
    /// 浮层中已选中的建议标签；正文里的 `#标签` 会自动合并进来
    pub tags: Option<Vec<String>>,
    /// 项目名；正文里的 `@项目` 优先级更低
    pub project: Option<String>,
    pub source: Option<String>,
    pub source_url: Option<String>,
}

// —— 内部帮手 ——

/// SQL 动态绑定值：列表查询里既有整数也有文本
enum Bind {
    Int(i64),
    Text(String),
}

type NoteQuery<'q> = QueryAs<'q, Sqlite, NoteRow, SqliteArguments<'q>>;

fn bind_all<'q>(mut q: NoteQuery<'q>, binds: Vec<Bind>) -> NoteQuery<'q> {
    for b in binds {
        q = match b {
            Bind::Int(v) => q.bind(v),
            Bind::Text(v) => q.bind(v),
        };
    }
    q
}

const NOTE_COLUMNS: &str = r#"
    n.id, n.content, n.status, n.source, n.source_url,
    n.project_id, n.project_name, n.tags_text, n.pinned,
    n.archived_at, n.deleted_at, n.created_at, n.updated_at
"#;

/// 把用户输入转成安全的 FTS5 查询串。
///
/// 每个词都包成带前缀匹配的短语（`"词"*`），内部双引号翻倍转义，
/// 这样 `*` `(` `OR` 等 FTS 语法字符不会导致语法错误（TC-FR07-12）。
fn fts_query(raw: &str) -> Option<String> {
    let phrases: Vec<String> = raw
        .split_whitespace()
        .map(|token| token.replace('"', "\"\""))
        .filter(|token| !token.is_empty())
        .map(|token| format!("\"{token}\"*"))
        .collect();

    if phrases.is_empty() {
        None
    } else {
        Some(phrases.join(" "))
    }
}

struct ParsedCapture {
    content: String,
    tags: Vec<String>,
    project: Option<String>,
}

/// 解析捕捉语法：`#([^\s#@]+)` 为标签，`@([^\s#@]+)` 为项目，其余为正文（FR-01.4）
fn parse_capture(raw: &str) -> ParsedCapture {
    let mut content = String::new();
    let mut tags: Vec<String> = Vec::new();
    let mut project: Option<String> = None;
    let mut chars = raw.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != '#' && ch != '@' {
            content.push(ch);
            continue;
        }

        let mut token = String::new();
        while let Some(&next) = chars.peek() {
            if next.is_whitespace() || next == '#' || next == '@' {
                break;
            }
            token.push(next);
            chars.next();
        }

        // 孤立的 # / @ 当作普通字符保留
        if token.is_empty() {
            content.push(ch);
            continue;
        }

        if ch == '#' {
            if !tags.iter().any(|t| t == &token) {
                tags.push(token);
            }
        } else if project.is_none() {
            project = Some(token);
        }
    }

    ParsedCapture {
        content: normalize_content(&content),
        tags,
        project,
    }
}

async fn ensure_tag(tx: &mut SqliteConnection, name: &str, now: i64) -> Result<i64, AppError> {
    if let Some(existing) = sqlx::query_scalar::<_, i64>("SELECT id FROM muse_tags WHERE name = ?")
        .bind(name)
        .fetch_optional(&mut *tx)
        .await?
    {
        return Ok(existing);
    }

    let id = id::next_id()?;
    sqlx::query("INSERT INTO muse_tags (id, name, color, created_at, updated_at) VALUES (?, ?, ?, ?, ?)")
        .bind(id)
        .bind(name)
        .bind(tag_color(name))
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;
    Ok(id)
}

async fn ensure_project(tx: &mut SqliteConnection, name: &str, now: i64) -> Result<i64, AppError> {
    if let Some(existing) =
        sqlx::query_scalar::<_, i64>("SELECT id FROM muse_projects WHERE name = ?")
            .bind(name)
            .fetch_optional(&mut *tx)
            .await?
    {
        return Ok(existing);
    }

    let id = id::next_id()?;
    sqlx::query(
        "INSERT INTO muse_projects (id, name, created_at, updated_at) VALUES (?, ?, ?, ?)",
    )
    .bind(id)
    .bind(name)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await?;
    Ok(id)
}

async fn fetch_note(conn: &mut SqliteConnection, note_id: i64) -> Result<NoteDto, AppError> {
    let sql = format!("SELECT {NOTE_COLUMNS} FROM v_muse_notes_full n WHERE n.id = ?");
    let row: Option<NoteRow> = sqlx::query_as(&sql)
        .bind(note_id)
        .fetch_optional(&mut *conn)
        .await?;

    row.map(NoteDto::from)
        .ok_or_else(|| AppError::NotFound(format!("note {note_id}")))
}

/// 更新灵感的单个字段后返回最新快照，找不到则报 NotFound
async fn touch_and_fetch(
    db: &Db,
    note_id: i64,
    sql: &str,
    bind: Bind,
) -> Result<NoteDto, AppError> {
    let mut conn = db.pool.acquire().await?;
    let query = sqlx::query(sql);
    let query = match bind {
        Bind::Int(v) => query.bind(v),
        Bind::Text(v) => query.bind(v),
    };
    let res = query
        .bind(now_ms())
        .bind(note_id)
        .execute(&mut *conn)
        .await?;

    if res.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("note {note_id}")));
    }
    fetch_note(&mut conn, note_id).await
}

// —— 查询 ——

#[tauri::command]
pub async fn list_muse_notes(
    db: tauri::State<'_, Db>,
    filter: NoteFilter,
) -> Result<Vec<NoteDto>, AppError> {
    let mut sql = format!("SELECT {NOTE_COLUMNS} FROM v_muse_notes_full n WHERE 1 = 1");
    let mut binds: Vec<Bind> = Vec::new();

    match filter.kind.as_str() {
        "archive" => sql.push_str(" AND n.archived_at IS NOT NULL"),
        "inbox" => sql.push_str(" AND n.archived_at IS NULL AND n.status = 'new'"),
        "today" => sql.push_str(
            " AND n.archived_at IS NULL \
             AND n.created_at >= (CAST(strftime('%s','now','start of day') AS INTEGER) * 1000)",
        ),
        "unsorted" => {
            sql.push_str(" AND n.archived_at IS NULL AND n.tags_text = '' AND n.project_id IS NULL");
        }
        "tag" => {
            let name = filter
                .value
                .as_deref()
                .ok_or_else(|| AppError::Invalid("tag filter requires a value".into()))?;
            sql.push_str(
                " AND n.archived_at IS NULL AND n.id IN (\
                   SELECT nt.note_id FROM muse_note_tags nt \
                   JOIN muse_tags t ON t.id = nt.tag_id WHERE t.name = ?)",
            );
            binds.push(Bind::Text(name.to_string()));
        }
        "project" => {
            let name = filter
                .value
                .as_deref()
                .ok_or_else(|| AppError::Invalid("project filter requires a value".into()))?;
            sql.push_str(
                " AND n.archived_at IS NULL \
                 AND n.project_id = (SELECT id FROM muse_projects WHERE name = ?)",
            );
            binds.push(Bind::Text(name.to_string()));
        }
        // `all` 与 `review` 都是「未归档的全部灵感」，回顾的随机顺序在前端洗牌
        _ => sql.push_str(" AND n.archived_at IS NULL"),
    }

    if let Some(status) = filter
        .status
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty() && *s != "all")
    {
        sql.push_str(" AND n.status = ?");
        binds.push(Bind::Text(status.to_string()));
    }

    if let Some(keyword) = filter.keyword.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        match fts_query(keyword) {
            Some(q) => {
                sql.push_str(
                    " AND n.id IN (SELECT note_id FROM muse_notes_fts WHERE muse_notes_fts MATCH ?)",
                );
                binds.push(Bind::Text(q));
            }
            None => {
                // 纯符号等无法构成 FTS 词元时退回 LIKE，保证不报错
                sql.push_str(" AND n.content LIKE ?");
                binds.push(Bind::Text(format!("%{keyword}%")));
            }
        }
    }

    sql.push_str(" ORDER BY n.pinned DESC, n.created_at DESC");

    let rows = bind_all(sqlx::query_as(&sql), binds)
        .fetch_all(&db.pool)
        .await?;
    Ok(rows.into_iter().map(NoteDto::from).collect())
}

#[tauri::command]
pub async fn get_muse_note(db: tauri::State<'_, Db>, id: String) -> Result<NoteDto, AppError> {
    let note_id = id::parse_id(&id)?;
    let mut conn = db.pool.acquire().await?;
    fetch_note(&mut conn, note_id).await
}

#[tauri::command]
pub async fn get_muse_note_counts(db: tauri::State<'_, Db>) -> Result<NoteCountsDto, AppError> {
    let counts: NoteCountsDto = sqlx::query_as(
        r#"
        SELECT
          (SELECT COUNT(*) FROM v_muse_inbox)    AS inbox,
          (SELECT COUNT(*) FROM v_muse_today)    AS today,
          (SELECT COUNT(*) FROM v_muse_notes_full WHERE archived_at IS NULL) AS "all",
          (SELECT COUNT(*) FROM v_muse_unsorted) AS unsorted,
          (SELECT COUNT(*) FROM v_muse_archive)  AS archive,
          (SELECT COUNT(*) FROM muse_notes WHERE deleted_at IS NOT NULL) AS trash
        "#,
    )
    .fetch_one(&db.pool)
    .await?;
    Ok(counts)
}

#[tauri::command]
pub async fn list_muse_tags(db: tauri::State<'_, Db>) -> Result<Vec<MuseTagDto>, AppError> {
    let rows: Vec<TagStatRow> = sqlx::query_as(
        "SELECT id, name, color, note_count FROM v_muse_tag_stats \
         ORDER BY note_count DESC, name ASC",
    )
    .fetch_all(&db.pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| MuseTagDto {
            id: id::id_to_string(r.id),
            color: r.color.unwrap_or_else(|| tag_color(&r.name).to_string()),
            name: r.name,
            note_count: r.note_count,
        })
        .collect())
}

#[tauri::command]
pub async fn list_muse_projects(db: tauri::State<'_, Db>) -> Result<Vec<ProjectDto>, AppError> {
    let rows: Vec<ProjectStatRow> = sqlx::query_as(
        "SELECT s.id, s.name, s.color, s.status, s.note_count, s.done_count \
         FROM v_muse_project_stats s \
         JOIN muse_projects p ON p.id = s.id \
         WHERE p.deleted_at IS NULL \
         ORDER BY s.note_count DESC, s.name ASC",
    )
    .fetch_all(&db.pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| ProjectDto {
            id: id::id_to_string(r.id),
            name: r.name,
            color: r.color,
            status: r.status,
            note_count: r.note_count,
            done_count: r.done_count,
        })
        .collect())
}

/// 关联灵感推荐：共享标签最多者优先（FR-04.6）
#[tauri::command]
pub async fn list_muse_related_notes(
    db: tauri::State<'_, Db>,
    id: String,
    limit: Option<i64>,
) -> Result<Vec<NoteDto>, AppError> {
    let note_id = id::parse_id(&id)?;
    let sql = format!(
        r#"
        SELECT {NOTE_COLUMNS}
        FROM v_muse_notes_full n
        JOIN muse_note_tags nt ON nt.note_id = n.id
        WHERE nt.tag_id IN (SELECT tag_id FROM muse_note_tags WHERE note_id = ?)
          AND n.id <> ?
          AND n.archived_at IS NULL
        GROUP BY n.id
        ORDER BY COUNT(nt.tag_id) DESC, n.updated_at DESC
        LIMIT ?
        "#
    );

    let rows: Vec<NoteRow> = sqlx::query_as(&sql)
        .bind(note_id)
        .bind(note_id)
        .bind(limit.unwrap_or(3).clamp(1, 20))
        .fetch_all(&db.pool)
        .await?;
    Ok(rows.into_iter().map(NoteDto::from).collect())
}

// —— 写入 ——

#[tauri::command]
pub async fn create_muse_note(
    db: tauri::State<'_, Db>,
    req: CreateNoteRequest,
) -> Result<NoteDto, AppError> {
    let parsed = parse_capture(&req.content);
    if parsed.content.is_empty() {
        return Err(AppError::Invalid("content required".into()));
    }
    let content = truncate_chars(&parsed.content, MAX_CONTENT_CHARS);

    // 显式选中的标签在前，正文解析出的标签补在后面，去重后截到上限
    let mut tags: Vec<String> = Vec::new();
    for name in req.tags.unwrap_or_default().into_iter().chain(parsed.tags) {
        let name = name.trim().trim_start_matches('#').to_string();
        if !name.is_empty() && !tags.contains(&name) {
            tags.push(name);
        }
    }
    tags.truncate(MAX_TAGS_PER_NOTE);

    let project = req
        .project
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .or(parsed.project);

    let now = now_ms();
    let note_id = id::next_id()?;
    let mut tx: Transaction<'_, Sqlite> = db.pool.begin().await?;

    let project_id = match project {
        Some(name) => Some(ensure_project(&mut tx, &name, now).await?),
        None => None,
    };

    sqlx::query(
        "INSERT INTO muse_notes (id, content, status, source, source_url, project_id, created_at, updated_at) \
         VALUES (?, ?, 'new', ?, ?, ?, ?, ?)",
    )
    .bind(note_id)
    .bind(&content)
    .bind(req.source.as_deref().unwrap_or("quick"))
    .bind(req.source_url.as_deref())
    .bind(project_id)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await?;

    for name in &tags {
        let tag_id = ensure_tag(&mut tx, name, now).await?;
        sqlx::query(
            "INSERT OR IGNORE INTO muse_note_tags (note_id, tag_id, created_at) VALUES (?, ?, ?)",
        )
        .bind(note_id)
        .bind(tag_id)
        .bind(now)
        .execute(&mut *tx)
        .await?;
    }

    let note = fetch_note(&mut tx, note_id).await?;
    tx.commit().await?;
    Ok(note)
}

#[tauri::command]
pub async fn update_muse_note_content(
    db: tauri::State<'_, Db>,
    id: String,
    content: String,
) -> Result<NoteDto, AppError> {
    let note_id = id::parse_id(&id)?;
    let content = truncate_chars(&normalize_content(&content), MAX_CONTENT_CHARS);
    if content.is_empty() {
        return Err(AppError::Invalid("content required".into()));
    }

    touch_and_fetch(
        &db,
        note_id,
        "UPDATE muse_notes SET content = ?, updated_at = ? WHERE id = ?",
        Bind::Text(content),
    )
    .await
}

#[tauri::command]
pub async fn update_muse_note_status(
    db: tauri::State<'_, Db>,
    id: String,
    status: String,
) -> Result<NoteDto, AppError> {
    let note_id = id::parse_id(&id)?;
    if !matches!(status.as_str(), "new" | "check" | "doing" | "done") {
        return Err(AppError::Invalid(format!("unknown status: {status}")));
    }

    touch_and_fetch(
        &db,
        note_id,
        "UPDATE muse_notes SET status = ?, updated_at = ? WHERE id = ?",
        Bind::Text(status),
    )
    .await
}

/// 设置项目：传项目名，不存在则自动创建；传空则移出项目（FR-04.5）
#[tauri::command]
pub async fn update_muse_note_project(
    db: tauri::State<'_, Db>,
    id: String,
    project: Option<String>,
) -> Result<NoteDto, AppError> {
    let note_id = id::parse_id(&id)?;
    let name = project.map(|p| p.trim().to_string()).filter(|p| !p.is_empty());

    let now = now_ms();
    let mut tx: Transaction<'_, Sqlite> = db.pool.begin().await?;

    let project_id = match name {
        Some(name) => Some(ensure_project(&mut tx, &name, now).await?),
        None => None,
    };

    let res = sqlx::query("UPDATE muse_notes SET project_id = ?, updated_at = ? WHERE id = ?")
        .bind(project_id)
        .bind(now)
        .bind(note_id)
        .execute(&mut *tx)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("note {id}")));
    }

    let note = fetch_note(&mut tx, note_id).await?;
    tx.commit().await?;
    Ok(note)
}

#[tauri::command]
pub async fn add_muse_tag_to_note(
    db: tauri::State<'_, Db>,
    note_id: String,
    tag_name: String,
) -> Result<NoteDto, AppError> {
    let nid = id::parse_id(&note_id)?;
    let name = tag_name.trim().trim_start_matches('#').trim().to_string();
    if name.is_empty() {
        return Err(AppError::Invalid("tag name required".into()));
    }

    let existing: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM muse_note_tags WHERE note_id = ?")
            .bind(nid)
            .fetch_one(&db.pool)
            .await?;
    if existing >= MAX_TAGS_PER_NOTE as i64 {
        return Err(AppError::Invalid(format!(
            "a note can hold at most {MAX_TAGS_PER_NOTE} tags"
        )));
    }

    let now = now_ms();
    let mut tx: Transaction<'_, Sqlite> = db.pool.begin().await?;
    let tag_id = ensure_tag(&mut tx, &name, now).await?;
    sqlx::query(
        "INSERT OR IGNORE INTO muse_note_tags (note_id, tag_id, created_at) VALUES (?, ?, ?)",
    )
    .bind(nid)
    .bind(tag_id)
    .bind(now)
    .execute(&mut *tx)
    .await?;
    sqlx::query("UPDATE muse_notes SET updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(nid)
        .execute(&mut *tx)
        .await?;

    let note = fetch_note(&mut tx, nid).await?;
    tx.commit().await?;
    Ok(note)
}

#[tauri::command]
pub async fn remove_muse_tag_from_note(
    db: tauri::State<'_, Db>,
    note_id: String,
    tag_name: String,
) -> Result<NoteDto, AppError> {
    let nid = id::parse_id(&note_id)?;
    let now = now_ms();

    let mut tx: Transaction<'_, Sqlite> = db.pool.begin().await?;
    sqlx::query(
        "DELETE FROM muse_note_tags WHERE note_id = ? \
         AND tag_id = (SELECT id FROM muse_tags WHERE name = ?)",
    )
    .bind(nid)
    .bind(tag_name.trim().trim_start_matches('#'))
    .execute(&mut *tx)
    .await?;
    sqlx::query("UPDATE muse_notes SET updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(nid)
        .execute(&mut *tx)
        .await?;

    let note = fetch_note(&mut tx, nid).await?;
    tx.commit().await?;
    Ok(note)
}

#[tauri::command]
pub async fn archive_muse_note(db: tauri::State<'_, Db>, id: String) -> Result<NoteDto, AppError> {
    let note_id = id::parse_id(&id)?;
    touch_and_fetch(
        &db,
        note_id,
        "UPDATE muse_notes SET archived_at = ?, updated_at = ? WHERE id = ?",
        Bind::Int(now_ms()),
    )
    .await
}

#[tauri::command]
pub async fn unarchive_muse_note(db: tauri::State<'_, Db>, id: String) -> Result<NoteDto, AppError> {
    let note_id = id::parse_id(&id)?;
    let mut conn = db.pool.acquire().await?;
    let res = sqlx::query("UPDATE muse_notes SET archived_at = NULL, updated_at = ? WHERE id = ?")
        .bind(now_ms())
        .bind(note_id)
        .execute(&mut *conn)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("note {id}")));
    }
    fetch_note(&mut conn, note_id).await
}

/// 软删除：写入 `deleted_at`，灵感移出所有视图但留在回收站（FR-09.2）
#[tauri::command]
pub async fn delete_muse_note(db: tauri::State<'_, Db>, id: String) -> Result<(), AppError> {
    let note_id = id::parse_id(&id)?;
    let now = now_ms();
    let res = sqlx::query("UPDATE muse_notes SET deleted_at = ?, updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(now)
        .bind(note_id)
        .execute(&db.pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("note {id}")));
    }
    Ok(())
}

fn validate_source(source: &str) -> Result<(), AppError> {
    if VALID_SOURCES.contains(&source) {
        Ok(())
    } else {
        Err(AppError::Invalid(format!("unknown source: {source}")))
    }
}

fn validate_project_status(status: &str) -> Result<(), AppError> {
    if VALID_PROJECT_STATUSES.contains(&status) {
        Ok(())
    } else {
        Err(AppError::Invalid(format!("unknown project status: {status}")))
    }
}

fn backup_dir(app_data: &Path) -> PathBuf {
    app_data.join("muse-backups")
}

fn file_mtime_ms(path: &Path) -> i64 {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

async fn fetch_tag(db: &Db, tag_id: i64) -> Result<MuseTagDto, AppError> {
    let row: TagStatRow = sqlx::query_as(
        "SELECT id, name, color, note_count FROM v_muse_tag_stats WHERE id = ?",
    )
    .bind(tag_id)
    .fetch_optional(&db.pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("tag {tag_id}")))?;

    Ok(MuseTagDto {
        id: id::id_to_string(row.id),
        color: row.color.unwrap_or_else(|| tag_color(&row.name).to_string()),
        name: row.name,
        note_count: row.note_count,
    })
}

async fn fetch_project(db: &Db, project_id: i64) -> Result<ProjectDto, AppError> {
    let row: ProjectStatRow = sqlx::query_as(
        "SELECT s.id, s.name, s.color, s.status, s.note_count, s.done_count \
         FROM v_muse_project_stats s \
         JOIN muse_projects p ON p.id = s.id \
         WHERE s.id = ? AND p.deleted_at IS NULL",
    )
    .bind(project_id)
    .fetch_optional(&db.pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("project {project_id}")))?;

    Ok(ProjectDto {
        id: id::id_to_string(row.id),
        name: row.name,
        color: row.color,
        status: row.status,
        note_count: row.note_count,
        done_count: row.done_count,
    })
}

/// 改看来源（FR-01）
#[tauri::command]
pub async fn update_muse_note_source(
    db: tauri::State<'_, Db>,
    id: String,
    source: String,
) -> Result<NoteDto, AppError> {
    validate_source(&source)?;
    let note_id = id::parse_id(&id)?;
    touch_and_fetch(
        &db,
        note_id,
        "UPDATE muse_notes SET source = ?, updated_at = ? WHERE id = ?",
        Bind::Text(source),
    )
    .await
}

/// 回收站列表（FR-09.3）
#[tauri::command]
pub async fn list_muse_trash(db: tauri::State<'_, Db>) -> Result<Vec<NoteDto>, AppError> {
    let rows: Vec<NoteRow> = sqlx::query_as(
        r#"
        SELECT
          n.id, n.content, n.status, n.source, n.source_url,
          n.project_id, p.name AS project_name,
          COALESCE((
            SELECT group_concat(t.name, ',')
            FROM muse_note_tags nt
            JOIN muse_tags t ON t.id = nt.tag_id
            WHERE nt.note_id = n.id
          ), '') AS tags_text,
          n.pinned, n.archived_at, n.deleted_at, n.created_at, n.updated_at
        FROM muse_notes n
        LEFT JOIN muse_projects p ON p.id = n.project_id
        WHERE n.deleted_at IS NOT NULL
        ORDER BY n.deleted_at DESC
        "#,
    )
    .fetch_all(&db.pool)
    .await?;

    Ok(rows.into_iter().map(NoteDto::from).collect())
}

/// 从回收站恢复（清空 deleted_at）
#[tauri::command]
pub async fn restore_muse_note(db: tauri::State<'_, Db>, id: String) -> Result<NoteDto, AppError> {
    let note_id = id::parse_id(&id)?;
    let mut conn = db.pool.acquire().await?;
    let res = sqlx::query("UPDATE muse_notes SET deleted_at = NULL, updated_at = ? WHERE id = ?")
        .bind(now_ms())
        .bind(note_id)
        .execute(&mut *conn)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("note {id}")));
    }
    fetch_note(&mut conn, note_id).await
}

/// 彻底删除单条
#[tauri::command]
pub async fn purge_muse_note(db: tauri::State<'_, Db>, id: String) -> Result<(), AppError> {
    let note_id = id::parse_id(&id)?;
    let res = sqlx::query("DELETE FROM muse_notes WHERE id = ? AND deleted_at IS NOT NULL")
        .bind(note_id)
        .execute(&db.pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("trash note {id}")));
    }
    Ok(())
}

/// 清空回收站
#[tauri::command]
pub async fn empty_muse_trash(db: tauri::State<'_, Db>) -> Result<u64, AppError> {
    let res = sqlx::query("DELETE FROM muse_notes WHERE deleted_at IS NOT NULL")
        .execute(&db.pool)
        .await?;
    Ok(res.rows_affected())
}

/// 清理超过 30 天的软删除灵感（启动时调用）
#[tauri::command]
pub async fn purge_expired_muse_notes(db: tauri::State<'_, Db>) -> Result<u64, AppError> {
    let cutoff = now_ms().saturating_sub(TRASH_RETENTION_DAYS * 24 * 60 * 60 * 1000);
    let res = sqlx::query("DELETE FROM muse_notes WHERE deleted_at IS NOT NULL AND deleted_at < ?")
        .bind(cutoff)
        .execute(&db.pool)
        .await?;
    Ok(res.rows_affected())
}

/// 标签改名（全局生效，并触达 FTS 重建）
#[tauri::command]
pub async fn rename_muse_tag(
    db: tauri::State<'_, Db>,
    id: String,
    name: String,
) -> Result<MuseTagDto, AppError> {
    let tag_id = id::parse_id(&id)?;
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::Invalid("tag name required".into()));
    }

    let conflict: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM muse_tags WHERE name = ? AND id != ?",
    )
    .bind(&name)
    .bind(tag_id)
    .fetch_optional(&db.pool)
    .await?;
    if conflict.is_some() {
        return Err(AppError::Invalid(format!("tag already exists: {name}")));
    }

    let mut tx = db.pool.begin().await?;
    let res = sqlx::query("UPDATE muse_tags SET name = ?, updated_at = ? WHERE id = ?")
        .bind(&name)
        .bind(now_ms())
        .bind(tag_id)
        .execute(&mut *tx)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("tag {id}")));
    }

    // 触发 notes UPDATE → FTS 用新标签名重建
    sqlx::query(
        "UPDATE muse_notes SET updated_at = ? \
         WHERE id IN (SELECT note_id FROM muse_note_tags WHERE tag_id = ?)",
    )
    .bind(now_ms())
    .bind(tag_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    fetch_tag(&db, tag_id).await
}

/// 标签着色
#[tauri::command]
pub async fn update_muse_tag_color(
    db: tauri::State<'_, Db>,
    id: String,
    color: String,
) -> Result<MuseTagDto, AppError> {
    let tag_id = id::parse_id(&id)?;
    let color = color.trim().to_string();
    if color.is_empty() {
        return Err(AppError::Invalid("color required".into()));
    }

    let res = sqlx::query("UPDATE muse_tags SET color = ?, updated_at = ? WHERE id = ?")
        .bind(&color)
        .bind(now_ms())
        .bind(tag_id)
        .execute(&db.pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("tag {id}")));
    }
    fetch_tag(&db, tag_id).await
}

/// 合并标签：from → to，再删 from（FR-05.3）
#[tauri::command]
pub async fn merge_muse_tags(
    db: tauri::State<'_, Db>,
    from_id: String,
    to_id: String,
) -> Result<(), AppError> {
    let from = id::parse_id(&from_id)?;
    let to = id::parse_id(&to_id)?;
    if from == to {
        return Err(AppError::Invalid("cannot merge a tag into itself".into()));
    }

    let mut tx = db.pool.begin().await?;

    let from_exists: Option<i64> =
        sqlx::query_scalar("SELECT id FROM muse_tags WHERE id = ?")
            .bind(from)
            .fetch_optional(&mut *tx)
            .await?;
    let to_exists: Option<i64> = sqlx::query_scalar("SELECT id FROM muse_tags WHERE id = ?")
        .bind(to)
        .fetch_optional(&mut *tx)
        .await?;
    if from_exists.is_none() {
        return Err(AppError::NotFound(format!("tag {from_id}")));
    }
    if to_exists.is_none() {
        return Err(AppError::NotFound(format!("tag {to_id}")));
    }

    sqlx::query(
        "INSERT OR IGNORE INTO muse_note_tags (note_id, tag_id, created_at) \
         SELECT note_id, ?, created_at FROM muse_note_tags WHERE tag_id = ?",
    )
    .bind(to)
    .bind(from)
    .execute(&mut *tx)
    .await?;

    sqlx::query("DELETE FROM muse_note_tags WHERE tag_id = ?")
        .bind(from)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM muse_tags WHERE id = ?")
        .bind(from)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}

/// 删除标签：只解绑，不删灵感（FR-05）
#[tauri::command]
pub async fn delete_muse_tag(db: tauri::State<'_, Db>, id: String) -> Result<(), AppError> {
    let tag_id = id::parse_id(&id)?;
    let res = sqlx::query("DELETE FROM muse_tags WHERE id = ?")
        .bind(tag_id)
        .execute(&db.pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("tag {id}")));
    }
    Ok(())
}

/// 新建项目（设置页入口）
#[tauri::command]
pub async fn create_muse_project(
    db: tauri::State<'_, Db>,
    name: String,
    color: Option<String>,
) -> Result<ProjectDto, AppError> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::Invalid("project name required".into()));
    }

    let existing: Option<i64> =
        sqlx::query_scalar("SELECT id FROM muse_projects WHERE name = ? AND deleted_at IS NULL")
            .bind(&name)
            .fetch_optional(&db.pool)
            .await?;
    if let Some(pid) = existing {
        return fetch_project(&db, pid).await;
    }

    let pid = id::next_id()?;
    let now = now_ms();
    sqlx::query(
        "INSERT INTO muse_projects (id, name, color, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(pid)
    .bind(&name)
    .bind(color.as_deref())
    .bind(now)
    .bind(now)
    .execute(&db.pool)
    .await?;

    fetch_project(&db, pid).await
}

/// 更新项目（改名 / 颜色 / 状态 / 描述）
#[tauri::command]
pub async fn update_muse_project(
    db: tauri::State<'_, Db>,
    id: String,
    req: UpdateProjectRequest,
) -> Result<ProjectDto, AppError> {
    let project_id = id::parse_id(&id)?;

    if let Some(ref status) = req.status {
        validate_project_status(status)?;
    }
    if let Some(ref name) = req.name {
        let name = name.trim();
        if name.is_empty() {
            return Err(AppError::Invalid("project name required".into()));
        }
        let conflict: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM muse_projects WHERE name = ? AND id != ? AND deleted_at IS NULL",
        )
        .bind(name)
        .bind(project_id)
        .fetch_optional(&db.pool)
        .await?;
        if conflict.is_some() {
            return Err(AppError::Invalid(format!("project already exists: {name}")));
        }
    }

    let mut tx = db.pool.begin().await?;
    let now = now_ms();

    if let Some(name) = req.name.as_ref().map(|s| s.trim().to_string()) {
        let res = sqlx::query(
            "UPDATE muse_projects SET name = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(&name)
        .bind(now)
        .bind(project_id)
        .execute(&mut *tx)
        .await?;
        if res.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("project {id}")));
        }
        // 项目名变更 → 触达 FTS
        sqlx::query("UPDATE muse_notes SET updated_at = ? WHERE project_id = ?")
            .bind(now)
            .bind(project_id)
            .execute(&mut *tx)
            .await?;
    }
    if let Some(color) = &req.color {
        sqlx::query(
            "UPDATE muse_projects SET color = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(color)
        .bind(now)
        .bind(project_id)
        .execute(&mut *tx)
        .await?;
    }
    if let Some(status) = &req.status {
        sqlx::query(
            "UPDATE muse_projects SET status = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(status)
        .bind(now)
        .bind(project_id)
        .execute(&mut *tx)
        .await?;
    }
    if let Some(description) = &req.description {
        sqlx::query(
            "UPDATE muse_projects SET description = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(description)
        .bind(now)
        .bind(project_id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    fetch_project(&db, project_id).await
}

/// 删除项目：仅当无灵感关联（含归档，不含已进回收站的）
#[tauri::command]
pub async fn delete_muse_project(db: tauri::State<'_, Db>, id: String) -> Result<(), AppError> {
    let project_id = id::parse_id(&id)?;
    let linked: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM muse_notes WHERE project_id = ? AND deleted_at IS NULL",
    )
    .bind(project_id)
    .fetch_one(&db.pool)
    .await?;
    if linked > 0 {
        return Err(AppError::Invalid(format!(
            "project still has {linked} notes; unlink them first"
        )));
    }

    let now = now_ms();
    let res = sqlx::query(
        "UPDATE muse_projects SET deleted_at = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(now)
    .bind(now)
    .bind(project_id)
    .execute(&db.pool)
    .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("project {id}")));
    }
    Ok(())
}

async fn vacuum_into(db: &Db, dest: &Path) -> Result<(), AppError> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if dest.exists() {
        std::fs::remove_file(dest)?;
    }
    let dest_str = dest
        .to_str()
        .ok_or_else(|| AppError::Invalid("backup path is not valid UTF-8".into()))?
        .replace('\'', "''");
    // VACUUM INTO 不能用 bind 参数；路径已做单引号转义
    let sql = format!("VACUUM INTO '{dest_str}'");
    sqlx::query(&sql).execute(&db.pool).await?;
    Ok(())
}

fn prune_auto_backups(dir: &Path, keep: usize) -> Result<(), AppError> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("muse_auto_") && n.ends_with(".db"))
        })
        .collect();

    entries.sort_by_key(|e| std::cmp::Reverse(file_mtime_ms(&e.path())));
    for entry in entries.into_iter().skip(keep) {
        let _ = std::fs::remove_file(entry.path());
    }
    Ok(())
}

/// 手动 / 指定路径备份（VACUUM INTO）
#[tauri::command]
pub async fn backup_muse_db(
    app: tauri::AppHandle,
    db: tauri::State<'_, Db>,
    path: Option<String>,
) -> Result<String, AppError> {
    let dest = if let Some(p) = path.filter(|s| !s.trim().is_empty()) {
        PathBuf::from(p)
    } else {
        let app_data = app.path().app_data_dir()?;
        let dir = backup_dir(&app_data);
        std::fs::create_dir_all(&dir)?;
        let stamp = now_ms();
        dir.join(format!("muse_manual_{stamp}.db"))
    };

    vacuum_into(&db, &dest).await?;
    Ok(dest.display().to_string())
}

/// 启动自动备份，保留最近 7 份
#[tauri::command]
pub async fn auto_backup_muse_db(
    app: tauri::AppHandle,
    db: tauri::State<'_, Db>,
) -> Result<String, AppError> {
    let app_data = app.path().app_data_dir()?;
    let dir = backup_dir(&app_data);
    std::fs::create_dir_all(&dir)?;
    let stamp = now_ms();
    let dest = dir.join(format!("muse_auto_{stamp}.db"));
    vacuum_into(&db, &dest).await?;
    prune_auto_backups(&dir, AUTO_BACKUP_KEEP)?;
    Ok(dest.display().to_string())
}

/// 列出备份目录中的文件
#[tauri::command]
pub async fn list_muse_backups(app: tauri::AppHandle) -> Result<Vec<BackupInfoDto>, AppError> {
    let app_data = app.path().app_data_dir()?;
    let dir = backup_dir(&app_data);
    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut items = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("db") {
            continue;
        }
        let meta = entry.metadata()?;
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("backup.db")
            .to_string();
        items.push(BackupInfoDto {
            path: path.display().to_string(),
            file_name,
            size: meta.len(),
            created_at: file_mtime_ms(&path),
        });
    }
    items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(items)
}

/// 校验备份并写入待恢复文件；下次启动（或调用 restart）后生效
#[tauri::command]
pub async fn restore_muse_db(app: tauri::AppHandle, path: String) -> Result<(), AppError> {
    let src = PathBuf::from(&path);
    if !src.exists() {
        return Err(AppError::NotFound(format!("backup {path}")));
    }

    // 轻量校验：能打开且含 muse_notes
    let options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(&src)
        .read_only(true);
    let conn = sqlx::SqlitePool::connect_with(options).await?;
    let ok: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'muse_notes'",
    )
    .fetch_one(&conn)
    .await?;
    conn.close().await;
    if ok == 0 {
        return Err(AppError::Invalid("backup is not a valid Muse database".into()));
    }

    let app_data = app.path().app_data_dir()?;
    let pending = app_data.join("shixu.db.pending_restore");
    if pending.exists() {
        std::fs::remove_file(&pending)?;
    }
    std::fs::copy(&src, &pending)?;
    Ok(())
}

#[tauri::command]
pub fn restart_app(app: tauri::AppHandle) {
    app.restart();
}

/// 启动维护：自动备份 + 回收站过期清理（异步、不阻塞 UI）
pub async fn run_muse_startup_maintenance(app: tauri::AppHandle) -> Result<(), AppError> {
    let db = app.try_state::<Db>().ok_or_else(|| {
        AppError::Invalid("database not ready".into())
    })?;

    let app_data = app.path().app_data_dir()?;
    let dir = backup_dir(&app_data);
    std::fs::create_dir_all(&dir)?;
    let stamp = now_ms();
    let dest = dir.join(format!("muse_auto_{stamp}.db"));
    vacuum_into(&db, &dest).await?;
    prune_auto_backups(&dir, AUTO_BACKUP_KEEP)?;

    let cutoff = now_ms().saturating_sub(TRASH_RETENTION_DAYS * 24 * 60 * 60 * 1000);
    let purged = sqlx::query("DELETE FROM muse_notes WHERE deleted_at IS NOT NULL AND deleted_at < ?")
        .bind(cutoff)
        .execute(&db.pool)
        .await?
        .rows_affected();
    if purged > 0 {
        log::info!("Purged {purged} expired Muse trash notes");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tags_and_project_out_of_content() {
        let parsed = parse_capture("捕捉成本决定记录量 #产品 #交互 @灵感工具");
        assert_eq!(parsed.content, "捕捉成本决定记录量");
        assert_eq!(parsed.tags, vec!["产品", "交互"]);
        assert_eq!(parsed.project.as_deref(), Some("灵感工具"));
    }

    #[test]
    fn dedupes_tags_and_keeps_first_project() {
        let parsed = parse_capture("#产品 想法 #产品 @A @B");
        assert_eq!(parsed.tags, vec!["产品"]);
        assert_eq!(parsed.project.as_deref(), Some("A"));
        assert_eq!(parsed.content, "想法");
    }

    #[test]
    fn keeps_bare_symbols_as_text() {
        let parsed = parse_capture("C# 与 # 号");
        assert!(parsed.tags.is_empty());
        assert_eq!(parsed.content, "C# 与 # 号");
    }

    #[test]
    fn preserves_line_breaks() {
        let parsed = parse_capture("第一行\n第二行");
        assert_eq!(parsed.content, "第一行\n第二行");
    }

    #[test]
    fn escapes_fts_special_characters() {
        assert_eq!(fts_query("灵感"), Some("\"灵感\"*".into()));
        assert_eq!(fts_query("a\"b"), Some("\"a\"\"b\"*".into()));
        assert_eq!(fts_query("   "), None);
    }
}
