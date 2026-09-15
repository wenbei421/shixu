-- =========================
-- 应用元信息（迁移/诊断用）
-- =========================
CREATE TABLE IF NOT EXISTS app_meta (
  key TEXT PRIMARY KEY NOT NULL,
  value TEXT NOT NULL,
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT OR IGNORE INTO app_meta (key, value) VALUES ('schema_version', '1');

-- =========================
-- 工作片段：窗口活动聚合记录（Phase 1）
-- =========================
CREATE TABLE IF NOT EXISTS fragments (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  started_at TEXT NOT NULL,                          -- 片段开始时间（UTC）
  ended_at TEXT,                                     -- 片段结束时间（UTC，空表示进行中）
  duration_seconds INTEGER NOT NULL DEFAULT 0,        -- 时长（秒）
  app_name TEXT NOT NULL,                             -- 应用名
  window_title TEXT NOT NULL DEFAULT '',              -- 窗口标题
  note TEXT,                                         -- 用户备注（可选）
  category TEXT,                                     -- 活动分类（可选，后续 AI/规则写入）
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_fragments_started_at ON fragments(started_at);
CREATE INDEX IF NOT EXISTS idx_fragments_app_name ON fragments(app_name);
CREATE INDEX IF NOT EXISTS idx_fragments_ended_at ON fragments(ended_at);
