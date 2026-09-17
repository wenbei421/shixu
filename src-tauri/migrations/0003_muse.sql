-- =========================================================
-- Muse 灵感库
-- 表前缀 muse_，视图前缀 v_muse_，主键为雪花 i64
-- 时间戳统一为 Unix 毫秒（INTEGER），前端以 number 使用
-- =========================================================

-- =========================================================
-- 1. 项目
-- =========================================================
CREATE TABLE IF NOT EXISTS muse_projects (
  id          INTEGER PRIMARY KEY NOT NULL,
  name        TEXT    NOT NULL UNIQUE,
  description TEXT,
  color       TEXT,
  status      TEXT    NOT NULL DEFAULT 'active'
              CHECK (status IN ('active','paused','done','archived')),
  created_at  INTEGER NOT NULL,
  updated_at  INTEGER NOT NULL,
  deleted_at  INTEGER
);

CREATE INDEX IF NOT EXISTS idx_muse_projects_status  ON muse_projects(status);
CREATE INDEX IF NOT EXISTS idx_muse_projects_deleted ON muse_projects(deleted_at);

-- =========================================================
-- 2. 灵感
-- =========================================================
CREATE TABLE IF NOT EXISTS muse_notes (
  id          INTEGER PRIMARY KEY NOT NULL,
  content     TEXT    NOT NULL,
  status      TEXT    NOT NULL DEFAULT 'new'
              CHECK (status IN ('new','check','doing','done')),
  priority    INTEGER NOT NULL DEFAULT 0 CHECK (priority BETWEEN 0 AND 3),
  source      TEXT    NOT NULL DEFAULT 'manual'
              CHECK (source IN ('quick','manual','clip','book','pod','web','api','import')),
  source_url  TEXT,
  project_id  INTEGER REFERENCES muse_projects(id) ON DELETE SET NULL,
  pinned      INTEGER NOT NULL DEFAULT 0 CHECK (pinned IN (0,1)),
  archived_at INTEGER,
  deleted_at  INTEGER,
  created_at  INTEGER NOT NULL,
  updated_at  INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_muse_notes_status      ON muse_notes(status);
CREATE INDEX IF NOT EXISTS idx_muse_notes_project_id  ON muse_notes(project_id);
CREATE INDEX IF NOT EXISTS idx_muse_notes_created_at  ON muse_notes(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_muse_notes_updated_at  ON muse_notes(updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_muse_notes_archived_at ON muse_notes(archived_at);
CREATE INDEX IF NOT EXISTS idx_muse_notes_deleted_at  ON muse_notes(deleted_at);

-- =========================================================
-- 3. 标签
-- =========================================================
CREATE TABLE IF NOT EXISTS muse_tags (
  id         INTEGER PRIMARY KEY NOT NULL,
  name       TEXT    NOT NULL UNIQUE,
  color      TEXT,
  parent_id  INTEGER REFERENCES muse_tags(id) ON DELETE SET NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_muse_tags_parent_id ON muse_tags(parent_id);

-- =========================================================
-- 4. 灵感-标签（多对多）
-- =========================================================
CREATE TABLE IF NOT EXISTS muse_note_tags (
  note_id    INTEGER NOT NULL REFERENCES muse_notes(id) ON DELETE CASCADE,
  tag_id     INTEGER NOT NULL REFERENCES muse_tags(id)  ON DELETE CASCADE,
  created_at INTEGER NOT NULL,
  PRIMARY KEY (note_id, tag_id)
);

CREATE INDEX IF NOT EXISTS idx_muse_note_tags_tag_id ON muse_note_tags(tag_id);

-- =========================================================
-- 5. 回顾记录
-- =========================================================
CREATE TABLE IF NOT EXISTS muse_reviews (
  id             INTEGER PRIMARY KEY NOT NULL,
  note_id        INTEGER NOT NULL REFERENCES muse_notes(id) ON DELETE CASCADE,
  reviewed_at    INTEGER NOT NULL,
  result         TEXT NOT NULL
                 CHECK (result IN ('keep','archive','promote','merge','delete','snooze')),
  next_review_at INTEGER,
  note           TEXT,
  created_at     INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_muse_reviews_note_id        ON muse_reviews(note_id);
CREATE INDEX IF NOT EXISTS idx_muse_reviews_next_review_at ON muse_reviews(next_review_at);

-- =========================================================
-- 6. 设置（Muse 专属键值）
-- =========================================================
CREATE TABLE IF NOT EXISTS muse_settings (
  key        TEXT PRIMARY KEY NOT NULL,
  value      TEXT,
  updated_at INTEGER NOT NULL
);

-- =========================================================
-- 7. FTS5 全文搜索
-- =========================================================
CREATE VIRTUAL TABLE IF NOT EXISTS muse_notes_fts USING fts5(
  note_id UNINDEXED,
  content,
  tags_text,
  project_name,
  tokenize = 'unicode61 remove_diacritics 2'
);

-- =========================================================
-- 8. 触发器：muse_notes -> FTS
-- =========================================================
CREATE TRIGGER IF NOT EXISTS trg_muse_notes_ai AFTER INSERT ON muse_notes BEGIN
  INSERT INTO muse_notes_fts(note_id, content, tags_text, project_name)
  VALUES (
    NEW.id,
    NEW.content,
    '',
    COALESCE((SELECT name FROM muse_projects WHERE id = NEW.project_id), '')
  );
END;

CREATE TRIGGER IF NOT EXISTS trg_muse_notes_au AFTER UPDATE ON muse_notes BEGIN
  DELETE FROM muse_notes_fts WHERE note_id = NEW.id;
  INSERT INTO muse_notes_fts(note_id, content, tags_text, project_name)
  SELECT
    NEW.id,
    NEW.content,
    COALESCE((
      SELECT group_concat(t.name, ' ')
      FROM muse_note_tags nt
      JOIN muse_tags t ON t.id = nt.tag_id
      WHERE nt.note_id = NEW.id
    ), ''),
    COALESCE((SELECT name FROM muse_projects WHERE id = NEW.project_id), '');
END;

CREATE TRIGGER IF NOT EXISTS trg_muse_notes_ad AFTER DELETE ON muse_notes BEGIN
  DELETE FROM muse_notes_fts WHERE note_id = OLD.id;
END;

-- =========================================================
-- 9. 触发器：muse_note_tags -> FTS
-- =========================================================
CREATE TRIGGER IF NOT EXISTS trg_muse_note_tags_ai AFTER INSERT ON muse_note_tags BEGIN
  DELETE FROM muse_notes_fts WHERE note_id = NEW.note_id;
  INSERT INTO muse_notes_fts(note_id, content, tags_text, project_name)
  SELECT
    n.id,
    n.content,
    COALESCE((
      SELECT group_concat(t.name, ' ')
      FROM muse_note_tags nt
      JOIN muse_tags t ON t.id = nt.tag_id
      WHERE nt.note_id = n.id
    ), ''),
    COALESCE((SELECT name FROM muse_projects WHERE id = n.project_id), '')
  FROM muse_notes n
  WHERE n.id = NEW.note_id;
END;

CREATE TRIGGER IF NOT EXISTS trg_muse_note_tags_ad AFTER DELETE ON muse_note_tags BEGIN
  DELETE FROM muse_notes_fts WHERE note_id = OLD.note_id;
  INSERT INTO muse_notes_fts(note_id, content, tags_text, project_name)
  SELECT
    n.id,
    n.content,
    COALESCE((
      SELECT group_concat(t.name, ' ')
      FROM muse_note_tags nt
      JOIN muse_tags t ON t.id = nt.tag_id
      WHERE nt.note_id = n.id
    ), ''),
    COALESCE((SELECT name FROM muse_projects WHERE id = n.project_id), '')
  FROM muse_notes n
  WHERE n.id = OLD.note_id;
END;

-- =========================================================
-- 10. 视图
-- =========================================================
DROP VIEW IF EXISTS v_muse_notes_full;
CREATE VIEW v_muse_notes_full AS
SELECT
  n.*,
  p.name AS project_name,
  COALESCE((
    SELECT group_concat(t.name, ',')
    FROM muse_note_tags nt
    JOIN muse_tags t ON t.id = nt.tag_id
    WHERE nt.note_id = n.id
  ), '') AS tags_text
FROM muse_notes n
LEFT JOIN muse_projects p ON p.id = n.project_id
WHERE n.deleted_at IS NULL;

DROP VIEW IF EXISTS v_muse_inbox;
CREATE VIEW v_muse_inbox AS
SELECT * FROM v_muse_notes_full
WHERE archived_at IS NULL AND status = 'new';

DROP VIEW IF EXISTS v_muse_today;
CREATE VIEW v_muse_today AS
SELECT * FROM v_muse_notes_full
WHERE archived_at IS NULL
  AND created_at >= (CAST(strftime('%s','now','start of day') AS INTEGER) * 1000);

DROP VIEW IF EXISTS v_muse_unsorted;
CREATE VIEW v_muse_unsorted AS
SELECT * FROM v_muse_notes_full
WHERE archived_at IS NULL
  AND tags_text = ''
  AND project_id IS NULL;

DROP VIEW IF EXISTS v_muse_archive;
CREATE VIEW v_muse_archive AS
SELECT * FROM v_muse_notes_full
WHERE archived_at IS NOT NULL;

DROP VIEW IF EXISTS v_muse_review_due;
CREATE VIEW v_muse_review_due AS
SELECT n.*, r.next_review_at AS due_at, r.result AS last_result
FROM v_muse_notes_full n
JOIN muse_reviews r ON r.note_id = n.id
WHERE n.archived_at IS NULL
  AND r.next_review_at IS NOT NULL
  AND r.next_review_at <= (CAST(strftime('%s','now') AS INTEGER) * 1000);

-- 标签统计：只统计未删除、未归档的灵感
DROP VIEW IF EXISTS v_muse_tag_stats;
CREATE VIEW v_muse_tag_stats AS
SELECT
  t.id,
  t.name,
  t.color,
  COUNT(n.id) AS note_count
FROM muse_tags t
LEFT JOIN muse_note_tags nt ON nt.tag_id = t.id
LEFT JOIN muse_notes n ON n.id = nt.note_id
  AND n.deleted_at IS NULL AND n.archived_at IS NULL
GROUP BY t.id;

DROP VIEW IF EXISTS v_muse_project_stats;
CREATE VIEW v_muse_project_stats AS
SELECT
  p.id,
  p.name,
  p.color,
  p.status,
  COUNT(n.id) AS note_count,
  SUM(CASE WHEN n.status = 'done' THEN 1 ELSE 0 END) AS done_count
FROM muse_projects p
LEFT JOIN muse_notes n ON n.project_id = p.id
  AND n.deleted_at IS NULL AND n.archived_at IS NULL
GROUP BY p.id;
