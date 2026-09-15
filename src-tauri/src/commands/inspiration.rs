use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

use crate::{
    error::AppError,
    id::{self, UNCLASSIFIED_CATEGORY_ID},
    sqlite::Db,
};

fn sid(id: i64) -> String {
    id::id_to_string(id)
}

// —— Categories ——

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryDto {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub sort_order: i64,
    pub inspiration_count: i64,
}

#[derive(Debug, FromRow)]
struct CategoryRow {
    id: i64,
    name: String,
    color: Option<String>,
    sort_order: i64,
    inspiration_count: i64,
}

#[tauri::command]
pub async fn list_inspiration_categories(
    db: tauri::State<'_, Db>,
) -> Result<Vec<CategoryDto>, AppError> {
    let rows: Vec<CategoryRow> = sqlx::query_as(
        r#"
        SELECT c.id, c.name, c.color, c.sort_order,
               (SELECT COUNT(*) FROM inspirations i WHERE i.category_id = c.id) AS inspiration_count
        FROM inspiration_categories c
        ORDER BY c.sort_order ASC, c.name ASC
        "#,
    )
    .fetch_all(&db.pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| CategoryDto {
            id: sid(r.id),
            name: r.name,
            color: r.color,
            sort_order: r.sort_order,
            inspiration_count: r.inspiration_count,
        })
        .collect())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertCategoryRequest {
    pub name: String,
    pub color: Option<String>,
}

#[tauri::command]
pub async fn create_inspiration_category(
    db: tauri::State<'_, Db>,
    req: UpsertCategoryRequest,
) -> Result<CategoryDto, AppError> {
    let name = req.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::Invalid("category name required".into()));
    }
    let id = id::next_id()?;
    sqlx::query(
        "INSERT INTO inspiration_categories (id, name, color) VALUES (?, ?, ?)",
    )
    .bind(id)
    .bind(&name)
    .bind(req.color.as_deref())
    .execute(&db.pool)
    .await?;

    Ok(CategoryDto {
        id: sid(id),
        name,
        color: req.color,
        sort_order: 0,
        inspiration_count: 0,
    })
}

#[tauri::command]
pub async fn update_inspiration_category(
    db: tauri::State<'_, Db>,
    id: String,
    req: UpsertCategoryRequest,
) -> Result<(), AppError> {
    let cid = id::parse_id(&id)?;
    let name = req.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::Invalid("category name required".into()));
    }
    let res = sqlx::query(
        "UPDATE inspiration_categories SET name = ?, color = ?, updated_at = datetime('now') WHERE id = ?",
    )
    .bind(&name)
    .bind(req.color.as_deref())
    .bind(cid)
    .execute(&db.pool)
    .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("category {id}")));
    }
    Ok(())
}

#[tauri::command]
pub async fn delete_inspiration_category(
    db: tauri::State<'_, Db>,
    id: String,
) -> Result<(), AppError> {
    let cid = id::parse_id(&id)?;
    if cid == UNCLASSIFIED_CATEGORY_ID {
        return Err(AppError::Invalid("cannot delete default category".into()));
    }
    let mut tx = db.pool.begin().await?;
    sqlx::query("UPDATE inspirations SET category_id = ?, updated_at = datetime('now') WHERE category_id = ?")
        .bind(UNCLASSIFIED_CATEGORY_ID)
        .bind(cid)
        .execute(&mut *tx)
        .await?;
    let res = sqlx::query("DELETE FROM inspiration_categories WHERE id = ?")
        .bind(cid)
        .execute(&mut *tx)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("category {id}")));
    }
    tx.commit().await?;
    Ok(())
}

// —— Tags ——

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TagDto {
    pub id: String,
    pub name: String,
}

#[derive(Debug, FromRow)]
struct TagRow {
    id: i64,
    name: String,
}

#[tauri::command]
pub async fn list_inspiration_tags(db: tauri::State<'_, Db>) -> Result<Vec<TagDto>, AppError> {
    let rows: Vec<TagRow> =
        sqlx::query_as("SELECT id, name FROM inspiration_tags ORDER BY name COLLATE NOCASE ASC")
            .fetch_all(&db.pool)
            .await?;
    Ok(rows
        .into_iter()
        .map(|r| TagDto {
            id: sid(r.id),
            name: r.name,
        })
        .collect())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertTagRequest {
    pub name: String,
}

#[tauri::command]
pub async fn create_inspiration_tag(
    db: tauri::State<'_, Db>,
    req: UpsertTagRequest,
) -> Result<TagDto, AppError> {
    let name = req.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::Invalid("tag name required".into()));
    }
    // 已存在则返回已有
    if let Some(existing) = sqlx::query_as::<_, TagRow>(
        "SELECT id, name FROM inspiration_tags WHERE name = ? COLLATE NOCASE",
    )
    .bind(&name)
    .fetch_optional(&db.pool)
    .await?
    {
        return Ok(TagDto {
            id: sid(existing.id),
            name: existing.name,
        });
    }
    let id = id::next_id()?;
    sqlx::query("INSERT INTO inspiration_tags (id, name) VALUES (?, ?)")
        .bind(id)
        .bind(&name)
        .execute(&db.pool)
        .await?;
    Ok(TagDto {
        id: sid(id),
        name,
    })
}

#[tauri::command]
pub async fn update_inspiration_tag(
    db: tauri::State<'_, Db>,
    id: String,
    req: UpsertTagRequest,
) -> Result<(), AppError> {
    let tid = id::parse_id(&id)?;
    let name = req.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::Invalid("tag name required".into()));
    }
    let res = sqlx::query(
        "UPDATE inspiration_tags SET name = ?, updated_at = datetime('now') WHERE id = ?",
    )
    .bind(&name)
    .bind(tid)
    .execute(&db.pool)
    .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("tag {id}")));
    }
    Ok(())
}

#[tauri::command]
pub async fn delete_inspiration_tag(
    db: tauri::State<'_, Db>,
    id: String,
) -> Result<(), AppError> {
    let tid = id::parse_id(&id)?;
    // map 行因 ON DELETE CASCADE 清除
    let res = sqlx::query("DELETE FROM inspiration_tags WHERE id = ?")
        .bind(tid)
        .execute(&db.pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("tag {id}")));
    }
    Ok(())
}

// —— Inspirations ——

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspirationSummaryDto {
    pub id: String,
    pub title: String,
    pub body_text: String,
    pub category_id: Option<String>,
    pub category_name: Option<String>,
    pub tag_ids: Vec<String>,
    pub tag_names: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspirationDetailDto {
    pub id: String,
    pub title: String,
    pub body_html: String,
    pub body_text: String,
    pub category_id: Option<String>,
    pub category_name: Option<String>,
    pub tag_ids: Vec<String>,
    pub tag_names: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, FromRow)]
struct InspirationListRow {
    id: i64,
    title: String,
    body_text: String,
    category_id: Option<i64>,
    category_name: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, FromRow)]
struct InspirationDetailRow {
    id: i64,
    title: String,
    body_html: String,
    body_text: String,
    category_id: Option<i64>,
    category_name: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListInspirationsQuery {
    pub category_id: Option<String>,
    pub tag_ids: Option<Vec<String>>,
    pub keyword: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertInspirationRequest {
    pub title: Option<String>,
    pub body_html: String,
    pub body_text: String,
    pub category_id: Option<String>,
    pub tag_ids: Option<Vec<String>>,
    pub tag_names: Option<Vec<String>>,
}

async fn load_tags_for(
    pool: &SqlitePool,
    inspiration_id: i64,
) -> Result<(Vec<String>, Vec<String>), AppError> {
    let rows: Vec<TagRow> = sqlx::query_as(
        r#"
        SELECT t.id, t.name
        FROM inspiration_tags t
        INNER JOIN inspiration_tag_map m ON m.tag_id = t.id
        WHERE m.inspiration_id = ?
        ORDER BY t.name COLLATE NOCASE ASC
        "#,
    )
    .bind(inspiration_id)
    .fetch_all(pool)
    .await?;
    Ok((
        rows.iter().map(|r| sid(r.id)).collect(),
        rows.into_iter().map(|r| r.name).collect(),
    ))
}

fn has_content(body_text: &str, body_html: &str) -> bool {
    let text_ok = !body_text.trim().is_empty();
    let img_ok = body_html.contains("<img") || body_html.contains("data:image/");
    text_ok || img_ok
}

fn resolve_title(title: Option<&str>, body_text: &str) -> String {
    let t = title.map(str::trim).unwrap_or("");
    if !t.is_empty() {
        return t.to_string();
    }
    let first = body_text.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
    let first = first.trim();
    if first.is_empty() {
        "无标题".to_string()
    } else {
        first.chars().take(40).collect()
    }
}

async fn ensure_tag_ids(
    pool: &SqlitePool,
    tag_ids: Option<Vec<String>>,
    tag_names: Option<Vec<String>>,
) -> Result<Vec<i64>, AppError> {
    let mut ids: Vec<i64> = Vec::new();
    if let Some(raw) = tag_ids {
        for s in raw {
            ids.push(id::parse_id(&s)?);
        }
    }
    if let Some(names) = tag_names {
        for name in names {
            let name = name.trim().to_string();
            if name.is_empty() {
                continue;
            }
            if let Some(existing) = sqlx::query_as::<_, TagRow>(
                "SELECT id, name FROM inspiration_tags WHERE name = ? COLLATE NOCASE",
            )
            .bind(&name)
            .fetch_optional(pool)
            .await?
            {
                if !ids.contains(&existing.id) {
                    ids.push(existing.id);
                }
                continue;
            }
            let nid = id::next_id()?;
            sqlx::query("INSERT INTO inspiration_tags (id, name) VALUES (?, ?)")
                .bind(nid)
                .bind(&name)
                .execute(pool)
                .await?;
            ids.push(nid);
        }
    }
    Ok(ids)
}

async fn replace_tag_map(
    pool: &SqlitePool,
    inspiration_id: i64,
    tag_ids: &[i64],
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM inspiration_tag_map WHERE inspiration_id = ?")
        .bind(inspiration_id)
        .execute(pool)
        .await?;
    for tid in tag_ids {
        sqlx::query(
            "INSERT OR IGNORE INTO inspiration_tag_map (inspiration_id, tag_id) VALUES (?, ?)",
        )
        .bind(inspiration_id)
        .bind(tid)
        .execute(pool)
        .await?;
    }
    Ok(())
}

#[tauri::command]
pub async fn list_inspirations(
    db: tauri::State<'_, Db>,
    query: ListInspirationsQuery,
) -> Result<Vec<InspirationSummaryDto>, AppError> {
    let mut sql = String::from(
        r#"
        SELECT i.id, i.title, i.body_text, i.category_id, c.name AS category_name,
               i.created_at, i.updated_at
        FROM inspirations i
        LEFT JOIN inspiration_categories c ON c.id = i.category_id
        WHERE 1=1
        "#,
    );
    let mut binds: Vec<String> = Vec::new();

    if let Some(cid) = query.category_id.as_deref().filter(|s| !s.is_empty() && *s != "all") {
        let id = id::parse_id(cid)?;
        sql.push_str(" AND i.category_id = ?");
        binds.push(id.to_string());
    }

    if let Some(keyword) = query.keyword.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        sql.push_str(" AND (i.title LIKE ? OR i.body_text LIKE ?)");
        let like = format!("%{keyword}%");
        binds.push(like.clone());
        binds.push(like);
    }

    if let Some(tag_ids) = &query.tag_ids {
        if !tag_ids.is_empty() {
            let parsed: Result<Vec<i64>, _> = tag_ids.iter().map(|s| id::parse_id(s)).collect();
            let parsed = parsed?;
            sql.push_str(" AND i.id IN (SELECT inspiration_id FROM inspiration_tag_map WHERE tag_id IN (");
            sql.push_str(&parsed.iter().map(|_| "?").collect::<Vec<_>>().join(","));
            sql.push_str(") GROUP BY inspiration_id HAVING COUNT(DISTINCT tag_id) = ?)");
            for t in &parsed {
                binds.push(t.to_string());
            }
            binds.push(parsed.len().to_string());
        }
    }

    sql.push_str(" ORDER BY i.updated_at DESC");

    let mut q = sqlx::query_as::<_, InspirationListRow>(&sql);
    for b in &binds {
        // bind as text then sqlite coerces for integers; for LIKE keep text
        q = q.bind(b);
    }
    // Problem: category_id and tag ids need integer binds. Using string bind for integers works in SQLite.
    let rows = q.fetch_all(&db.pool).await?;

    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        let (tag_ids, tag_names) = load_tags_for(&db.pool, r.id).await?;
        let summary_text: String = r.body_text.chars().take(160).collect();
        out.push(InspirationSummaryDto {
            id: sid(r.id),
            title: r.title,
            body_text: summary_text,
            category_id: r.category_id.map(sid),
            category_name: r.category_name,
            tag_ids,
            tag_names,
            created_at: r.created_at,
            updated_at: r.updated_at,
        });
    }
    Ok(out)
}

#[tauri::command]
pub async fn get_inspiration(
    db: tauri::State<'_, Db>,
    id: String,
) -> Result<InspirationDetailDto, AppError> {
    let iid = id::parse_id(&id)?;
    let row: Option<InspirationDetailRow> = sqlx::query_as(
        r#"
        SELECT i.id, i.title, i.body_html, i.body_text, i.category_id, c.name AS category_name,
               i.created_at, i.updated_at
        FROM inspirations i
        LEFT JOIN inspiration_categories c ON c.id = i.category_id
        WHERE i.id = ?
        "#,
    )
    .bind(iid)
    .fetch_optional(&db.pool)
    .await?;

    let Some(r) = row else {
        return Err(AppError::NotFound(format!("inspiration {id}")));
    };
    let (tag_ids, tag_names) = load_tags_for(&db.pool, r.id).await?;
    Ok(InspirationDetailDto {
        id: sid(r.id),
        title: r.title,
        body_html: r.body_html,
        body_text: r.body_text,
        category_id: r.category_id.map(sid),
        category_name: r.category_name,
        tag_ids,
        tag_names,
        created_at: r.created_at,
        updated_at: r.updated_at,
    })
}

#[tauri::command]
pub async fn create_inspiration(
    db: tauri::State<'_, Db>,
    req: UpsertInspirationRequest,
) -> Result<InspirationDetailDto, AppError> {
    if !has_content(&req.body_text, &req.body_html) {
        return Err(AppError::Invalid("inspiration content required".into()));
    }
    let category_id = match req.category_id.as_deref() {
        Some(s) if !s.is_empty() => id::parse_id(s)?,
        _ => UNCLASSIFIED_CATEGORY_ID,
    };
    let title = resolve_title(req.title.as_deref(), &req.body_text);
    let id = id::next_id()?;
    let tag_ids = ensure_tag_ids(&db.pool, req.tag_ids, req.tag_names).await?;

    sqlx::query(
        r#"
        INSERT INTO inspirations (id, title, body_html, body_text, category_id)
        VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(&title)
    .bind(&req.body_html)
    .bind(&req.body_text)
    .bind(category_id)
    .execute(&db.pool)
    .await?;

    replace_tag_map(&db.pool, id, &tag_ids).await?;
    get_inspiration(db, sid(id)).await
}

#[tauri::command]
pub async fn update_inspiration(
    db: tauri::State<'_, Db>,
    id: String,
    req: UpsertInspirationRequest,
) -> Result<InspirationDetailDto, AppError> {
    if !has_content(&req.body_text, &req.body_html) {
        return Err(AppError::Invalid("inspiration content required".into()));
    }
    let iid = id::parse_id(&id)?;
    let category_id = match req.category_id.as_deref() {
        Some(s) if !s.is_empty() => id::parse_id(s)?,
        _ => UNCLASSIFIED_CATEGORY_ID,
    };
    let title = resolve_title(req.title.as_deref(), &req.body_text);
    let tag_ids = ensure_tag_ids(&db.pool, req.tag_ids, req.tag_names).await?;

    let res = sqlx::query(
        r#"
        UPDATE inspirations
        SET title = ?, body_html = ?, body_text = ?, category_id = ?, updated_at = datetime('now')
        WHERE id = ?
        "#,
    )
    .bind(&title)
    .bind(&req.body_html)
    .bind(&req.body_text)
    .bind(category_id)
    .bind(iid)
    .execute(&db.pool)
    .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("inspiration {id}")));
    }
    replace_tag_map(&db.pool, iid, &tag_ids).await?;
    get_inspiration(db, id).await
}

#[tauri::command]
pub async fn delete_inspiration(
    db: tauri::State<'_, Db>,
    id: String,
) -> Result<(), AppError> {
    let iid = id::parse_id(&id)?;
    let res = sqlx::query("DELETE FROM inspirations WHERE id = ?")
        .bind(iid)
        .execute(&db.pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("inspiration {id}")));
    }
    Ok(())
}
