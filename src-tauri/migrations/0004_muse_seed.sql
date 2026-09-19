-- =========================================================
-- Muse 灵感库：首次运行的示例数据
-- ID 取自《数据库设计》种子清单；时间戳相对安装时刻生成，
-- 以便「今日 / N 天前」等相对时间展示与原型一致。
-- =========================================================

INSERT OR IGNORE INTO muse_settings(key, value, updated_at) VALUES
  ('muse.seeded',       '1',     CAST(strftime('%s','now') AS INTEGER) * 1000),
  ('muse.default_view', 'card',  CAST(strftime('%s','now') AS INTEGER) * 1000);

-- —— 项目 ——
INSERT OR IGNORE INTO muse_projects(id, name, description, color, status, created_at, updated_at) VALUES
  (2344000000000000001, '灵感工具 App', 'PC 端灵感捕捉与管理工具', '#5b5bd6', 'active',
   CAST(strftime('%s','now') AS INTEGER) * 1000, CAST(strftime('%s','now') AS INTEGER) * 1000),
  (2344000000000000002, '个人博客', '长文写作与知识输出', '#e0872b', 'active',
   CAST(strftime('%s','now') AS INTEGER) * 1000, CAST(strftime('%s','now') AS INTEGER) * 1000);

-- —— 标签（配色取自原型 TAG_COLORS）——
INSERT OR IGNORE INTO muse_tags(id, name, color, created_at, updated_at) VALUES
  (2345000000000000001, '产品', '#5b5bd6', CAST(strftime('%s','now') AS INTEGER) * 1000, CAST(strftime('%s','now') AS INTEGER) * 1000),
  (2345000000000000002, '交互', '#8b5cf6', CAST(strftime('%s','now') AS INTEGER) * 1000, CAST(strftime('%s','now') AS INTEGER) * 1000),
  (2345000000000000003, '金句', '#e5484d', CAST(strftime('%s','now') AS INTEGER) * 1000, CAST(strftime('%s','now') AS INTEGER) * 1000),
  (2345000000000000004, 'AI',   '#d946ef', CAST(strftime('%s','now') AS INTEGER) * 1000, CAST(strftime('%s','now') AS INTEGER) * 1000),
  (2345000000000000005, '设计', '#22a06b', CAST(strftime('%s','now') AS INTEGER) * 1000, CAST(strftime('%s','now') AS INTEGER) * 1000),
  (2345000000000000006, '方法', '#0d9488', CAST(strftime('%s','now') AS INTEGER) * 1000, CAST(strftime('%s','now') AS INTEGER) * 1000),
  (2345000000000000007, '写作', '#e0872b', CAST(strftime('%s','now') AS INTEGER) * 1000, CAST(strftime('%s','now') AS INTEGER) * 1000),
  (2345000000000000008, '效率', '#ca8a04', CAST(strftime('%s','now') AS INTEGER) * 1000, CAST(strftime('%s','now') AS INTEGER) * 1000),
  (2345000000000000009, '问题', '#3b82f6', CAST(strftime('%s','now') AS INTEGER) * 1000, CAST(strftime('%s','now') AS INTEGER) * 1000);

-- —— 灵感（10 条，时间偏移与原型种子一致）——
INSERT OR IGNORE INTO muse_notes(id, content, status, source, project_id, created_at, updated_at) VALUES
  (2346000000000000001,
   '如果灵感软件能像输入法一样常驻，按一个键就呼出，捕捉成本会降到最低——捕捉成本决定记录量。',
   'new', 'quick', 2344000000000000001,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 120000,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 120000),

  (2346000000000000002,
   '好工具不是功能多，而是让用户更快到达「啊哈」时刻。',
   'check', 'book', NULL,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 18000000,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 18000000),

  (2346000000000000003,
   '灵感碰撞：把三个月前的旧笔记和今天的剪藏随机配对，让 AI 生成三个新选题角度。',
   'doing', 'clip', 2344000000000000001,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 86400000,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 86400000),

  (2346000000000000004,
   'PC 端的优势是键盘流、多窗口、本地文件和批量操作——不要照搬移动端的设计思路。',
   'new', 'manual', NULL,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 97200000,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 97200000),

  (2346000000000000005,
   '每周回顾时只问三个问题：哪些还让我激动？哪些可以合并？哪些该直接删掉？',
   'check', 'pod', NULL,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 172800000,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 172800000),

  (2346000000000000006,
   '灵感卡片应该有「状态」而不是「文件夹」——灵感是流动的，不该被钉死在目录里。',
   'done', 'manual', 2344000000000000002,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 259200000,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 259200000),

  (2346000000000000007,
   '写作的难点从来不是「写什么」，而是「敢不敢把半成品拿出来」。',
   'doing', 'manual', 2344000000000000002,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 345600000,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 345600000),

  (2346000000000000008,
   '标签系统超过 30 个就会失控，应该定期合并同义标签。',
   'new', 'quick', NULL,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 432000000,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 432000000),

  (2346000000000000009,
   '一个好问题：如果这个功能只能保留一个按钮，会是哪一个？',
   'check', 'book', NULL,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 518400000,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 518400000),

  (2346000000000000010,
   '设计不是让它好看，而是让它不言自明。',
   'done', 'book', NULL,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 691200000,
   CAST(strftime('%s','now') AS INTEGER) * 1000 - 691200000);

-- —— 灵感-标签关联 ——
INSERT OR IGNORE INTO muse_note_tags(note_id, tag_id, created_at) VALUES
  (2346000000000000001, 2345000000000000001, CAST(strftime('%s','now') AS INTEGER) * 1000),
  (2346000000000000001, 2345000000000000002, CAST(strftime('%s','now') AS INTEGER) * 1000),
  (2346000000000000002, 2345000000000000001, CAST(strftime('%s','now') AS INTEGER) * 1000),
  (2346000000000000002, 2345000000000000003, CAST(strftime('%s','now') AS INTEGER) * 1000),
  (2346000000000000003, 2345000000000000001, CAST(strftime('%s','now') AS INTEGER) * 1000),
  (2346000000000000003, 2345000000000000004, CAST(strftime('%s','now') AS INTEGER) * 1000),
  (2346000000000000004, 2345000000000000005, CAST(strftime('%s','now') AS INTEGER) * 1000),
  (2346000000000000004, 2345000000000000002, CAST(strftime('%s','now') AS INTEGER) * 1000),
  (2346000000000000005, 2345000000000000006, CAST(strftime('%s','now') AS INTEGER) * 1000),
  (2346000000000000006, 2345000000000000001, CAST(strftime('%s','now') AS INTEGER) * 1000),
  (2346000000000000007, 2345000000000000007, CAST(strftime('%s','now') AS INTEGER) * 1000),
  (2346000000000000008, 2345000000000000006, CAST(strftime('%s','now') AS INTEGER) * 1000),
  (2346000000000000008, 2345000000000000008, CAST(strftime('%s','now') AS INTEGER) * 1000),
  (2346000000000000009, 2345000000000000001, CAST(strftime('%s','now') AS INTEGER) * 1000),
  (2346000000000000009, 2345000000000000009, CAST(strftime('%s','now') AS INTEGER) * 1000),
  (2346000000000000010, 2345000000000000005, CAST(strftime('%s','now') AS INTEGER) * 1000),
  (2346000000000000010, 2345000000000000003, CAST(strftime('%s','now') AS INTEGER) * 1000);
