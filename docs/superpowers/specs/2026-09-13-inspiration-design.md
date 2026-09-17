# 灵感记录（Inspiration）功能设计规格

**日期**：2026-09-13  
**状态**：已批准（实现中）  
**范围**：拾序 OS MVP — 灵感收集、分类、标签、检索与编辑  

---

## 1. 背景与目标

灵感目前分散在抖音、钉钉、微信等工具中。本功能在拾序内提供**本地、可分类、可打标签**的灵感库，支持文字为主、可含链接与图片的内容，便于事后查找。

**成功标准（MVP）**

- 能在应用内新建灵感并保存到本地 SQLite  
- 可自定义分类（如公众号、拾序、工作）并按分类筛选  
- 可打标签（输入即建），并有标签管理（重命名/删除）  
- 列表可按关键词搜索标题与正文纯文本  
- 正文使用自有编辑器 `@icreate/minimal-ai-editor`，图片以 base64 内嵌 HTML  

---

## 2. 已确认决策

| 项 | 选择 |
|---|---|
| 录入方式 | 打开应用 → 新建 → 手动粘贴/输入/选图 |
| 浏览布局 | 左侧分类筛选 + 右侧列表 |
| 内容模型 | 文字为主，可含链接与图片 |
| 编辑器 | `@icreate/minimal-ai-editor@0.0.17`（非官方 shadcn 富文本） |
| 只读详情 | `MinimalDocumentPreview`；编辑态用 `MinimalAiEditor` |
| 图片存储 | base64 内嵌 `body_html`，单张约 ≤2MB |
| 标签 | 录入时可现打现建 + 独立「管理标签」 |
| 主键 | **雪花 ID**（64-bit / 十进制约 18～19 位；与日后服务端同形，避免迁移转换） |
| SQLite 路径 | **暂留 appData**；开发 `shixu.dev.db` / 生产 `shixu.db`；后续可再迁文档目录 |
| 明确不做（MVP） | 全局快捷键、第三方自动抓取、标签合并、云同步、导出、AI 侧栏 |

---

## 3. 信息架构与交互

### 3.1 导航

- 侧栏工作区新增「灵感」入口（路由如 `/inspiration`）  
- 「设置」仍在侧栏底部；用户 footer 不变  

### 3.2 灵感页布局

```
┌─────────────┬──────────────────────────────────────┐
│ 分类        │ 搜索 | 标签筛选 | 管理标签 | 新建     │
│ · 全部      ├──────────────────────────────────────┤
│ · 未分类    │ 列表项：标题 / 摘要 / 分类 / 标签 / 时间 │
│ · 自定义…   │                                      │
│ [管理分类]  │ 点击 → Sheet/面板：详情编辑           │
└─────────────┴──────────────────────────────────────┘
```

- 点列表项默认打开**只读详情**（`MinimalDocumentPreview` 渲染 `body_html`）  
- 点「编辑」切换为 `MinimalAiEditor`；保存后回到只读或关闭面板  

### 3.3 新建 / 编辑

- 标题：可选；为空时用正文首行或「无标题」  
- 分类：必选；提供默认「未分类」  
- 标签：多选 + 输入新建  
- 正文：`MinimalAiEditor`  
  - 建议 `toolbar-preset="note"`（或精简后的 note/comment）  
  - `v-model:model-html` → `body_html`  
  - 用编辑器给出的纯文本（如 `autoSaveHandler` 的 `text`）→ `body_text`  
  - `persistDocument=false`，避免与 SQLite 双写  
  - 图片：`uploadImage` 将 `File` 转为 `data:image/...;base64,...` 返回；超限提示  

### 3.4 分类管理

- 左栏可新增、重命名、删除分类  
- 删除分类：将该分类下灵感归入「未分类」（`category_id` 置为未分类 id）  

### 3.5 标签管理

- 「管理标签」抽屉或子面板：列表、重命名、删除  
- 删除标签：仅解除关联，不删除灵感  
- 标签合并：**不做**（v1.1）  

### 3.6 列表与检索

- 列表接口只返回摘要字段（不含完整 `body_html`）  
- 筛选：`categoryId`、`tagIds`、`keyword`（匹配 `title` + `body_text`）  
- 排序：默认 `updated_at` 倒序  

---

## 4. 标识与数据模型

### 4.1 主键（雪花，迁移友好）

为后续「服务端写库 + 多用户」避免 id 类型转换：

- 分类 / 标签 / 灵感主键均为 **雪花 ID**：底层是 **64-bit 整数（`i64`）**；写成十进制时大约是 **18～19 位数字**（例如 `1234567890123456789`），不是 64 位十进制数字  
- SQLite 用 `INTEGER`（最多 8 字节，可存该 64-bit 值）  
- **Rust 侧生成**（`i64`）；创建命令返回前写入  
- **IPC/JSON 一律序列化为十进制字符串**（约 18～19 个字符），避免 JavaScript `Number` 精度丢失；前端类型为 `string`，sqlx 读写用 `i64`  
- Worker 号段约定（写入实现注释与常量）：  
  - **本地客户端预留**例如 `worker_id = 0..15`（MVP 固定用 `0`）  
  - **未来服务端**使用 `16..31`（或更高段），保证本地已生成 id 与服务端新 id **无需改写即可并存导入**  

默认「未分类」分类：启动迁移/种子时用**固定雪花常量**（写死一个合法 id），保证各环境引用稳定。

### 4.2 表结构（SQLite migration）

```sql
-- inspiration_categories
id INTEGER PRIMARY KEY NOT NULL   -- snowflake i64
name TEXT NOT NULL UNIQUE
color TEXT NULL
sort_order INTEGER NOT NULL DEFAULT 0
created_at / updated_at

-- inspiration_tags
id INTEGER PRIMARY KEY NOT NULL
name TEXT NOT NULL UNIQUE COLLATE NOCASE
created_at / updated_at

-- inspirations
id INTEGER PRIMARY KEY NOT NULL
title TEXT NOT NULL DEFAULT ''
body_html TEXT NOT NULL DEFAULT ''
body_text TEXT NOT NULL DEFAULT ''
category_id INTEGER NULL REFERENCES inspiration_categories(id) ON DELETE SET NULL
created_at / updated_at

-- inspiration_tag_map
inspiration_id INTEGER NOT NULL
tag_id INTEGER NOT NULL
PRIMARY KEY (inspiration_id, tag_id)
FOREIGN KEY ... ON DELETE CASCADE
```

种子数据：插入默认分类「未分类」（固定雪花 id）。

索引：`inspirations(category_id)`、`inspirations(updated_at)`；全文检索 MVP 用 `LIKE`，FTS 留后续。

> 说明：现有 `fragments` 表可继续自增；灵感域单独雪花，不强制改历史表。
---

## 5. 后端命令（Tauri）

沿用现有 `sqlx` + `Db` 模式（参考 zap / 现有 `get_db_status`）。

| 命令 | 说明 |
|---|---|
| `list_inspiration_categories` | 含每类计数（可选） |
| `create/update/delete_inspiration_category` | CRUD |
| `list_inspiration_tags` | 标签库 |
| `create/update/delete_inspiration_tag` | 现建与管理 |
| `list_inspirations` | 摘要列表 + 筛选 |
| `get_inspiration` | 含 `body_html` |
| `create/update/delete_inspiration` | 写入时同步 `body_text` 与标签关联 |

错误：通过现有 `AppError` 扩展业务变体（如 NotFound）。

数据库路径：**保持** `app.path().app_data_dir()`（决策 B）；文件名按构建环境隔离——`tauri dev` → `shixu.dev.db`，`tauri build` → `shixu.db`。

---

## 6. 前端结构（建议）

| 单元 | 职责 |
|---|---|
| `src/pages/inspiration.vue` | 页面壳：左分类 + 右列表工具条 |
| `src/components/inspiration/*` | 列表项、编辑 Sheet、分类栏、标签管理、编辑器封装 |
| `src/lib/inspiration.ts` | `invoke` 封装与类型 |
| `src/i18n/locales/*` | `nav.inspiration` 等文案 |
| `AppSidebar.vue` | 增加导航项 |

编辑器封装：

- 编辑：`MinimalAiEditor` + style.css + base64 `uploadImage` + `toolbar-preset`  
- 只读：`MinimalDocumentPreview`（`html` = `body_html`）  
- 对外暴露 `html` + `text`；id 在 TS 中为 `string`  

UI：优先 shadcn-vue（Button、Sheet、Input、Badge、Separator、Dialog 等）；不引入第二套富文本。

---

## 7. 错误处理与约束

- 单图超过约 2MB：拒绝插入并提示  
- 保存时至少有非空 `body_text` 或 HTML 中含图，否则提示不可保存空灵感  
- 删除分类/标签前可简单确认 Dialog  

---

## 8. 测试要点

- Migration 可重复执行；默认「未分类」存在  
- 创建灵感带分类与新标签；列表摘要不含巨型 base64  
- 按分类、标签、关键词筛选正确  
- 删标签只解绑；删分类后灵感归入未分类  
- 编辑器封装：`uploadImage` 返回 data URL  

---

## 9. 架构关系

```
Vue 灵感页 ──invoke──► Tauri commands ──► sqlx SqlitePool (appData/shixu[.dev].db)
                │
                └── MinimalAiEditor (html + text；图片 base64 在 html 内)
```

与现有 `fragments` 表并存，互不影响。

---

## 10. 规格自检记录

- 无 TBD/TODO 占位  
- 编辑器、预览、图片、库路径、布局决策一致  
- 主键为雪花（64-bit ≈ 十进制 18～19 位）+ JSON 字符串，与日后服务端同形，避免迁移转换  
- Worker 号段本地/服务端已约定  
- 范围聚焦单一子系统（灵感），可一份实现计划覆盖  
- 「删除分类」行为已明确为归入未分类  
- 库路径决策 B（appData）已写入  
