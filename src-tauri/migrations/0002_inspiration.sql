-- =========================
-- 灵感：分类 / 标签 / 正文
-- 主键均为雪花 i64（非 AUTOINCREMENT）
-- =========================

CREATE TABLE IF NOT EXISTS inspiration_categories (
  id INTEGER PRIMARY KEY NOT NULL,
  name TEXT NOT NULL UNIQUE,
  color TEXT,
  sort_order INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS inspiration_tags (
  id INTEGER PRIMARY KEY NOT NULL,
  name TEXT NOT NULL UNIQUE COLLATE NOCASE,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS inspirations (
  id INTEGER PRIMARY KEY NOT NULL,
  title TEXT NOT NULL DEFAULT '',
  body_html TEXT NOT NULL DEFAULT '',
  body_text TEXT NOT NULL DEFAULT '',
  category_id INTEGER REFERENCES inspiration_categories(id) ON DELETE SET NULL,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_inspirations_category_id ON inspirations(category_id);
CREATE INDEX IF NOT EXISTS idx_inspirations_updated_at ON inspirations(updated_at);

CREATE TABLE IF NOT EXISTS inspiration_tag_map (
  inspiration_id INTEGER NOT NULL,
  tag_id INTEGER NOT NULL,
  PRIMARY KEY (inspiration_id, tag_id),
  FOREIGN KEY (inspiration_id) REFERENCES inspirations(id) ON DELETE CASCADE,
  FOREIGN KEY (tag_id) REFERENCES inspiration_tags(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_inspiration_tag_map_tag ON inspiration_tag_map(tag_id);

-- 固定雪花 ID：未分类（与 src/id.rs::UNCLASSIFIED_CATEGORY_ID 一致）
INSERT OR IGNORE INTO inspiration_categories (id, name, sort_order)
VALUES (1930000000000000001, '未分类', 0);
