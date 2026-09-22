# 待办与灵感附件设计

> 日期：2026-09-20  
> 范围：待办（Todo）与灵感（Muse）共用附件「资料袋」  
> 前置：`docs/todo/数据库设计.md`（附件表草案）、`docs/muse/raw/数据库设计.md`（`muse_attachments` 延后项）

## 1. 目标

为待办与灵感支持添加附件（图片、压缩包、md、docx、pdf 等）：捕获时与详情均可添加；常见类型应用内预览，其余用系统默认程序打开。

**成功标准**

- 捕获待办/灵感时可暂存附件，创建成功后落盘入库
- 详情面板可增删附件、点击预览或系统打开
- 单文件 ≤10MB，单条最多 5 个；类型白名单校验
- Muse 正文 base64 插图保持不变，与附件「资料袋」分离

## 2. 决策摘要

| 决策 | 选择 |
|---|---|
| 方案 | 轻量「文件清单」型：元数据入库，文件落盘 |
| 添加入口 | 捕获 + 详情（方案 B） |
| 打开方式 | 图片 / md / pdf 应用内预览；其余系统打开（方案 B） |
| 体积与数量 | 单文件 ≤10MB，单条最多 5 个（方案 C） |
| 存储 | 共用命令层 API；磁盘按 `todo/{id}/`、`muse/{id}/` 分目录（方案 C） |
| 表 | `sys_attachments`（`owner_type` + `owner_id`） |
| 明确不做 | 附件全文检索、云同步、跨 owner 去重、docx 应用内预览、无 owner 孤儿暂存表 |

## 3. 数据与存储

### 3.1 表 `sys_attachments`

| 字段 | 说明 |
|---|---|
| `id` | 雪花 i64 |
| `owner_type` | `'todo'` \| `'muse'` |
| `owner_id` | 对应 `todo_tasks.id` / `muse_notes.id` |
| `filename` | 原始文件名（展示用） |
| `stored_name` | 磁盘唯一名（防冲突） |
| `mime_type` | MIME |
| `size_bytes` | 字节数 |
| `hash_sha256` | 校验；同 owner 内可提示重复 |
| `created_at` | Unix 毫秒 |
| `deleted_at` | 软删除 |

索引：`(owner_type, owner_id)` where `deleted_at IS NULL`；可选 `hash_sha256`。

### 3.2 磁盘布局

相对 `app_data`：

```
attachments/
  todo/{owner_id}/{stored_name}
  muse/{owner_id}/{stored_name}
```

相对路径写入库（或由 `owner_type` + `owner_id` + `stored_name` 拼出），便于迁移。

### 3.3 类型白名单

- 图片：`png` / `jpg` / `jpeg` / `gif` / `webp`
- 文档：`pdf` / `md` / `markdown` / `docx` / `xlsx` / `xls`
- 压缩包：`zip` / `7z` / `rar`

扩展名与 MIME 双检；路径禁止 `..` 等 traversal。

### 3.4 与现有能力边界

- Muse 编辑器正文插图（base64 内嵌 `content`）不动
- 附件是独立资料袋，不写入正文
- 删除 owner：级联软删附件并删除对应磁盘文件

### 3.5 备份

本迭代将 `attachments/` 纳入现有应用/Muse 备份拷贝范围；恢复时与 DB 一并还原。若当前备份仅拷 DB，同步扩展备份逻辑。

## 4. 后端命令

共用命令，按 `owner_type` + `owner_id` 操作：

| 命令 | 行为 |
|---|---|
| `list_attachments` | 列出未软删元数据 |
| `add_attachment` | 校验类型/大小/数量 → 写盘 → 插库；失败回滚（删已写文件） |
| `remove_attachment` | 软删元数据并删除磁盘文件 |
| `get_attachment_path` | 返回绝对路径（校验归属） |
| `open_attachment` | 系统默认程序打开 |
| `read_attachment_text` / `read_attachment_bytes` | 预览用（md 文本；图片/PDF 字节或经 `convertFileSrc`） |

实现位置建议：`src-tauri/src/commands/attachments.rs`（或 `sys` 模块），在 todo/muse 删除路径中挂钩清理。

## 5. 前端交互

### 5.1 共用组件

- `AttachmentPicker`：选文件 / 拖拽
- `AttachmentList`：文件名、大小、类型图标、删除；点击按类型分流
- `AttachmentPreview`：Sheet/Dialog — 图片展示、md 渲染、PDF 内嵌

### 5.2 挂载点

- 待办捕获 `TodoCaptureDialog`：输入区下迷你附件条
- 待办详情 `todo.vue` 右侧：完整列表 + 添加
- 灵感捕获（Muse capture）：同上迷你条
- 灵感详情：完整列表 + 添加

### 5.3 捕获时序

1. 选文件 → 前端暂存（内存列表，未落库）
2. 创建 todo/muse 成功拿到 `owner_id` → 逐个 `add_attachment`
3. 部分失败：已写入保留并 toast；用户可在详情删除重试  
   （不做无 owner 孤儿暂存表）

### 5.4 预览分流

- 图片 / `md` / `pdf` → `AttachmentPreview`
- `docx` / `xlsx` / `xls` / 压缩包 / 其他 → `open_attachment`

## 6. 错误处理

| 情况 | 行为 |
|---|---|
| 超 10MB / 已满 5 个 | 拒绝，toast 说明限制 |
| 不支持扩展名 | 拒绝，提示允许类型摘要 |
| 写盘或插库失败 | 回滚，toast「添加失败」 |
| 库有盘无 | 列表标「缺失」；预览/打开提示损坏，可删元数据 |
| owner 不存在 | 命令报错，不写盘 |

同名文件：`stored_name` 保证唯一，展示用原 `filename`。

## 7. 测试与验收

**自动化（首版）**

- Rust：类型/大小/数量校验；add 失败回滚；list 不含软删；path traversal 拒绝
- 前端：捕获暂存 → 创建后批量 add；详情增删；预览分流

**手工**

- 拖拽多文件、超限、不支持类型
- 删任务/灵感后对应附件目录清空
- 备份含 `attachments/`，恢复后附件仍可打开

**验收清单**

- [ ] 待办捕获与详情均可添加附件
- [ ] 灵感捕获与详情均可添加附件
- [ ] 图片 / md / pdf 应用内预览；docx/压缩包系统打开
- [ ] 超限与类型拒绝有明确提示
- [ ] 删除 owner 后附件元数据与文件清理
- [ ] Muse 正文插图行为无回归
