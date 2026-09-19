use serde::{Deserialize, Serialize};
use sqlx::{FromRow, QueryBuilder, Sqlite, SqliteConnection, SqlitePool};

use crate::{error::AppError, id, id::now_ms, sqlite::Db};

const TASK_SELECT: &str = r#"
SELECT
  t.id, t.title, t.description, t.status, t.priority, t.due_at,
  t.project_id, p.name AS project_name, t.parent_id, t.sort_order,
  t.completed_at, t.source, t.created_at, t.updated_at,
  COALESCE((
    SELECT group_concat(g.name, ',')
    FROM todo_task_tags tt
    JOIN sys_tags g ON g.id = tt.tag_id
    WHERE tt.task_id = t.id
  ), '') AS tags_text
FROM todo_tasks t
LEFT JOIN sys_projects p ON p.id = t.project_id AND p.deleted_at IS NULL
"#;

/// 区分「字段缺省」与「显式 null」。serde 默认把两者都读成 `None`，
/// 会让「清空截止日 / 移出项目」静默失效。
fn double_option<'de, D, T>(de: D) -> Result<Option<Option<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::deserialize(de).map(Some)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskDto {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub due_at: Option<i64>,
    pub project_id: Option<String>,
    pub project_name: Option<String>,
    pub parent_id: Option<String>,
    pub sort_order: f64,
    pub completed_at: Option<i64>,
    pub source: String,
    pub tags: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, FromRow)]
struct TaskRow {
    id: i64,
    title: String,
    description: String,
    status: String,
    priority: String,
    due_at: Option<i64>,
    project_id: Option<i64>,
    project_name: Option<String>,
    parent_id: Option<i64>,
    sort_order: f64,
    completed_at: Option<i64>,
    source: String,
    created_at: i64,
    updated_at: i64,
    tags_text: String,
}

impl From<TaskRow> for TaskDto {
    fn from(r: TaskRow) -> Self {
        let tags = if r.tags_text.is_empty() {
            Vec::new()
        } else {
            r.tags_text.split(',').map(|s| s.to_string()).collect()
        };
        Self {
            id: id::id_to_string(r.id),
            title: r.title,
            description: r.description,
            status: r.status,
            priority: r.priority,
            due_at: r.due_at,
            project_id: r.project_id.map(id::id_to_string),
            project_name: r.project_name,
            parent_id: r.parent_id.map(id::id_to_string),
            sort_order: r.sort_order,
            completed_at: r.completed_at,
            source: r.source,
            tags,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListTodoQuery {
    pub perspective: Option<String>,
    pub keyword: Option<String>,
    pub priority: Option<String>,
}

fn validate_status(status: &str) -> Result<(), AppError> {
    match status {
        "todo" | "doing" | "done" | "cancelled" => Ok(()),
        _ => Err(AppError::Invalid(format!("unknown status: {status}"))),
    }
}

fn validate_priority(priority: &str) -> Result<(), AppError> {
    match priority {
        "high" | "medium" | "low" | "none" => Ok(()),
        _ => Err(AppError::Invalid(format!("unknown priority: {priority}"))),
    }
}

async fn fetch_task(pool: &SqlitePool, task_id: i64) -> Result<TaskDto, AppError> {
    let sql = format!("{TASK_SELECT} WHERE t.id = ? AND t.deleted_at IS NULL");
    let row: TaskRow = sqlx::query_as(&sql)
        .bind(task_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("task {task_id}")))?;
    Ok(TaskDto::from(row))
}

async fn replace_task_tags(
    conn: &mut SqliteConnection,
    task_id: i64,
    tag_names: &[String],
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM todo_task_tags WHERE task_id = ?")
        .bind(task_id)
        .execute(&mut *conn)
        .await?;

    let now = now_ms();
    for raw in tag_names {
        let name = raw.trim();
        if name.is_empty() {
            continue;
        }
        // 标签是硬删（见 delete_muse_tag），删除后由外键级联清掉关联，无需过滤 deleted_at
        let existing: Option<i64> = sqlx::query_scalar("SELECT id FROM sys_tags WHERE name = ?")
            .bind(name)
            .fetch_optional(&mut *conn)
            .await?;
        let tag_id = if let Some(id) = existing {
            id
        } else {
            let tid = id::next_id()?;
            sqlx::query(
                "INSERT INTO sys_tags (id, name, created_at, updated_at) VALUES (?, ?, ?, ?)",
            )
            .bind(tid)
            .bind(name)
            .bind(now)
            .bind(now)
            .execute(&mut *conn)
            .await?;
            tid
        };
        sqlx::query(
            "INSERT OR IGNORE INTO todo_task_tags (task_id, tag_id, created_at) VALUES (?, ?, ?)",
        )
        .bind(task_id)
        .bind(tag_id)
        .bind(now)
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

#[tauri::command]
pub async fn list_todo_tasks(
    db: tauri::State<'_, Db>,
    query: ListTodoQuery,
) -> Result<Vec<TaskDto>, AppError> {
    let perspective = query.perspective.as_deref().unwrap_or("all");
    let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(TASK_SELECT);
    qb.push(" WHERE t.deleted_at IS NULL AND t.archived = 0 AND t.parent_id IS NULL");

    match perspective {
        "inbox" => {
            qb.push(" AND t.project_id IS NULL AND t.due_at IS NULL AND t.status != 'done'");
        }
        "today" => {
            qb.push(
                " AND t.status != 'done' AND t.due_at IS NOT NULL \
                 AND t.due_at < (CAST(strftime('%s','now','start of day','+1 day') AS INTEGER) * 1000)",
            );
        }
        "upcoming" => {
            qb.push(
                " AND t.status != 'done' AND t.due_at IS NOT NULL \
                 AND t.due_at >= (CAST(strftime('%s','now','start of day','+1 day') AS INTEGER) * 1000)",
            );
        }
        "done" => {
            qb.push(" AND t.status = 'done'");
        }
        _ => {
            qb.push(" AND t.status != 'done'");
        }
    }

    if let Some(ref priority) = query.priority {
        let priority = priority.trim();
        if !priority.is_empty() && priority != "all" {
            validate_priority(priority)?;
            qb.push(" AND t.priority = ");
            qb.push_bind(priority.to_string());
        }
    }

    if let Some(ref kw) = query.keyword {
        let kw = kw.trim();
        if !kw.is_empty() {
            let pattern = format!("%{kw}%");
            qb.push(" AND (t.title LIKE ");
            qb.push_bind(pattern.clone());
            qb.push(" OR t.description LIKE ");
            qb.push_bind(pattern);
            qb.push(")");
        }
    }

    qb.push(" ORDER BY t.sort_order ASC, t.updated_at DESC");

    let rows: Vec<TaskRow> = qb.build_query_as().fetch_all(&db.pool).await?;
    Ok(rows.into_iter().map(TaskDto::from).collect())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTodoRequest {
    pub title: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub due_at: Option<i64>,
    pub project_id: Option<String>,
    pub parent_id: Option<String>,
    pub tag_names: Option<Vec<String>>,
    pub source: Option<String>,
}

#[tauri::command]
pub async fn create_todo_task(
    db: tauri::State<'_, Db>,
    req: CreateTodoRequest,
) -> Result<TaskDto, AppError> {
    let title = req.title.trim().to_string();
    if title.is_empty() {
        return Err(AppError::Invalid("title required".into()));
    }
    let status = req.status.as_deref().unwrap_or("todo");
    let priority = req.priority.as_deref().unwrap_or("none");
    validate_status(status)?;
    validate_priority(priority)?;
    let source = req.source.as_deref().unwrap_or("manual");
    match source {
        "manual" | "quick" | "import" | "api" => {}
        _ => return Err(AppError::Invalid(format!("unknown source: {source}"))),
    }

    let project_id = match req.project_id.as_deref() {
        Some(s) if !s.trim().is_empty() => Some(id::parse_id(s)?),
        _ => None,
    };

    let parent_id = match req.parent_id.as_deref() {
        Some(s) if !s.trim().is_empty() => {
            let pid = id::parse_id(s)?;
            // 只允许挂在顶层任务下（一层子待办）
            let parent_parent: Option<Option<i64>> = sqlx::query_scalar(
                "SELECT parent_id FROM todo_tasks WHERE id = ? AND deleted_at IS NULL",
            )
            .bind(pid)
            .fetch_optional(&db.pool)
            .await?;
            match parent_parent {
                Some(None) => Some(pid),
                Some(Some(_)) => {
                    return Err(AppError::Invalid("subtasks cannot nest".into()));
                }
                None => return Err(AppError::NotFound(format!("parent task {s}"))),
            }
        }
        _ => None,
    };

    let tid = id::next_id()?;
    let now = now_ms();
    let description = req.description.unwrap_or_default();

    let mut tx = db.pool.begin().await?;

    sqlx::query(
        "INSERT INTO todo_tasks \
         (id, title, description, status, priority, due_at, project_id, parent_id, sort_order, source, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, 0, ?, ?, ?)",
    )
    .bind(tid)
    .bind(&title)
    .bind(&description)
    .bind(status)
    .bind(priority)
    .bind(req.due_at)
    .bind(project_id)
    .bind(parent_id)
    .bind(source)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await?;

    if let Some(tags) = req.tag_names.as_ref() {
        replace_task_tags(&mut tx, tid, tags).await?;
    }

    tx.commit().await?;
    fetch_task(&db.pool, tid).await
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTodoRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    #[serde(default, deserialize_with = "double_option")]
    pub due_at: Option<Option<i64>>,
    #[serde(default, deserialize_with = "double_option")]
    pub project_id: Option<Option<String>>,
    pub tag_names: Option<Vec<String>>,
}

#[tauri::command]
pub async fn update_todo_task(
    db: tauri::State<'_, Db>,
    id: String,
    req: UpdateTodoRequest,
) -> Result<TaskDto, AppError> {
    let task_id = id::parse_id(&id)?;
    let now = now_ms();

    if let Some(ref status) = req.status {
        validate_status(status)?;
    }
    if let Some(ref priority) = req.priority {
        validate_priority(priority)?;
    }

    let title = match req.title {
        Some(ref raw) => {
            let title = raw.trim();
            if title.is_empty() {
                return Err(AppError::Invalid("title required".into()));
            }
            Some(title.to_string())
        }
        None => None,
    };

    let project_id = match req.project_id {
        Some(Some(ref s)) if !s.trim().is_empty() => Some(Some(id::parse_id(s)?)),
        Some(_) => Some(None),
        None => None,
    };

    let mut tx = db.pool.begin().await?;

    // 一条 UPDATE 写完所有字段：updated_at 恒写，故 rows_affected 兼作存在性判断
    let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new("UPDATE todo_tasks SET updated_at = ");
    qb.push_bind(now);
    if let Some(title) = title {
        qb.push(", title = ");
        qb.push_bind(title);
    }
    if let Some(description) = req.description {
        qb.push(", description = ");
        qb.push_bind(description);
    }
    if let Some(status) = req.status {
        let completed_at = if status == "done" { Some(now) } else { None };
        qb.push(", status = ");
        qb.push_bind(status);
        qb.push(", completed_at = ");
        qb.push_bind(completed_at);
    }
    if let Some(priority) = req.priority {
        qb.push(", priority = ");
        qb.push_bind(priority);
    }
    if let Some(due_at) = req.due_at {
        qb.push(", due_at = ");
        qb.push_bind(due_at);
    }
    if let Some(project_id) = project_id {
        qb.push(", project_id = ");
        qb.push_bind(project_id);
    }
    qb.push(" WHERE id = ");
    qb.push_bind(task_id);
    qb.push(" AND deleted_at IS NULL");

    let res = qb.build().execute(&mut *tx).await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("task {id}")));
    }

    if let Some(tags) = req.tag_names.as_ref() {
        replace_task_tags(&mut tx, task_id, tags).await?;
    }

    tx.commit().await?;
    fetch_task(&db.pool, task_id).await
}

#[tauri::command]
pub async fn complete_todo_task(
    db: tauri::State<'_, Db>,
    id: String,
    done: bool,
) -> Result<TaskDto, AppError> {
    let task_id = id::parse_id(&id)?;
    let now = now_ms();
    let (status, completed_at) = if done {
        ("done", Some(now))
    } else {
        ("todo", None)
    };
    let res = sqlx::query(
        "UPDATE todo_tasks SET status = ?, completed_at = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(status)
    .bind(completed_at)
    .bind(now)
    .bind(task_id)
    .execute(&db.pool)
    .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("task {id}")));
    }
    fetch_task(&db.pool, task_id).await
}

#[tauri::command]
pub async fn delete_todo_task(db: tauri::State<'_, Db>, id: String) -> Result<(), AppError> {
    let task_id = id::parse_id(&id)?;
    let now = now_ms();
    let res = sqlx::query(
        "UPDATE todo_tasks SET deleted_at = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(now)
    .bind(now)
    .bind(task_id)
    .execute(&db.pool)
    .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("task {id}")));
    }
    Ok(())
}

#[tauri::command]
pub async fn list_todo_subtasks(
    db: tauri::State<'_, Db>,
    parent_id: String,
) -> Result<Vec<TaskDto>, AppError> {
    let pid = id::parse_id(&parent_id)?;
    let sql = format!(
        "{TASK_SELECT}
         WHERE t.deleted_at IS NULL AND t.parent_id = ?
         ORDER BY t.sort_order ASC, t.created_at ASC"
    );
    let rows: Vec<TaskRow> = sqlx::query_as(&sql).bind(pid).fetch_all(&db.pool).await?;
    Ok(rows.into_iter().map(TaskDto::from).collect())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoExportDto {
    pub exported_at: i64,
    pub tasks: Vec<TaskDto>,
}

/// 导出全部未删除任务（含子待办），供前端落盘 JSON
#[tauri::command]
pub async fn export_todo_json(db: tauri::State<'_, Db>) -> Result<TodoExportDto, AppError> {
    let sql = format!(
        "{TASK_SELECT}
         WHERE t.deleted_at IS NULL
         ORDER BY t.parent_id IS NOT NULL, t.sort_order ASC, t.created_at ASC"
    );
    let rows: Vec<TaskRow> = sqlx::query_as(&sql).fetch_all(&db.pool).await?;
    Ok(TodoExportDto {
        exported_at: now_ms(),
        tasks: rows.into_iter().map(TaskDto::from).collect(),
    })
}

/// 5 个视角下的计数。字段名与前端的 `TodoPerspective` 联合类型一一对应，
/// 用 `camelCase` 序列化后，前端可直接 `counts[item.id]` 取值。
///
/// `all` 是 SQLite 保留字，SQL 侧用别名 `"all"`；Rust 字段不能叫 `all`，
/// 所以用 `all_count` + `sqlx(rename)` / `serde(rename)` 双向映射。
#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TodoCountsDto {
    pub inbox: i64,
    pub today: i64,
    pub upcoming: i64,
    #[sqlx(rename = "all")]
    #[serde(rename = "all")]
    pub all_count: i64,
    pub done: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoCountsQuery {
    pub priority: Option<String>,
}

/// 在「指定 priority 过滤下」统计每个 perspective 的任务数。
/// 与 `list_todo_tasks` 的 WHERE 语义完全对齐（不含 keyword，关键词只是搜索过滤不影响计数）。
#[tauri::command]
pub async fn get_todo_counts(
    db: tauri::State<'_, Db>,
    query: TodoCountsQuery,
) -> Result<TodoCountsDto, AppError> {
    // 把 priority 过滤拼成 SQL 片段（与 list_todo_tasks 完全一致）
    let mut priority_sql = String::new();
    if let Some(ref p) = query.priority {
        let p = p.trim();
        if !p.is_empty() && p != "all" {
            validate_priority(p)?;
            // p 已经被 validate_priority 校验过，不会出现 SQL 注入
            priority_sql.push_str(&format!(" AND t.priority = '{p}'"));
        }
    }

    let base =
        format!("t.deleted_at IS NULL AND t.archived = 0 AND t.parent_id IS NULL{priority_sql}");

    let sql = format!(
        r#"
        SELECT
          (SELECT COUNT(*) FROM todo_tasks t WHERE {base}
              AND t.project_id IS NULL AND t.due_at IS NULL AND t.status != 'done') AS inbox,
          (SELECT COUNT(*) FROM todo_tasks t WHERE {base}
              AND t.status != 'done' AND t.due_at IS NOT NULL
              AND t.due_at < (CAST(strftime('%s','now','start of day','+1 day') AS INTEGER) * 1000)) AS today,
          (SELECT COUNT(*) FROM todo_tasks t WHERE {base}
              AND t.status != 'done' AND t.due_at IS NOT NULL
              AND t.due_at >= (CAST(strftime('%s','now','start of day','+1 day') AS INTEGER) * 1000)) AS upcoming,
          (SELECT COUNT(*) FROM todo_tasks t WHERE {base}
              AND t.status != 'done') AS "all",
          (SELECT COUNT(*) FROM todo_tasks t WHERE {base}
              AND t.status = 'done') AS done
        "#
    );

    let row: TodoCountsDto = sqlx::query_as(&sql).fetch_one(&db.pool).await?;
    Ok(row)
}
