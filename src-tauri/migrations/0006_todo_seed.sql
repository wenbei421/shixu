-- =========================================================
-- 待办：首次运行的示例数据（对齐 docs/todo/原型.html seedTasks）
-- 复用 Muse 种子项目/标签；时间戳相对安装时刻，透视筛选与后端 UTC 日界一致。
-- =========================================================

INSERT OR IGNORE INTO sys_settings(key, value, updated_at) VALUES
  ('todo.seeded', '1', CAST(strftime('%s','now') AS INTEGER) * 1000);

-- —— 补项目（灵感工具 App / 个人博客 已由 0004 灌入）——
INSERT OR IGNORE INTO sys_projects(id, name, description, color, status, created_at, updated_at) VALUES
  (2344000000000000003, '个人成长', '阅读、习惯与自我提升', '#0891b2', 'active',
   CAST(strftime('%s','now') AS INTEGER) * 1000, CAST(strftime('%s','now') AS INTEGER) * 1000),
  (2344000000000000004, '家庭采购', '日常采购清单', '#e0872b', 'active',
   CAST(strftime('%s','now') AS INTEGER) * 1000, CAST(strftime('%s','now') AS INTEGER) * 1000);

-- —— 补标签（设计/写作/产品/效率 已由 0004 灌入；配色取自原型 TAG_COLORS）——
INSERT OR IGNORE INTO sys_tags(id, name, color, created_at, updated_at) VALUES
  (2345000000000000010, '工作', '#5b5bd6', CAST(strftime('%s','now') AS INTEGER) * 1000, CAST(strftime('%s','now') AS INTEGER) * 1000),
  (2345000000000000011, '学习', '#0891b2', CAST(strftime('%s','now') AS INTEGER) * 1000, CAST(strftime('%s','now') AS INTEGER) * 1000),
  (2345000000000000012, '购物', '#e0872b', CAST(strftime('%s','now') AS INTEGER) * 1000, CAST(strftime('%s','now') AS INTEGER) * 1000),
  (2345000000000000013, '健康', '#0d9488', CAST(strftime('%s','now') AS INTEGER) * 1000, CAST(strftime('%s','now') AS INTEGER) * 1000),
  (2345000000000000014, '生活', '#e5484d', CAST(strftime('%s','now') AS INTEGER) * 1000, CAST(strftime('%s','now') AS INTEGER) * 1000);

-- —— 顶层待办 ——
-- due 辅助：与 list_todo_tasks 相同，使用 UTC start of day
--   today18 / today10 / yesterday12 / tomorrow9 / dayAfter14 / nextWeek10

INSERT OR IGNORE INTO todo_tasks(
  id, title, description, status, priority, due_at, project_id, parent_id,
  sort_order, completed_at, archived, source, created_at, updated_at
) VALUES
  -- 1 今天 · 进行中 · 高
  (2347000000000000001,
   '设计 PC 端待办软件的原型图',
   '参考竞品：Todoist、TickTick。注意利用 PC 端键盘流优势，做出差异化。',
   'doing', 'high',
   CAST(strftime('%s','now','start of day','+18 hours') AS INTEGER) * 1000,
   2344000000000000001, NULL, 1.0, NULL, 0, 'manual',
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 7200000,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 1800000),

  -- 2 今天 · 待办 · 高（可能已逾期到上午）
  (2347000000000000002,
   '给张总发邮件确认下周会议时间',
   '',
   'todo', 'high',
   CAST(strftime('%s','now','start of day','+10 hours') AS INTEGER) * 1000,
   NULL, NULL, 2.0, NULL, 0, 'quick',
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 10800000,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 10800000),

  -- 3 即将到来 · 明天
  (2347000000000000003,
   '阅读《原子习惯》第 3 章并做笔记',
   '重点关注「习惯回路」和「环境设计」两节。',
   'todo', 'medium',
   CAST(strftime('%s','now','start of day','+1 day','+9 hours') AS INTEGER) * 1000,
   2344000000000000003, NULL, 3.0, NULL, 0, 'manual',
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 18000000,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 18000000),

  -- 4 已完成 · 昨日
  (2347000000000000004,
   '购买牛奶和鸡蛋',
   '',
   'done', 'low',
   CAST(strftime('%s','now','start of day','-1 day','+12 hours') AS INTEGER) * 1000,
   2344000000000000004, NULL, 4.0,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 72000000,
   0, 'manual',
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 86400000,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 72000000),

  -- 5 即将到来 · 约 5 天后
  (2347000000000000005,
   '写一篇关于「本地优先软件」的博客',
   '围绕 Tauri + SQLite 的本地优先架构展开，重点讲数据可导出、离线可用。',
   'doing', 'medium',
   CAST(strftime('%s','now','start of day','+5 day','+10 hours') AS INTEGER) * 1000,
   2344000000000000002, NULL, 5.0, NULL, 0, 'manual',
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 97200000,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 86400000),

  -- 6 即将到来 · 后天
  (2347000000000000006,
   '整理本周的会议纪要并同步给团队',
   '',
   'todo', 'medium',
   CAST(strftime('%s','now','start of day','+2 day','+14 hours') AS INTEGER) * 1000,
   NULL, NULL, 6.0, NULL, 0, 'quick',
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 28800000,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 28800000),

  -- 7 收集箱（无日期、无项目）
  (2347000000000000007,
   '预约牙医做半年一次的检查',
   '',
   'todo', 'low', NULL, NULL, NULL, 7.0, NULL, 0, 'quick',
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 172800000,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 172800000),

  -- 8 已完成 · 无日期
  (2347000000000000008,
   '整理收件箱，把零散待办归入项目',
   '',
   'done', 'medium', NULL, NULL, NULL, 8.0,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 172800000,
   0, 'manual',
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 259200000,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 172800000),

  -- 9 全部 · 有项目无日期
  (2347000000000000009,
   '研究 dnd-kit 在 Tauri 中的拖拽性能',
   '对比原生 HTML5 拖拽和 dnd-kit 的差异，看是否满足看板场景。',
   'todo', 'low', NULL,
   2344000000000000001, NULL, 9.0, NULL, 0, 'manual',
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 259200000,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 259200000),

  -- 10 收集箱
  (2347000000000000010,
   '清理三个月前的旧笔记，合并同义标签',
   '',
   'todo', 'low', NULL, NULL, NULL, 10.0, NULL, 0, 'quick',
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 345600000,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 345600000),

  -- 11 即将到来
  (2347000000000000011,
   '和家人一起规划国庆假期行程',
   '',
   'todo', 'medium',
   CAST(strftime('%s','now','start of day','+5 day','+10 hours') AS INTEGER) * 1000,
   NULL, NULL, 11.0, NULL, 0, 'manual',
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 432000000,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 432000000),

  -- 12 已归档
  (2347000000000000012,
   '已归档：去年双十一购物清单',
   '',
   'done', 'low', NULL, NULL, NULL, 12.0,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 2505600000,
   1, 'import',
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 2592000000,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 2592000000);

-- —— 子待办 ——
INSERT OR IGNORE INTO todo_tasks(
  id, title, description, status, priority, due_at, project_id, parent_id,
  sort_order, completed_at, archived, source, created_at, updated_at
) VALUES
  (2347000000000000101, '确定布局框架', '', 'done', 'none', NULL, NULL, 2347000000000000001, 1.0,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 3600000, 0, 'manual',
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 7200000,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 3600000),
  (2347000000000000102, '编写 HTML/CSS 原型', '', 'done', 'none', NULL, NULL, 2347000000000000001, 2.0,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 2400000, 0, 'manual',
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 7200000,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 2400000),
  (2347000000000000103, '补充交互细节', '', 'todo', 'none', NULL, NULL, 2347000000000000001, 3.0,
   NULL, 0, 'manual',
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 7200000,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 7200000),

  (2347000000000000104, '读完第 3 章', '', 'todo', 'none', NULL, NULL, 2347000000000000003, 1.0,
   NULL, 0, 'manual',
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 18000000,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 18000000),
  (2347000000000000105, '整理 3 条可执行的方法', '', 'todo', 'none', NULL, NULL, 2347000000000000003, 2.0,
   NULL, 0, 'manual',
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 18000000,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 18000000),

  (2347000000000000106, '列出大纲', '', 'done', 'none', NULL, NULL, 2347000000000000005, 1.0,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 90000000, 0, 'manual',
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 97200000,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 90000000),
  (2347000000000000107, '写引言部分', '', 'todo', 'none', NULL, NULL, 2347000000000000005, 2.0,
   NULL, 0, 'manual',
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 97200000,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 97200000),
  (2347000000000000108, '补充代码示例', '', 'todo', 'none', NULL, NULL, 2347000000000000005, 3.0,
   NULL, 0, 'manual',
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 97200000,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 97200000);

-- —— 任务-标签 ——
-- 设计=…005  写作=…007  产品=…001  效率=…008
-- 工作=…010  学习=…011  购物=…012  健康=…013  生活=…014
INSERT OR IGNORE INTO todo_task_tags(task_id, tag_id, created_at) VALUES
  (2347000000000000001, 2345000000000000010, CAST(strftime('%s','now') AS INTEGER) * 1000), -- 工作
  (2347000000000000001, 2345000000000000005, CAST(strftime('%s','now') AS INTEGER) * 1000), -- 设计
  (2347000000000000002, 2345000000000000010, CAST(strftime('%s','now') AS INTEGER) * 1000), -- 工作
  (2347000000000000003, 2345000000000000011, CAST(strftime('%s','now') AS INTEGER) * 1000), -- 学习
  (2347000000000000004, 2345000000000000012, CAST(strftime('%s','now') AS INTEGER) * 1000), -- 购物
  (2347000000000000005, 2345000000000000007, CAST(strftime('%s','now') AS INTEGER) * 1000), -- 写作
  (2347000000000000005, 2345000000000000001, CAST(strftime('%s','now') AS INTEGER) * 1000), -- 产品
  (2347000000000000006, 2345000000000000010, CAST(strftime('%s','now') AS INTEGER) * 1000), -- 工作
  (2347000000000000007, 2345000000000000013, CAST(strftime('%s','now') AS INTEGER) * 1000), -- 健康
  (2347000000000000008, 2345000000000000008, CAST(strftime('%s','now') AS INTEGER) * 1000), -- 效率
  (2347000000000000009, 2345000000000000011, CAST(strftime('%s','now') AS INTEGER) * 1000), -- 学习
  (2347000000000000009, 2345000000000000005, CAST(strftime('%s','now') AS INTEGER) * 1000), -- 设计
  (2347000000000000010, 2345000000000000008, CAST(strftime('%s','now') AS INTEGER) * 1000), -- 效率
  (2347000000000000011, 2345000000000000014, CAST(strftime('%s','now') AS INTEGER) * 1000), -- 生活
  (2347000000000000012, 2345000000000000012, CAST(strftime('%s','now') AS INTEGER) * 1000); -- 购物
