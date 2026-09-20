# 待办与灵感附件 实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 为待办与灵感提供共用附件「资料袋」：捕获/详情可添加；图片·md·pdf 应用内预览，其余系统打开；单文件 ≤10MB、单条最多 5 个。

**架构：** SQLite `sys_attachments` 存元数据；文件落盘 `app_data/attachments/{todo|muse}/{owner_id}/{stored_name}`；Rust 共用 commands；Vue 共用 `Attachment*` 组件挂到捕获对话框与详情面板。Muse 正文 base64 插图不改动。

**技术栈：** Tauri 2、sqlx/sqlite、Vue 3、Pinia、`@tauri-apps/plugin-dialog`、`marked`（已有）、雪花 ID

**规格：** `docs/superpowers/specs/2026-09-20-attachments-design.md`

---

## 文件结构

| 文件 | 职责 |
|---|---|
| `src-tauri/migrations/0007_sys_attachments.sql` | 建表 + 索引 |
| `src-tauri/src/commands/attachments.rs` | 校验、路径、CRUD、预览读、系统打开 |
| `src-tauri/src/commands/mod.rs` | `mod attachments; pub use` |
| `src-tauri/src/lib.rs` | 注册 invoke handlers |
| `src-tauri/src/sqlite.rs` | `attachments_dir`；`clear_user_data`/`remove_runtime_data` 清理附件 |
| `src-tauri/src/commands/todo.rs` | `delete_todo_task` 挂钩清附件 |
| `src-tauri/src/commands/muse.rs` | soft/purge/empty/expired 挂钩；备份/恢复拷附件目录 |
| `src-tauri/Cargo.toml` | 增加 `sha2`、`hex`（算 hash） |
| `src/lib/attachments.ts` | 类型、常量、invoke 封装、预览分流辅助 |
| `src/components/attachments/AttachmentList.vue` | 列表 + 删除 + 点击分流 |
| `src/components/attachments/AttachmentPicker.vue` | 选文件/拖拽，发出 pending 或直接 add |
| `src/components/attachments/AttachmentPreview.vue` | Dialog：图 / md / pdf |
| `src/components/todo/TodoCaptureDialog.vue` | 挂迷你附件条，创建后批量 add |
| `src/pages/todo.vue` | 详情区挂完整附件 |
| `src/components/muse/MuseCaptureDialog.vue` | 同上 |
| `src/components/muse/MuseDetailPane.vue` | 详情区挂完整附件 |
| `src/i18n/locales/{zh,en,es}.json` | `attachments.*` 文案 |
| `src/lib/attachments.test.ts` | 扩展名白名单 / 预览分流纯函数测试 |

---

### 任务 1：Migration `sys_attachments`

**文件：**
- 创建：`src-tauri/migrations/0007_sys_attachments.sql`

- [ ] **步骤 1：写入迁移**

```sql
-- 共用附件元数据（文件落盘见 app_data/attachments/）
CREATE TABLE IF NOT EXISTS sys_attachments (
  id INTEGER PRIMARY KEY NOT NULL,
  owner_type TEXT NOT NULL CHECK (owner_type IN ('todo', 'muse')),
  owner_id INTEGER NOT NULL,
  filename TEXT NOT NULL,
  stored_name TEXT NOT NULL,
  mime_type TEXT NOT NULL,
  size_bytes INTEGER NOT NULL,
  hash_sha256 TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  deleted_at INTEGER
);

CREATE INDEX IF NOT EXISTS idx_sys_attachments_owner
  ON sys_attachments(owner_type, owner_id)
  WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_sys_attachments_hash
  ON sys_attachments(hash_sha256);
```

- [ ] **步骤 2：Commit**

```bash
git add src-tauri/migrations/0007_sys_attachments.sql
git commit -m "feat(附件): 添加 sys_attachments 迁移"
```

---

### 任务 2：Rust 附件模块（校验 + 命令）

**文件：**
- 创建：`src-tauri/src/commands/attachments.rs`
- 修改：`src-tauri/src/commands/mod.rs`、`src-tauri/src/lib.rs`、`src-tauri/src/sqlite.rs`、`src-tauri/Cargo.toml`

- [ ] **步骤 1：依赖**

在 `Cargo.toml` `[dependencies]` 增加：

```toml
sha2 = "0.10"
hex = "0.4"
```

- [ ] **步骤 2：`sqlite.rs` 增加目录助手**

```rust
pub fn attachments_dir(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("attachments")
}

pub fn attachments_dir_name() -> &'static str {
    "attachments"
}
```

并在 `remove_runtime_data` 中删除 `attachments_dir`（若存在）；在 `clear_user_data` 的 `TABLES` 数组加入 `"sys_attachments"`（靠前，无 FK 到业务表亦可任意顺序，但建议在 notes/tasks 之前）。

- [ ] **步骤 3：编写 `attachments.rs` 核心校验（可单测）**

常量与函数（放在同一文件顶部）：

```rust
pub const MAX_ATTACHMENT_BYTES: u64 = 10 * 1024 * 1024;
pub const MAX_ATTACHMENTS_PER_OWNER: i64 = 5;

const ALLOWED_EXT: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp",
    "pdf", "md", "markdown", "docx",
    "zip", "7z", "rar",
];

pub fn normalize_ext(filename: &str) -> Option<String> {
    let name = std::path::Path::new(filename)
        .file_name()?
        .to_str()?;
    let ext = std::path::Path::new(name)
        .extension()?
        .to_str()?
        .to_ascii_lowercase();
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
    matches!(ext, "png" | "jpg" | "jpeg" | "gif" | "webp" | "pdf" | "md" | "markdown")
}

/// 拒绝含 `..` 或绝对路径段的 stored_name
pub fn validate_stored_name(name: &str) -> Result<(), AppError> {
    if name.is_empty() || name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err(AppError::Invalid("invalid stored_name".into()));
    }
    Ok(())
}
```

- [ ] **步骤 4：单元测试（同文件 `#[cfg(test)]`）**

```rust
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
```

运行：

```bash
cd src-tauri && cargo test commands::attachments::tests -- --nocapture
```

预期：PASS

- [ ] **步骤 5：实现 DTO 与命令**

```rust
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
    pub missing: bool, // 库有盘无
}

fn parse_owner_type(s: &str) -> Result<&str, AppError> {
    match s {
        "todo" | "muse" => Ok(s),
        _ => Err(AppError::Invalid(format!("unknown owner_type: {s}"))),
    }
}

async fn owner_exists(pool: &sqlx::SqlitePool, owner_type: &str, owner_id: i64) -> Result<(), AppError> {
    let sql = match owner_type {
        "todo" => "SELECT 1 FROM todo_tasks WHERE id = ? AND deleted_at IS NULL",
        "muse" => "SELECT 1 FROM muse_notes WHERE id = ? AND deleted_at IS NULL",
        _ => unreachable!(),
    };
    let ok: Option<i64> = sqlx::query_scalar(sql).bind(owner_id).fetch_optional(pool).await?;
    if ok.is_none() {
        return Err(AppError::NotFound(format!("{owner_type} {owner_id}")));
    }
    Ok(())
}

fn owner_dir(app_data: &Path, owner_type: &str, owner_id: i64) -> PathBuf {
    sqlite::attachments_dir(app_data)
        .join(owner_type)
        .join(owner_id.to_string())
}
```

命令签名（实现要点写在注释里，编码时按此行为）：

| 命令 | 入参 | 行为 |
|---|---|---|
| `list_attachments` | `owner_type`, `owner_id` | 查未软删；对每个检查磁盘是否存在，设 `missing` |
| `add_attachment` | `owner_type`, `owner_id`, `source_path` | 校验 owner → 读源文件大小 ≤10MB → ext 白名单 → count &lt; 5 → 算 sha256 → `stored_name = "{id}.{ext}"` → 拷贝到 owner_dir → INSERT；任一步失败删已写文件 |
| `remove_attachment` | `id` | 软删 + 删磁盘文件（文件不存在也算成功） |
| `get_attachment_path` | `id` | 返回绝对路径字符串（校验未软删、无 traversal） |
| `open_attachment` | `id` | 用系统打开（同 `reveal_data_dir` 的 explorer/open/xdg-open 模式） |
| `read_attachment_text` | `id` | 仅 md/markdown；UTF-8 文本，超限已由大小保证 |
| `read_attachment_bytes` | `id` | 返回 `Vec<u8>`（预览图/pdf 备用） |

`add_attachment` 伪代码关键路径：

```rust
#[tauri::command]
pub async fn add_attachment(
    app: tauri::AppHandle,
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
        "SELECT COUNT(*) FROM sys_attachments WHERE owner_type = ? AND owner_id = ? AND deleted_at IS NULL",
    )
    .bind(ot)
    .bind(oid)
    .fetch_one(&db.pool)
    .await?;
    if count >= MAX_ATTACHMENTS_PER_OWNER {
        return Err(AppError::Invalid("attachment limit is 5".into()));
    }

    let bytes = std::fs::read(&src)?;
    use sha2::{Digest, Sha256};
    let hash = hex::encode(Sha256::digest(&bytes));

    let aid = id::next_id()?;
    let stored_name = format!("{aid}.{ext}");
    validate_stored_name(&stored_name)?;

    let app_data = app.path().app_data_dir()
        .map_err(|e| AppError::Invalid(format!("app_data_dir: {e}")))?;
    let dir = owner_dir(&app_data, ot, oid);
    std::fs::create_dir_all(&dir)?;
    let dest = dir.join(&stored_name);
    if let Err(e) = std::fs::write(&dest, &bytes) {
        return Err(e.into());
    }

    let now = id::now_ms();
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
    .bind(bytes.len() as i64)
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
        owner_id: owner_id.clone(),
        filename,
        mime_type: mime,
        size_bytes: bytes.len() as i64,
        hash_sha256: hash,
        created_at: now,
        missing: false,
    })
}
```

其余命令按任务 2 表格实现。提供公开辅助供 todo/muse 调用：

```rust
/// 软删某 owner 下全部附件并删磁盘目录
pub async fn purge_owner_attachments(
    pool: &sqlx::SqlitePool,
    app_data: &Path,
    owner_type: &str,
    owner_id: i64,
) -> Result<(), AppError> { /* UPDATE deleted_at; remove_dir_all(owner_dir) */ }

/// 按附件 id 列表硬删文件（purge note 后）
pub async fn hard_delete_owner_attachments(
    pool: &sqlx::SqlitePool,
    app_data: &Path,
    owner_type: &str,
    owner_id: i64,
) -> Result<(), AppError> { /* DELETE FROM sys_attachments; remove_dir_all */ }
```

- [ ] **步骤 6：注册**

`mod.rs`：`mod attachments; pub use attachments::*;`  
`lib.rs` `generate_handler!` 加入全部 7 个命令。

- [ ] **步骤 7：编译**

```bash
cd src-tauri && cargo test commands::attachments::tests && cargo check
```

预期：通过

- [ ] **步骤 8：Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/commands/attachments.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs src-tauri/src/sqlite.rs
git commit -m "feat(附件): 实现共用附件命令与校验"
```

---

### 任务 3：删除路径挂钩 + 备份/恢复

**文件：**
- 修改：`src-tauri/src/commands/todo.rs`（`delete_todo_task`）
- 修改：`src-tauri/src/commands/muse.rs`（`delete_muse_note`、`purge_muse_note`、`empty_muse_trash`、`purge_expired_muse_notes`、`backup_muse_db`、`auto_backup_muse_db`、`restore_muse_db`）

- [ ] **步骤 1：待办删除**

在 `delete_todo_task` 软删任务成功后，调用 `purge_owner_attachments`（需要 `AppHandle` 取 `app_data`——给命令增加 `app: AppHandle` 参数，或从 `db.path.parent()` 推导 app_data）。**优先用 `db.path.parent()`**，避免改前端 invoke 签名：

```rust
let app_data = db.path.parent().ok_or_else(|| AppError::Invalid("db parent".into()))?;
attachments::hard_delete_owner_attachments(&db.pool, app_data, "todo", task_id).await?;
```

说明：待办当前是软删且无回收站恢复 UI；规格要求删 owner 后清文件。对 todo 用 **硬删附件行 + 删目录**（任务已不可见）。若希望与软删一致可先 `purge_owner_attachments`（软删元数据+删文件）；推荐硬删以免残留行。

- [ ] **步骤 2：Muse 删除**

- `delete_muse_note`（进回收站）：`purge_owner_attachments` —— **软删元数据并删磁盘文件**（回收站不保留附件文件，节省空间；恢复后附件列表为空可接受，或改为仅软删元数据保留文件——**本计划选：进回收站时保留磁盘文件与元数据软删标记，恢复时 `deleted_at = NULL` 恢复附件**）。

更清晰策略（按此实现）：

| 事件 | 附件行为 |
|---|---|
| Muse 软删（进回收站） | 附件 `deleted_at` 同步软删，**文件保留** |
| Muse 恢复 | 附件 `deleted_at` 清空 |
| Muse 彻底删除 / 清空回收站 / 过期清理 | `hard_delete_owner_attachments` |
| Todo 删除 | `hard_delete_owner_attachments` |

实现辅助：

```rust
pub async fn soft_delete_owner_attachments(pool: &SqlitePool, owner_type: &str, owner_id: i64, now: i64) -> Result<(), AppError>
pub async fn restore_owner_attachments(pool: &SqlitePool, owner_type: &str, owner_id: i64) -> Result<(), AppError>
pub async fn hard_delete_owner_attachments(...) // DELETE rows + remove_dir_all
```

- [ ] **步骤 3：备份附带 attachments**

在 `vacuum_into` 成功后：

```rust
fn copy_attachments_sidecar(app_data: &Path, backup_db_path: &Path) -> Result<(), AppError> {
    let src = sqlite::attachments_dir(app_data);
    if !src.exists() {
        return Ok(());
    }
    let stem = backup_db_path.file_stem().and_then(|s| s.to_str()).unwrap_or("backup");
    let dest = backup_db_path
        .parent()
        .unwrap_or(Path::new("."))
        .join(format!("{stem}_attachments"));
    if dest.exists() {
        std::fs::remove_dir_all(&dest)?;
    }
    copy_dir_recursive(&src, &dest)?;
    Ok(())
}
```

`backup_muse_db` / `auto_backup_muse_db` 在写完 `.db` 后调用。

- [ ] **步骤 4：恢复**

`restore_muse_db`：校验通过后，除拷贝 pending db 外：

```rust
let sidecar = src.with_file_name(format!(
    "{}_attachments",
    src.file_stem().and_then(|s| s.to_str()).unwrap_or("backup")
));
let pending_att = app_data.join("attachments.pending");
if pending_att.exists() {
    std::fs::remove_dir_all(&pending_att)?;
}
if sidecar.exists() {
    copy_dir_recursive(&sidecar, &pending_att)?;
}
```

在 `sqlite::apply_pending_restore`：应用 pending db 后，若存在 `attachments.pending`，则替换 `attachments/`（先删旧再 rename）。

- [ ] **步骤 5：`cargo check` + Commit**

```bash
git add src-tauri/src/commands/todo.rs src-tauri/src/commands/muse.rs src-tauri/src/commands/attachments.rs src-tauri/src/sqlite.rs
git commit -m "feat(附件): 删除挂钩与备份恢复附带附件目录"
```

---

### 任务 4：前端 lib + 纯函数测试

**文件：**
- 创建：`src/lib/attachments.ts`、`src/lib/attachments.test.ts`

- [ ] **步骤 1：先写失败测试**

```ts
import { describe, expect, it } from 'vitest'
import { attachmentExt, canPreviewAttachment, formatAttachmentSize } from './attachments'

describe('attachments helpers', () => {
  it('detects previewable types', () => {
    expect(canPreviewAttachment('photo.PNG')).toBe(true)
    expect(canPreviewAttachment('a.docx')).toBe(false)
    expect(canPreviewAttachment('x.pdf')).toBe(true)
  })

  it('normalizes extension', () => {
    expect(attachmentExt('a.Markdown')).toBe('markdown')
    expect(attachmentExt('noext')).toBe(null)
  })

  it('formats size', () => {
    expect(formatAttachmentSize(512)).toMatch(/512/)
    expect(formatAttachmentSize(2048)).toMatch(/2/)
  })
})
```

运行：`pnpm test src/lib/attachments.test.ts`  
预期：FAIL（模块不存在）

- [ ] **步骤 2：实现 `src/lib/attachments.ts`**

```ts
import { convertFileSrc, invoke } from '@tauri-apps/api/core'

export type AttachmentOwnerType = 'todo' | 'muse'

export interface Attachment {
  id: string
  ownerType: AttachmentOwnerType
  ownerId: string
  filename: string
  mimeType: string
  sizeBytes: number
  hashSha256: string
  createdAt: number
  missing: boolean
}

/** 捕获阶段尚未落库的本地文件 */
export interface PendingAttachment {
  path: string
  filename: string
  sizeBytes: number
}

export const MAX_ATTACHMENT_BYTES = 10 * 1024 * 1024
export const MAX_ATTACHMENTS_PER_OWNER = 5

const PREVIEW_EXT = new Set(['png', 'jpg', 'jpeg', 'gif', 'webp', 'pdf', 'md', 'markdown'])
const ALLOWED_EXT = new Set([
  ...PREVIEW_EXT,
  'docx', 'zip', '7z', 'rar',
])

export function attachmentExt(filename: string): string | null {
  const base = filename.split(/[/\\]/).pop() ?? filename
  const i = base.lastIndexOf('.')
  if (i <= 0) return null
  const ext = base.slice(i + 1).toLowerCase()
  return ALLOWED_EXT.has(ext) ? ext : null
}

export function canPreviewAttachment(filename: string): boolean {
  const ext = attachmentExt(filename)
  return !!ext && PREVIEW_EXT.has(ext)
}

export function formatAttachmentSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

export function listAttachments(ownerType: AttachmentOwnerType, ownerId: string) {
  return invoke<Attachment[]>('list_attachments', { ownerType, ownerId })
}

export function addAttachment(ownerType: AttachmentOwnerType, ownerId: string, sourcePath: string) {
  return invoke<Attachment>('add_attachment', { ownerType, ownerId, sourcePath })
}

export function removeAttachment(id: string) {
  return invoke<void>('remove_attachment', { id })
}

export function getAttachmentPath(id: string) {
  return invoke<string>('get_attachment_path', { id })
}

export function openAttachment(id: string) {
  return invoke<void>('open_attachment', { id })
}

export function readAttachmentText(id: string) {
  return invoke<string>('read_attachment_text', { id })
}

export async function attachmentPreviewUrl(id: string): Promise<string> {
  const path = await getAttachmentPath(id)
  return convertFileSrc(path)
}

/** 创建 owner 后批量上传；返回成功数与错误信息 */
export async function flushPendingAttachments(
  ownerType: AttachmentOwnerType,
  ownerId: string,
  pending: PendingAttachment[],
): Promise<{ ok: number, errors: string[] }> {
  const errors: string[] = []
  let ok = 0
  for (const p of pending) {
    try {
      await addAttachment(ownerType, ownerId, p.path)
      ok++
    } catch (e) {
      errors.push(`${p.filename}: ${e instanceof Error ? e.message : String(e)}`)
    }
  }
  return { ok, errors }
}
```

注意：Rust 命令参数若用 `#[serde(rename_all = "camelCase")]` 结构体则前端传对象；若用独立参数 `owner_type`，Tauri 2 默认 snake→前端常用 camelCase 自动转换。与现有 `create_todo_task` 对齐：独立参数用 camelCase（`ownerType`）。

- [ ] **步骤 3：跑测试通过 + Commit**

```bash
pnpm test src/lib/attachments.test.ts
git add src/lib/attachments.ts src/lib/attachments.test.ts
git commit -m "feat(附件): 添加前端 attachments API 与单测"
```

---

### 任务 5：共用 UI 组件

**文件：**
- 创建：`src/components/attachments/AttachmentPicker.vue`
- 创建：`src/components/attachments/AttachmentList.vue`
- 创建：`src/components/attachments/AttachmentPreview.vue`
- 修改：`src/i18n/locales/zh.json`、`en.json`、`es.json`

- [ ] **步骤 1：i18n 键（三语同步）**

在各 locale 根级增加：

```json
"attachments": {
  "title": "附件",
  "add": "添加附件",
  "empty": "暂无附件",
  "remove": "删除",
  "missing": "文件缺失",
  "limitCount": "最多 5 个附件",
  "limitSize": "单个文件不能超过 10MB",
  "unsupported": "不支持的文件类型",
  "addFailed": "添加失败",
  "partialFailed": "部分附件添加失败",
  "openExternal": "使用系统打开",
  "preview": "预览",
  "close": "关闭",
  "dropHint": "拖拽文件到此处，或点击添加"
}
```

（en/es 译对应文案）

- [ ] **步骤 2：`AttachmentPicker.vue`**

Props：
- `disabled?: boolean`
- `remaining: number` — 还能再加几个
- `mode: 'pending' | 'owner'` — pending 只 emit；owner 需要 `ownerType`+`ownerId` 并直接 `addAttachment`

Emits：`added`（Attachment）、`pending`（PendingAttachment）、`error`（string）

行为：
- 按钮调用 `@tauri-apps/plugin-dialog` 的 `open({ multiple: true, filters: [...] })`
- 支持 `@dragover` / `@drop`（drop 在 Tauri 里若拿不到 path，可仅支持 dialog；若 `event` 无 path 则 toast 提示用按钮选）
- 每个文件：检查 ext、size、remaining；pending 模式 emit；owner 模式 invoke add

- [ ] **步骤 3：`AttachmentList.vue`**

Props：`items: Attachment[] | PendingAttachment[]`、`readonly?: boolean`

- 显示图标（按 ext）、filename、size、`missing` 标记
- 删除按钮 → emit `remove`
- 点击已落库项：`canPreviewAttachment` → emit `preview`；否则 `openAttachment(id)`

- [ ] **步骤 4：`AttachmentPreview.vue`**

Props：`open`、`attachment: Attachment | null`  
Emits：`close`

- 图片：`<img :src="url">`，`url = await attachmentPreviewUrl(id)`
- pdf：`<iframe :src="url">` 或 `<embed>`
- md：`readAttachmentText` + `marked` + DOMPurify（项目已有依赖）渲染
- 加载失败显示 missing/错误

- [ ] **步骤 5：Commit**

```bash
git add src/components/attachments src/i18n/locales
git commit -m "feat(附件): 添加共用附件 UI 组件"
```

---

### 任务 6：接入待办（捕获 + 详情）

**文件：**
- 修改：`src/components/todo/TodoCaptureDialog.vue`
- 修改：`src/pages/todo.vue`

- [ ] **步骤 1：捕获对话框**

在 `TodoCaptureDialog`：
- `const pendingFiles = ref<PendingAttachment[]>([])`
- 打开时清空；模板输入区下挂 `AttachmentList`（pending）+ `AttachmentPicker mode="pending"`
- `save()` 在 `store.create(...)` 成功后：

```ts
const task = await store.create({...})
if (pendingFiles.value.length && task?.id) {
  const { errors } = await flushPendingAttachments('todo', task.id, pendingFiles.value)
  if (errors.length) error.value = t('attachments.partialFailed')
}
pendingFiles.value = []
```

确认 `store.create` 返回带 `id` 的任务（若当前只 refresh，改为返回 `create_todo_task` 结果）。

- [ ] **步骤 2：详情面板**

在 `todo.vue` 选中任务的详情区（描述/标签附近）增加「附件」区块：
- `watch(selectedId)` → `listAttachments('todo', id)`
- `AttachmentList` + `AttachmentPicker mode="owner"`
- `AttachmentPreview` 控制预览

- [ ] **步骤 3：手工冒烟要点写进 commit message 即可；Commit**

```bash
git add src/components/todo/TodoCaptureDialog.vue src/pages/todo.vue src/stores/todo.ts
git commit -m "feat(待办): 捕获与详情支持附件"
```

---

### 任务 7：接入灵感 Muse（捕获 + 详情）

**文件：**
- 修改：`src/components/muse/MuseCaptureDialog.vue`
- 修改：`src/components/muse/MuseDetailPane.vue`
- 必要时修改：`src/stores/muse.ts`（确保 create 返回 note id）

- [ ] **步骤 1：捕获** — 同任务 6，`ownerType: 'muse'`

- [ ] **步骤 2：详情** — 在 `MuseDetailPane` 状态/标签区块附近挂附件列表；切换选中 note 时刷新

- [ ] **步骤 3：Commit**

```bash
git add src/components/muse/MuseCaptureDialog.vue src/components/muse/MuseDetailPane.vue src/stores/muse.ts
git commit -m "feat(灵感): 捕获与详情支持附件"
```

---

### 任务 8：端到端验证

- [ ] **步骤 1：自动化**

```bash
cd src-tauri && cargo test commands::attachments::tests
pnpm test src/lib/attachments.test.ts
pnpm typecheck
```

预期：全部通过

- [ ] **步骤 2：手工验收（对照规格清单）**

- [ ] 待办捕获可添加附件，保存后详情可见
- [ ] 灵感捕获同上
- [ ] 详情可继续添加至 5 个；第 6 个被拒
- [ ] &gt;10MB / 不支持类型有 toast
- [ ] 图片 / md / pdf 应用内预览；docx/zip 系统打开
- [ ] 删除待办后 `attachments/todo/{id}` 目录消失
- [ ] Muse 进回收站再恢复：附件策略符合任务 3 表
- [ ] 备份目录出现 `*_attachments`；恢复后附件可开
- [ ] Muse 正文插图（base64）行为无回归

- [ ] **步骤 3：若有修复则 commit；否则完成**

---

## 自检对照规格

| 规格章节 | 对应任务 |
|---|---|
| §3 表与磁盘布局 | 任务 1–2 |
| §3.3 白名单与限制 | 任务 2、4 |
| §3.4 与 base64 边界 | 任务 7 不改编辑器 |
| §3.5 备份 | 任务 3 |
| §4 后端命令 | 任务 2 |
| §5 前端组件与挂载 | 任务 5–7 |
| §5.3 捕获时序 | 任务 6–7 |
| §5.4 预览分流 | 任务 4–5 |
| §6 错误处理 | 任务 2、5 |
| §7 测试与验收 | 任务 2、4、8 |
