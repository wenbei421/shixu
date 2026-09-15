# 灵感记录 实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 在拾序 OS 落地本地灵感库：分类筛选 + 列表、标签、MinimalAiEditor 编辑 / MinimalDocumentPreview 预览，SQLite 雪花主键。

**架构：** Tauri + sqlx 本地库（appData）；Rust 生成雪花 id（JSON 字符串）；Vue 页左分类右列表；编辑器用 `@icreate/minimal-ai-editor`，图片 base64 内嵌 HTML。

**技术栈：** Vue 3、shadcn-vue、sqlx/sqlite、@icreate/minimal-ai-editor、Tauri 2

**规格：** `docs/superpowers/specs/2026-09-13-inspiration-design.md`

---

## 文件结构

| 文件 | 职责 |
|---|---|
| `src-tauri/src/id.rs` | 雪花 id 生成 + 固定「未分类」常量 + `i64`↔字符串 |
| `src-tauri/migrations/0002_inspiration.sql` | 灵感相关表 + 种子 |
| `src-tauri/src/error.rs` | 扩展 NotFound / Invalid 等 |
| `src-tauri/src/commands/inspiration.rs` | 分类/标签/灵感 CRUD |
| `src-tauri/src/commands/mod.rs` | 导出并注册命令 |
| `src-tauri/src/lib.rs` | `invoke_handler` 注册 |
| `src/lib/inspiration.ts` | 前端类型与 invoke |
| `src/components/inspiration/InspirationEditor.vue` | 编辑器封装 |
| `src/components/inspiration/InspirationPreview.vue` | 预览封装 |
| `src/components/inspiration/InspirationSheet.vue` | 新建/详情/编辑 Sheet |
| `src/components/inspiration/CategorySidebar.vue` | 左栏分类 |
| `src/components/inspiration/TagManager.vue` | 标签管理 |
| `src/pages/inspiration.vue` | 页面壳 |
| `src/router/index.ts` | `/inspiration` |
| `src/components/AppSidebar.vue` | 导航项 |
| `src/i18n/locales/{zh,en,es}.json` | 文案 |
| `src/main.ts` 或 `App.vue` | 引入 editor CSS |

---

### 任务 1：雪花 ID 模块

**文件：**
- 创建：`src-tauri/src/id.rs`
- 修改：`src-tauri/src/lib.rs`（`mod id`）
- 修改：`src-tauri/Cargo.toml`（若需 `parking_lot` / 无额外依赖可自研）

- [ ] **步骤 1：** 实现 Twitter 风格雪花（41bit 时间 + 5bit worker + 5bit datacenter 可简化为 10bit worker + 12bit seq），`LOCAL_WORKER_ID = 0`，`UNCLASSIFIED_CATEGORY_ID` 写死常量。
- [ ] **步骤 2：** 提供 `next_id() -> i64`、`id_to_string(i64) -> String`、`parse_id(&str) -> Result<i64, AppError>`。
- [ ] **步骤 3：** `cargo test -p shixu-os --lib id`（或单元测 `next_id` 单调）。

### 任务 2：Migration

**文件：**
- 创建：`src-tauri/migrations/0002_inspiration.sql`

- [ ] **步骤 1：** 按规格建四表；种子 `INSERT`「未分类」使用 `UNCLASSIFIED_CATEGORY_ID`。
- [ ] **步骤 2：** 启动 app / `cargo check`，确认 migrate 不报错。

### 任务 3：Inspiration commands

**文件：**
- 创建：`src-tauri/src/commands/inspiration.rs`
- 修改：`commands/mod.rs`、`lib.rs`、`error.rs`

- [ ] **步骤 1：** 实现 categories / tags / inspirations 的 list/get/create/update/delete；list 灵感不含 `body_html`（或单独字段省略）；id 进出字符串。
- [ ] **步骤 2：** 删分类：将该类灵感 `category_id` 改为未分类 id，再删分类（禁止删未分类）。
- [ ] **步骤 3：** `cargo check`。

### 任务 4：前端 API + 路由导航

**文件：**
- 创建：`src/lib/inspiration.ts`
- 修改：`router`、`AppSidebar`、i18n

- [ ] **步骤 1：** 类型与 invoke 封装（id: string）。
- [ ] **步骤 2：** 路由 `/inspiration` + 侧栏「灵感」+ 三语文案。

### 任务 5：编辑器封装 + 灵感页 UI

**文件：**
- `src/components/inspiration/*`
- `src/pages/inspiration.vue`
- 引入 `@icreate/minimal-ai-editor/style.css`

- [ ] **步骤 1：** Editor：`toolbar-preset="note"`、`persist-document=false`、base64 `uploadImage`（2MB）、`v-model:model-html` + text。
- [ ] **步骤 2：** Preview：`MinimalDocumentPreview`。
- [ ] **步骤 3：** 页面：左分类、右搜索/标签/新建/列表；Sheet 只读/编辑切换。
- [ ] **步骤 4：** `pnpm typecheck` + 手动 `pnpm tauri dev` 点通主路径。

### 任务 6：收尾

- [ ] 更新规格状态为「已批准并实现中/完成」
- [ ] 确认 list 不拖大 base64；空内容保存有提示

---

## 规格覆盖自检

- 布局 B、录入 A、编辑器包、预览、base64、雪花、appData、标签现建+管理、分类 CRUD、筛选搜索：均有对应任务  
- 不做：快捷键/抓取/合并/云同步/AI：未列入  

---

**执行：** 用户已指示「开工」→ 本会话 **内联执行** 下列任务。
