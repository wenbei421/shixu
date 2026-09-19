下面一次性输出四份配套文档，可直接作为开发执行依据。

---

# 一、UI 设计规范

## 1.1 设计令牌（Design Tokens）

### 颜色

| Token | 值 | 用途 |
|---|---|---|
| `--color-bg` | `#f7f8fa` | 应用背景 |
| `--color-panel` | `#ffffff` | 面板、卡片 |
| `--color-side` | `#fbfbfc` | 侧边栏、详情栏 |
| `--color-border` | `#e9ebef` | 普通描边 |
| `--color-border-strong` | `#d8dbe1` | 悬停描边 |
| `--color-text` | `#1b1e23` | 主文字 |
| `--color-text-2` | `#5c636e` | 次要文字 |
| `--color-muted` | `#9299a3` | 弱化文字 |
| `--color-accent` | `#5b5bd6` | 主色 |
| `--color-accent-hover` | `#4f4fc9` | 主色悬停 |
| `--color-accent-soft` | `#eeedfb` | 主色浅底 |
| `--color-danger` | `#e5484d` | 危险操作 |
| `--color-warn` | `#e0872b` | 警告 |
| `--color-ok` | `#22a06b` | 成功 |

### 状态色

| 状态 | 色值 | 浅底 |
|---|---|---|
| new | `#3b82f6` | `rgba(59,130,246,.12)` |
| check | `#e0872b` | `rgba(224,135,43,.12)` |
| doing | `#8b5cf6` | `rgba(139,92,246,.12)` |
| done | `#22a06b` | `rgba(34,160,107,.12)` |

### 字号

| Token | 值 | 行高 | 用途 |
|---|---|---|---|
| `--fs-11` | 11px | 1.4 | 计数、时间、辅助 |
| `--fs-12` | 12px | 1.5 | 按钮、标签、次要 |
| `--fs-13` | 13px | 1.6 | 正文、导航 |
| `--fs-13h` | 13.5px | 1.62 | 卡片正文 |
| `--fs-15` | 15px | 1.68 | 捕捉输入 |
| `--fs-16` | 16px | 1.4 | 列表标题 |
| `--fs-18` | 18px | 1.3 | 页面标题 |

### 间距

| Token | 值 |
|---|---|
| `--sp-1` | 2px |
| `--sp-2` | 4px |
| `--sp-3` | 6px |
| `--sp-4` | 8px |
| `--sp-5` | 10px |
| `--sp-6` | 12px |
| `--sp-7` | 14px |
| `--sp-8` | 16px |
| `--sp-9` | 18px |
| `--sp-10` | 20px |
| `--sp-12` | 24px |

### 圆角

| Token | 值 | 用途 |
|---|---|---|
| `--r-xs` | 4px | 小标签、kbd |
| `--r-sm` | 6px | 按钮、chip |
| `--r-md` | 8px | 输入框、卡片 |
| `--r-lg` | 10px | 面板、卡片 |
| `--r-xl` | 12px | 窗口 |
| `--r-full` | 999px | 圆形、胶囊 |

### 阴影

| Token | 值 | 用途 |
|---|---|---|
| `--sh-1` | `0 1px 3px rgba(20,25,40,.12)` | 轻浮起 |
| `--sh-2` | `0 2px 12px rgba(20,25,40,.07)` | 卡片悬停 |
| `--sh-3` | `0 4px 14px rgba(91,91,214,.32)` | 主按钮 |
| `--sh-4` | `0 30px 80px rgba(20,25,40,.28)` | 窗口、浮层 |

### 动效

| Token | 值 | 用途 |
|---|---|---|
| `--dur-fast` | 120ms | 悬停、颜色 |
| `--dur-base` | 150ms | 卡片、按钮 |
| `--dur-slow` | 200ms | 浮层、Toast |
| `--ease-out` | `cubic-bezier(.2,.9,.3,1)` | 进入 |
| `--ease-pop` | `cubic-bezier(.2,.9,.3,1.2)` | 弹入 |

## 1.2 组件状态规范

### 按钮

| 状态 | 背景 | 文字 | 描边 | 阴影 |
|---|---|---|---|---|
| 默认（主） | `#5b5bd6` | `#fff` | 无 | `--sh-3` |
| 悬停 | `#4f4fc9` | `#fff` | 无 | `--sh-3` |
| 按下 | `#4545b8` | `#fff` | 无 | 无 |
| 禁用 | `#5b5bd6` 40% | `#fff` | 无 | 无 |
| 默认（次） | `#fff` | `#5c636e` | `#d8dbe1` | 无 |
| 悬停（次） | `#f4f5f7` | `#1b1e23` | `#d8dbe1` | 无 |
| 危险悬停 | `#fdecec` | `#e5484d` | `#f5b8b8` | 无 |

### 输入框

| 状态 | 背景 | 描边 | 阴影 |
|---|---|---|---|
| 默认 | `#f4f5f7` | 透明 | 无 |
| 悬停 | `#f4f5f7` | `#e9ebef` | 无 |
| 聚焦 | `#fff` | `#5b5bd6` | `0 0 0 3px #eeedfb` |
| 禁用 | `#f1f2f5` | 透明 | 无 |
| 错误 | `#fff` | `#e5484d` | `0 0 0 3px rgba(229,72,77,.12)` |

### 卡片

| 状态 | 描边 | 阴影 | 位移 |
|---|---|---|---|
| 默认 | `#e9ebef` | 无 | 0 |
| 悬停 | `#d8dbe1` | `--sh-2` | -1px |
| 选中 | `#5b5bd6` | `0 0 0 3px #eeedfb` | 0 |
| 拖拽中 | `#e9ebef` | 无 | 0（opacity 0.4） |

### 标签 Chip

| 状态 | 背景 | 文字 |
|---|---|---|
| 默认 | `#eeedfb` | `#5b5bd6` |
| 灰色 | `#f1f2f5` | `#5c636e` |
| 悬停 | 提亮 4% | 不变 |
| 可移除 | 末尾 ✕，opacity 0.55 → 1 |

## 1.3 布局栅格

| 区域 | 宽度 | 内边距 |
|---|---|---|
| 窗口 | `min(1440px, 96vw)` | — |
| 标题栏 | 高 38px | `0 14px` |
| 左栏 | 236px | `12px 10px` |
| 中栏 | 自适应 | 头部 `16px 18px 0`，列表 `12px 14px` |
| 右栏 | 344px | `16px` |

## 1.4 图标规范

| 项 | 规范 |
|---|---|
| 尺寸 | 15×15（`ic`）、14×14（来源图标） |
| 线宽 | 1.4（普通）、1.5（搜索）、1.8（加号） |
| 风格 | 线性，圆角端点 |
| 颜色 | 继承 `currentColor` |

## 1.5 响应式断点

| 断点 | 行为 |
|---|---|
| ≥ 1280px | 三栏完整 |
| 1024–1279px | 右栏收窄至 300px |
| 900–1023px | 右栏可折叠 |
| < 900px | 单栏 + 抽屉 |

---

# 二、Tauri Command API 清单

## 2.1 命名约定

| 项 | 约定 |
|---|---|
| 命令名 | `snake_case`，如 `create_note` |
| 参数 | 结构体，`camelCase` 前端，`snake_case` Rust |
| 返回 | `Result<T, ApiError>` |
| ID | 雪花 ID，前端用 `string` |

## 2.2 错误结构

```rust
#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: String,      // NOT_FOUND / VALIDATION / DB / CONFLICT
    pub message: String,
    pub detail: Option<String>,
}
```

## 2.3 命令清单

### Notes

| 命令 | 参数 | 返回 | 说明 |
|---|---|---|---|
| `create_note` | `content, tags[], project?, source` | `Note` | 新建灵感 |
| `get_note` | `id` | `Note` | 单条详情 |
| `update_note_content` | `id, content` | `Note` | 更新正文 |
| `update_note_status` | `id, status` | `Note` | 切换状态 |
| `update_note_project` | `id, projectId?` | `Note` | 设置项目 |
| `list_notes` | `filter, sort, page, pageSize` | `Page<Note>` | 列表查询 |
| `search_notes` | `q, limit` | `SearchHit[]` | 全文搜索 |
| `archive_note` | `id` | `Note` | 归档 |
| `unarchive_note` | `id` | `Note` | 取消归档 |
| `delete_note` | `id` | `void` | 软删除 |
| `restore_note` | `id` | `Note` | 恢复 |
| `purge_note` | `id` | `void` | 彻底删除 |
| `mark_note_done` | `id` | `Note` | 标记已输出 |

### Tags

| 命令 | 参数 | 返回 | 说明 |
|---|---|---|---|
| `list_tags` | `withStats` | `Tag[]` | 标签列表 |
| `create_tag` | `name, color?` | `Tag` | 新建 |
| `rename_tag` | `id, name` | `Tag` | 改名 |
| `update_tag_color` | `id, color` | `Tag` | 改色 |
| `merge_tags` | `fromId, toId` | `void` | 合并 |
| `delete_tag` | `id` | `void` | 删除 |
| `add_tag_to_note` | `noteId, tagName` | `Tag` | 绑定 |
| `remove_tag_from_note` | `noteId, tagId` | `void` | 解绑 |

### Projects

| 命令 | 参数 | 返回 | 说明 |
|---|---|---|---|
| `list_projects` | `status?` | `Project[]` | 项目列表 |
| `create_project` | `name, color?, description?` | `Project` | 新建 |
| `update_project` | `id, patch` | `Project` | 更新 |
| `delete_project` | `id` | `void` | 删除（有关联则报错） |
| `get_project_stats` | — | `ProjectStat[]` | 统计 |

### Reviews

| 命令 | 参数 | 返回 | 说明 |
|---|---|---|---|
| `create_review` | `noteId, result, nextReviewAt?` | `Review` | 记录回顾 |
| `list_due_reviews` | — | `Note[]` | 到期回顾 |
| `random_review` | `limit` | `Note[]` | 随机漫游 |
| `daily_review` | — | `Note[]` | 每日回顾 |

### Settings

| 命令 | 参数 | 返回 | 说明 |
|---|---|---|---|
| `get_setting` | `key` | `string?` | 读取 |
| `set_setting` | `key, value` | `void` | 写入 |
| `get_all_settings` | — | `Record<string,string>` | 全部 |

### Data

| 命令 | 参数 | 返回 | 说明 |
|---|---|---|---|
| `export_json` | `path` | `void` | 导出 JSON |
| `export_markdown` | `dir` | `void` | 导出 MD |
| `import_json` | `path` | `ImportReport` | 导入 |
| `backup_db` | `path?` | `string` | 备份 |
| `restore_db` | `path` | `void` | 恢复 |
| `vacuum_db` | — | `void` | 压缩 |

### AI（P1）

| 命令 | 参数 | 返回 | 说明 |
|---|---|---|---|
| `ai_suggest_tags` | `noteId` | `string[]` | 建议标签 |
| `ai_summarize` | `noteId` | `string` | 生成摘要 |
| `ai_related` | `noteId, limit` | `Note[]` | 关联推荐 |
| `ai_outline` | `noteId` | `string` | 生成大纲 |
| `ai_task_status` | `jobId` | `Job` | 任务状态 |

## 2.4 前端调用示例

```ts
import { invoke } from '@tauri-apps/api/core';

// 新建灵感
const note = await invoke<Note>('create_note', {
  content: '灵感内容',
  tags: ['产品', '交互'],
  project: '灵感工具 App',
  source: 'quick',
});

// 搜索
const hits = await invoke<SearchHit[]>('search_notes', {
  q: '灵感',
  limit: 50,
});

// 列表分页
const page = await invoke<Page<Note>>('list_notes', {
  filter: { type: 'inbox' },
  sort: { field: 'created_at', order: 'desc' },
  page: 1,
  pageSize: 20,
});
```

## 2.5 Rust 侧骨架

```rust
#[tauri::command]
async fn create_note(
    state: tauri::State<'_, AppState>,
    content: String,
    tags: Vec<String>,
    project: Option<String>,
    source: String,
) -> Result<Note, ApiError> {
    let content = content.trim().to_string();
    if content.is_empty() {
        return Err(ApiError::validation("内容不能为空"));
    }
    if content.chars().count() > 10_000 {
        return Err(ApiError::validation("内容超过 10000 字符"));
    }

    let id = state.snowflake.lock().unwrap().next_id();
    let now = now_ms();

    let conn = state.db.lock().unwrap();
    let tx = conn.transaction()?;

    tx.execute(
        "INSERT INTO muse_notes(id, content, status, source, created_at, updated_at)
         VALUES (?1, ?2, 'new', ?3, ?4, ?4)",
        params![id, content, source, now],
    )?;

    let project_id = match project {
        Some(name) if !name.is_empty() => Some(ensure_project(&tx, &name, now)?),
        _ => None,
    };
    if let Some(pid) = project_id {
        tx.execute("UPDATE muse_notes SET project_id = ?1 WHERE id = ?2", params![pid, id])?;
    }

    for tag in tags.iter().take(20) {
        let tag_id = ensure_tag(&tx, tag, now)?;
        tx.execute(
            "INSERT OR IGNORE INTO muse_note_tags(note_id, tag_id, created_at) VALUES (?1, ?2, ?3)",
            params![id, tag_id, now],
        )?;
    }

    tx.commit()?;
    Ok(load_note(&conn, id)?)
}
```

---

# 三、测试用例集

## 3.1 用例编号规则

`TC-FR{模块号}-{序号}`，如 `TC-FR01-01`。

优先级：P0 必测、P1 应测、P2 可测。

## 3.2 FR-01 快速捕捉

| 编号 | 用例 | 前置 | 步骤 | 预期 | 优先级 |
|---|---|---|---|---|---|
| TC-FR01-01 | 快捷键呼出 | 应用运行 | 按 `Ctrl+Shift+Space` | 浮层显示，输入框聚焦 | P0 |
| TC-FR01-02 | 快捷键关闭 | 浮层打开 | 再按快捷键 | 浮层关闭 | P0 |
| TC-FR01-03 | Esc 关闭 | 浮层打开 | 按 Esc | 浮层关闭，内容清空 | P0 |
| TC-FR01-04 | 空内容保存 | 浮层打开 | 不输入点保存 | 按钮置灰，不可点 | P0 |
| TC-FR01-05 | Enter 保存 | 输入内容 | 按 Enter | 保存成功，Toast | P0 |
| TC-FR01-06 | Shift+Enter 换行 | 输入内容 | 按 Shift+Enter | 换行不保存 | P0 |
| TC-FR01-07 | 解析 #标签 | 输入 `#产品` | 保存 | 标签写入，正文去掉 | P0 |
| TC-FR01-08 | 解析 @项目 | 输入 `@灵感工具 App` | 保存 | 项目关联 | P0 |
| TC-FR01-09 | 自动创建项目 | 输入新项目名 | 保存 | 项目自动创建 | P1 |
| TC-FR01-10 | 点击建议标签 | 浮层打开 | 点建议标签 | 加入已选 | P1 |
| TC-FR01-11 | 多标签 | 输入 3 个标签 | 保存 | 全部绑定 | P0 |
| TC-FR01-12 | 标签去重 | 输入重复标签 | 保存 | 只保留一个 | P1 |
| TC-FR01-13 | 超长内容 | 输入 10001 字符 | 保存 | 截断或提示 | P1 |
| TC-FR01-14 | 标签上限 | 输入 21 个标签 | 保存 | 超过 20 个被忽略 | P1 |
| TC-FR01-15 | 保存后进入收件箱 | 保存 | 查看收件箱 | 新灵感在首条 | P0 |
| TC-FR01-16 | 保存后自动选中 | 保存 | 查看详情栏 | 显示新灵感 | P1 |
| TC-FR01-17 | 连续捕捉 | 连续保存 10 条 | — | 无卡顿，全部入库 | P1 |
| TC-FR01-18 | 数据库异常 | 模拟写失败 | 保存 | 保留输入，提示重试 | P1 |

## 3.3 FR-02 收件箱

| 编号 | 用例 | 步骤 | 预期 | 优先级 |
|---|---|---|---|---|
| TC-FR02-01 | 条数正确 | 查看收件箱 | 与数据库一致 | P0 |
| TC-FR02-02 | 状态变更移出 | 改状态为 check | 移出收件箱 | P0 |
| TC-FR02-03 | 归档移出 | 归档 | 移出收件箱 | P0 |
| TC-FR02-04 | 删除移出 | 删除 | 移出收件箱 | P0 |
| TC-FR02-05 | 空状态 | 清空 | 显示空状态 | P1 |
| TC-FR02-06 | 排序 | 多条 | 按创建时间倒序 | P0 |

## 3.4 FR-03 列表与多视图

| 编号 | 用例 | 步骤 | 预期 | 优先级 |
|---|---|---|---|---|
| TC-FR03-01 | 切换卡片 | 点卡片按钮 | 卡片流显示 | P0 |
| TC-FR03-02 | 切换看板 | 点看板按钮 | 看板显示 | P0 |
| TC-FR03-03 | 切换时间线 | 点时间线按钮 | 时间线显示 | P0 |
| TC-FR03-04 | 视图切换无丢失 | 三视图切换 | 数据一致 | P0 |
| TC-FR03-05 | 看板拖拽 | 拖 new → doing | 状态更新 | P0 |
| TC-FR03-06 | 拖拽原列 | 拖到原列 | 不更新 | P1 |
| TC-FR03-07 | 拖拽失败回滚 | 模拟失败 | 回滚位置 | P1 |
| TC-FR03-08 | 状态筛选 | 点 chip | 列表过滤 | P0 |
| TC-FR03-09 | 时间线分组 | 查看 | 今天/昨天/N天前 | P1 |
| TC-FR03-10 | 高亮搜索词 | 搜索 | 命中片段高亮 | P1 |
| TC-FR03-11 | 500 条渲染 | 大数据 | ≤ 500ms | P1 |
| TC-FR03-12 | 空列表 | 无数据 | 显示空状态 | P1 |

## 3.5 FR-04 详情编辑

| 编号 | 用例 | 步骤 | 预期 | 优先级 |
|---|---|---|---|---|
| TC-FR04-01 | 点击卡片 | 点卡片 | 右栏显示详情 | P0 |
| TC-FR04-02 | 编辑保存 | 修改正文失焦 | 保存成功 | P0 |
| TC-FR04-03 | Esc 取消 | 编辑中按 Esc | 恢复原值 | P0 |
| TC-FR04-04 | 空内容恢复 | 清空失焦 | 恢复原值 | P1 |
| TC-FR04-05 | 状态切换 | 点状态按钮 | 更新并刷新 | P0 |
| TC-FR04-06 | 添加标签 | 点添加输入 | 绑定成功 | P0 |
| TC-FR04-07 | 移除标签 | 点 ✕ | 解绑成功 | P0 |
| TC-FR04-08 | 标签去重 | 添加已有标签 | 不重复 | P1 |
| TC-FR04-09 | 切换项目 | 下拉选择 | 关联更新 | P0 |
| TC-FR04-10 | 新建项目 | 选“+新建” | 项目创建并关联 | P1 |
| TC-FR04-11 | 移出项目 | 选“无项目” | 解除关联 | P1 |
| TC-FR04-12 | 标记已整理 | 点按钮 | 状态变 done | P0 |
| TC-FR04-13 | 归档 | 点归档 | archived_at 写入 | P0 |
| TC-FR04-14 | 删除确认 | 点删除 | 弹确认框 | P0 |
| TC-FR04-15 | 关联灵感跳转 | 点关联项 | 切换选中 | P1 |
| TC-FR04-16 | AI 按钮 | 点生成大纲 | Toast 提示 | P2 |

## 3.6 FR-05 标签管理

| 编号 | 用例 | 步骤 | 预期 | 优先级 |
|---|---|---|---|---|
| TC-FR05-01 | 标签列表 | 查看左栏 | 按数量倒序 | P0 |
| TC-FR05-02 | 计数正确 | — | 与数据库一致 | P0 |
| TC-FR05-03 | 点击筛选 | 点标签 | 过滤列表 | P0 |
| TC-FR05-04 | 改名 | 设置中改名 | 全局生效 | P1 |
| TC-FR05-05 | 改名后搜索 | 改名后搜索 | 仍可命中 | P1 |
| TC-FR05-06 | 合并标签 | A 并入 B | A 消失，计数并入 B | P1 |
| TC-FR05-07 | 删除标签 | 删除 | 解绑，不删灵感 | P0 |
| TC-FR05-08 | 标签着色 | 设置颜色 | 左栏圆点变色 | P2 |
| TC-FR05-09 | 超过 30 个 | 创建 31 个 | 折叠展开 | P2 |

## 3.7 FR-06 项目管理

| 编号 | 用例 | 步骤 | 预期 | 优先级 |
|---|---|---|---|---|
| TC-FR06-01 | 项目列表 | 查看左栏 | 显示项目 | P0 |
| TC-FR06-02 | 点击筛选 | 点项目 | 过滤列表 | P0 |
| TC-FR06-03 | 新建项目 | 捕捉 @新项目 | 自动创建 | P1 |
| TC-FR06-04 | 项目统计 | 查看 | 数量正确 | P1 |
| TC-FR06-05 | 删除空项目 | 删除 | 成功 | P1 |
| TC-FR06-06 | 删除有灵感项目 | 删除 | 阻止并提示 | P0 |
| TC-FR06-07 | 项目归档 | 归档 | 不显示在主列表 | P2 |

## 3.8 FR-07 搜索

| 编号 | 用例 | 步骤 | 预期 | 优先级 |
|---|---|---|---|---|
| TC-FR07-01 | 实时搜索 | 输入关键词 | 实时过滤 | P0 |
| TC-FR07-02 | 防抖 | 快速输入 | 150ms 后触发 | P1 |
| TC-FR07-03 | 搜正文 | 输入正文词 | 命中 | P0 |
| TC-FR07-04 | 搜标签 | 输入标签名 | 命中 | P0 |
| TC-FR07-05 | 搜项目 | 输入项目名 | 命中 | P0 |
| TC-FR07-06 | 高亮 | 搜索 | `<mark>` 高亮 | P1 |
| TC-FR07-07 | 清空 | 清空输入 | 恢复列表 | P0 |
| TC-FR07-08 | 无结果 | 输入不存在词 | 空状态 | P1 |
| TC-FR07-09 | Ctrl+P 聚焦 | 按快捷键 | 搜索框聚焦 | P0 |
| TC-FR07-10 | 中文搜索 | 中文词 | 命中 | P0 |
| TC-FR07-11 | 1w 条性能 | 大数据 | ≤ 200ms | P1 |
| TC-FR07-12 | 特殊字符 | 输入 `*` | 不报错 | P1 |

## 3.9 FR-08 回顾

| 编号 | 用例 | 步骤 | 预期 | 优先级 |
|---|---|---|---|---|
| TC-FR08-01 | 随机漫游 | 点回顾 | 随机列表 | P1 |
| TC-FR08-02 | 重新洗牌 | 点按钮 | 重新随机 | P1 |
| TC-FR08-03 | keep | 操作 | 写入 review | P1 |
| TC-FR08-04 | snooze | 操作 | 7 天后出现 | P1 |
| TC-FR08-05 | archive | 操作 | 归档 | P1 |
| TC-FR08-06 | promote | 操作 | 状态变 doing | P1 |
| TC-FR08-07 | 每日回顾 | 打开 | 5 条不重复 | P2 |

## 3.10 FR-09 归档与回收站

| 编号 | 用例 | 步骤 | 预期 | 优先级 |
|---|---|---|---|---|
| TC-FR09-01 | 归档 | 点归档 | 移出主列表 | P0 |
| TC-FR09-02 | 归档视图 | 点归档导航 | 显示归档 | P0 |
| TC-FR09-03 | 取消归档 | 点取消 | 回到主列表 | P0 |
| TC-FR09-04 | 删除 | 点删除 | 软删除 | P0 |
| TC-FR09-05 | 回收站 | 设置打开 | 显示已删除 | P1 |
| TC-FR09-06 | 恢复 | 点恢复 | 回到列表 | P1 |
| TC-FR09-07 | 清空 | 点清空 | 二次确认后物理删除 | P0 |
| TC-FR09-08 | 30 天自动清理 | 修改时间 | 自动删除 | P2 |

## 3.11 FR-10 AI 辅助

| 编号 | 用例 | 步骤 | 预期 | 优先级 |
|---|---|---|---|---|
| TC-FR10-01 | 建议标签 | 点按钮 | 返回 3 个 | P2 |
| TC-FR10-02 | 生成摘要 | 点按钮 | 返回摘要 | P2 |
| TC-FR10-03 | 关联推荐 | 点按钮 | 返回 3 条 | P2 |
| TC-FR10-04 | 未配置 Key | 点按钮 | 置灰提示 | P2 |
| TC-FR10-05 | 请求失败 | 模拟失败 | 重试 3 次 | P2 |
| TC-FR10-06 | 不阻塞 UI | 请求中 | 可继续操作 | P2 |

## 3.12 FR-11 设置

| 编号 | 用例 | 步骤 | 预期 | 优先级 |
|---|---|---|---|---|
| TC-FR11-01 | 修改设置 | 改主题 | 即时生效 | P1 |
| TC-FR11-02 | 重启保留 | 重启 | 设置保留 | P1 |
| TC-FR11-03 | 快捷键修改 | 改快捷键 | 新键生效 | P1 |
| TC-FR11-04 | 快捷键冲突 | 设为已占用 | 提示冲突 | P1 |

## 3.13 FR-12 导入导出

| 编号 | 用例 | 步骤 | 预期 | 优先级 |
|---|---|---|---|---|
| TC-FR12-01 | 导出 JSON | 导出 | 文件完整 | P1 |
| TC-FR12-02 | 导出 MD | 导出 | front matter 正确 | P1 |
| TC-FR12-03 | 导出 CSV | 导出 | 列正确 | P1 |
| TC-FR12-04 | 导入 JSON | 导入 | 数据恢复 | P1 |
| TC-FR12-05 | 导入去重 | 重复导入 | 不重复 | P1 |
| TC-FR12-06 | 格式错误 | 导入坏文件 | 跳过并报告 | P1 |
| TC-FR12-07 | 大文件 | 1w 条 | 不卡死 | P1 |

## 3.14 FR-13 存储与备份

| 编号 | 用例 | 步骤 | 预期 | 优先级 |
|---|---|---|---|---|
| TC-FR13-01 | 自动备份 | 启动 | 生成备份 | P0 |
| TC-FR13-02 | 保留 7 份 | 启动 8 次 | 只留 7 份 | P1 |
| TC-FR13-03 | 手动备份 | 点备份 | 生成文件 | P1 |
| TC-FR13-04 | 恢复 | 选备份恢复 | 数据一致 | P0 |
| TC-FR13-05 | 备份可独立打开 | 用 sqlite3 打开 | 正常 | P1 |
| TC-FR13-06 | 崩溃不丢数据 | 强杀进程 | 重启数据在 | P0 |
| TC-FR13-07 | 完整性检查 | `PRAGMA integrity_check` | ok | P0 |

## 3.15 FR-14 云同步（P2）

| 编号 | 用例 | 步骤 | 预期 | 优先级 |
|---|---|---|---|---|
| TC-FR14-01 | 双设备修改 | A、B 各改 | 合并无丢失 | P2 |
| TC-FR14-02 | 离线编辑 | 断网修改 | 联网后同步 | P2 |
| TC-FR14-03 | 冲突解决 | 同时改 | 按 updated_at | P2 |
| TC-FR14-04 | 暂停同步 | 点暂停 | 停止同步 | P2 |

## 3.16 性能与稳定性

| 编号 | 用例 | 指标 | 优先级 |
|---|---|---|---|
| TC-PERF-01 | 冷启动 | ≤ 1.5s | P0 |
| TC-PERF-02 | 捕捉浮层 | ≤ 300ms | P0 |
| TC-PERF-03 | 搜索 1w 条 | ≤ 200ms | P1 |
| TC-PERF-04 | 500 条渲染 | ≤ 500ms | P1 |
| TC-PERF-05 | 连续操作 1h | 无内存泄漏 | P1 |
| TC-PERF-06 | 10w 条容量 | 正常使用 | P1 |

## 3.17 键盘可访问性

| 编号 | 用例 | 步骤 | 预期 | 优先级 |
|---|---|---|---|---|
| TC-A11Y-01 | 全键盘操作 | 仅键盘 | 完成主要流程 | P1 |
| TC-A11Y-02 | Tab 顺序 | Tab | 合理顺序 | P1 |
| TC-A11Y-03 | 焦点可见 | 键盘操作 | 焦点环可见 | P1 |
| TC-A11Y-04 | 快捷键提示 | 查看 | 界面有提示 | P2 |

---

# 四、项目排期与人力估算

## 4.1 里程碑

| 里程碑 | 范围 | 周期 | 交付 |
|---|---|---|---|
| M0 环境搭建 | 仓库、Tauri、SQLite、CI | 1 周 | 可运行空壳 |
| M1 MVP | FR-01/02/03/04/05/07/09/13 | 4 周 | 可记录、整理、搜索 |
| M2 完整版 | FR-06/08/11/12 | 3 周 | 项目、回顾、设置、导入导出 |
| M3 智能化 | FR-10 | 2 周 | AI 标签、摘要、关联 |
| M4 同步 | FR-14 | 3 周 | 多端一致 |
| M5 打磨 | 性能、可访问、文档 | 2 周 | v1.0 |

**总计：约 15 周（3.5 个月）**

## 4.2 人力配置

| 角色 | 人数 | 职责 |
|---|---|---|
| 前端 | 1–2 | React + TS + 交互 |
| Rust | 1 | Tauri + SQLite |
| 设计 | 0.5 | UI + 规范 |
| 测试 | 0.5 | 用例 + 回归 |
| PM | 0.5 | 需求 + 协调 |

**最小配置：2 人（1 前端 + 1 Rust），约 4 个月。**
**推荐配置：3 人，约 3 个月。**

## 4.3 任务拆解（M1 MVP）

| 周 | 前端 | Rust | 设计 | 测试 |
|---|---|---|---|---|
| W1 | 项目搭建、设计令牌 | Tauri 初始化、SQLite 连接 | UI 规范 | 用例编写 |
| W2 | 三栏布局、左栏导航 | 建表、迁移、雪花 ID | 组件规范 | 数据库测试 |
| W3 | 卡片流、详情编辑 | Notes CRUD、Tags CRUD | 交互走查 | CRUD 测试 |
| W4 | 捕捉浮层、搜索 | FTS、搜索 API | 高保真 | 端到端 |
| W5 | 看板、时间线、归档 | 视图查询、归档 API | 细节 | 回归 |

## 4.4 工时估算（M1）

| 模块 | 前端 | Rust | 合计 |
|---|---|---|---|
| 环境搭建 | 8h | 12h | 20h |
| 布局与导航 | 16h | 4h | 20h |
| 快速捕捉 | 12h | 8h | 20h |
| 列表与多视图 | 20h | 8h | 28h |
| 详情编辑 | 16h | 8h | 24h |
| 标签管理 | 8h | 6h | 14h |
| 搜索 | 8h | 6h | 14h |
| 归档与回收站 | 6h | 6h | 12h |
| 备份 | 4h | 8h | 12h |
| 测试 | 8h | 8h | 16h |
| **合计** | **106h** | **74h** | **180h** |

按每日 6h 有效工时，约 **30 人天**，2 人并行约 **3 周**（含缓冲 4 周）。

## 4.5 风险登记

| 风险 | 影响 | 概率 | 应对 |
|---|---|---|---|
| Tauri 生态不熟 | 进度延后 | 中 | 提前做技术预研 |
| SQLite 并发 | 数据损坏 | 低 | WAL + 单连接 |
| 中文分词 | 搜索差 | 中 | 预留 jieba 接入 |
| AI 成本 | 超预算 | 中 | 本地模型兜底 |
| 雪花 ID 冲突 | 数据错乱 | 低 | 设备 ID 唯一 |
| 需求蔓延 | 延期 | 高 | 严格按 FR 排期 |

## 4.6 交付物清单

| 类别 | 交付物 |
|---|---|
| 代码 | 前端仓库、Rust 仓库 |
| 数据库 | `muse_schema.sql`、`muse_seed.sql` |
| 文档 | 功能详设、UI 规范、API 清单、测试用例 |
| 安装包 | Windows / macOS / Linux |
| 用户手册 | 快速上手、快捷键、FAQ |

## 4.7 发布检查清单

- [ ] 所有 P0 用例通过
- [ ] 性能达标
- [ ] 无 P0/P1 Bug
- [ ] 数据迁移脚本验证
- [ ] 备份恢复验证
- [ ] 安装包签名
- [ ] 用户手册完成
- [ ] 版本号、更新日志

---

需要我继续输出以下任一项吗？
- **Tauri 项目脚手架代码**（目录结构 + 配置）
- **SQLx / rusqlite Repository 层完整实现**
- **React 组件树与状态管理设计**
- **CI/CD 配置**（GitHub Actions 打包三端）
