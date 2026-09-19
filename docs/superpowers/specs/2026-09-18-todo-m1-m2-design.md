# 待办模块 M1+M2 设计

> 日期：2026-09-18  
> 范围：骨架 + 数据层（对齐 Muse 工程约定）  
> 前置文档：`docs/todo/详细设计.md`、`docs/todo/数据库设计.md`

## 1. 目标

交付可用的待办数据层与三栏骨架：共用项目/标签/设置，任务 CRUD，卡片列表可增删改完成。看板/时间线/全局快捷键/FTS 不在本阶段。

## 2. 决策摘要

| 决策 | 选择 |
|---|---|
| 参考模块 | Muse（非已弃用的 inspiration） |
| 共用表 | `sys_projects`、`sys_tags`、`sys_settings`（由 `muse_*` 迁移改名） |
| 模块私有表 | Muse 保留 `muse_notes` 等；Todo 新增 `todo_tasks`、`todo_task_tags` |
| 主键 | 雪花 i64（与 Muse 一致） |
| 时间戳 | Unix 毫秒 INTEGER |
| 延后 | HLC、op_log、FTS、提醒、重复任务 |

## 3. Schema

### 3.1 共用（迁移自 Muse）

- `sys_projects`：保留 Muse 字段；新增 `parent_id`、`sort_order`（REAL，待办排序/一层嵌套）
- `sys_tags`：保留 Muse 字段；新增 `usage_count`、`deleted_at`
- `sys_settings`：键名空间 `app.*` / `muse.*` / `todo.*`

### 3.2 Todo 私有

- `todo_tasks`：title、description、status、priority、due_at、project_id、parent_id、sort_order、completed_at、archived、source、软删、时间戳
- `todo_task_tags`：(task_id, tag_id) 多对多

FK：`project_id` → `sys_projects`，`tag_id` → `sys_tags`。

## 4. 后端

- Muse SQL 全部改为 `sys_*`；触发器/视图重建
- 新增 `commands/todo.rs` + 注册；命令前缀 `todo_*` / `list_todo_*`
- 共用 CRUD 可暂仍挂在 muse commands（改表名），后续再抽 `sys` 模块

## 5. 前端

- 路由 `/todo`，侧栏入口
- `lib/todo.ts`、`stores/todo.ts`、`pages/todo.vue` + 基础组件
- M1：三栏空壳；M2：接真实数据（透视：收集箱/今天/即将/全部/已完成）

## 6. 验收

- 新库 / 已有 Muse 库均可迁移到 `sys_*`，Muse 功能不回归
- 可创建任务、挂项目/标签、完成/撤销、按透视筛选
- 开发库仍为 `shixu.dev.db`，与生产隔离
