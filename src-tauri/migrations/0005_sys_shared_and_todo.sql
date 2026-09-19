-- =========================================================
-- 共用表提升为 sys_* + 待办私有表
-- muse_projects / muse_tags / muse_settings → sys_*
-- =========================================================

-- 1) 改名（SQLite 会更新其它表中的 FK 引用）
ALTER TABLE muse_projects RENAME TO sys_projects;
ALTER TABLE muse_tags RENAME TO sys_tags;
ALTER TABLE muse_settings RENAME TO sys_settings;

-- 2) 索引随表改名保留；补待办所需字段
ALTER TABLE sys_projects ADD COLUMN parent_id INTEGER REFERENCES sys_projects(id) ON DELETE SET NULL;
ALTER TABLE sys_projects ADD COLUMN sort_order REAL NOT NULL DEFAULT 0;
ALTER TABLE sys_projects ADD COLUMN icon TEXT;

ALTER TABLE sys_tags ADD COLUMN usage_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE sys_tags ADD COLUMN deleted_at INTEGER;

CREATE INDEX IF NOT EXISTS idx_sys_projects_parent ON sys_projects(parent_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_sys_projects_status ON sys_projects(status);
CREATE INDEX IF NOT EXISTS idx_sys_projects_deleted ON sys_projects(deleted_at);
CREATE INDEX IF NOT EXISTS idx_sys_tags_parent_id ON sys_tags(parent_id);
CREATE INDEX IF NOT EXISTS idx_sys_tags_deleted ON sys_tags(deleted_at);

-- 3) 重建 Muse 触发器（SQL 正文需指向 sys_*）
DROP TRIGGER IF EXISTS trg_muse_notes_ai;
DROP TRIGGER IF EXISTS trg_muse_notes_au;
DROP TRIGGER IF EXISTS trg_muse_notes_ad;
DROP TRIGGER IF EXISTS trg_muse_note_tags_ai;
DROP TRIGGER IF EXISTS trg_muse_note_tags_ad;

CREATE TRIGGER IF NOT EXISTS trg_muse_notes_ai AFTER INSERT ON muse_notes BEGIN
  INSERT INTO muse_notes_fts(note_id, content, tags_text, project_name)
  VALUES (
    NEW.id,
    NEW.content,
    '',
    COALESCE((SELECT name FROM sys_projects WHERE id = NEW.project_id), '')
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
      JOIN sys_tags t ON t.id = nt.tag_id
      WHERE nt.note_id = NEW.id
    ), ''),
    COALESCE((SELECT name FROM sys_projects WHERE id = NEW.project_id), '');
END;

CREATE TRIGGER IF NOT EXISTS trg_muse_notes_ad AFTER DELETE ON muse_notes BEGIN
  DELETE FROM muse_notes_fts WHERE note_id = OLD.id;
END;

CREATE TRIGGER IF NOT EXISTS trg_muse_note_tags_ai AFTER INSERT ON muse_note_tags BEGIN
  DELETE FROM muse_notes_fts WHERE note_id = NEW.note_id;
  INSERT INTO muse_notes_fts(note_id, content, tags_text, project_name)
  SELECT
    n.id,
    n.content,
    COALESCE((
      SELECT group_concat(t.name, ' ')
      FROM muse_note_tags nt
      JOIN sys_tags t ON t.id = nt.tag_id
      WHERE nt.note_id = n.id
    ), ''),
    COALESCE((SELECT name FROM sys_projects WHERE id = n.project_id), '')
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
      JOIN sys_tags t ON t.id = nt.tag_id
      WHERE nt.note_id = n.id
    ), ''),
    COALESCE((SELECT name FROM sys_projects WHERE id = n.project_id), '')
  FROM muse_notes n
  WHERE n.id = OLD.note_id;
END;

-- 4) 重建 Muse 视图
DROP VIEW IF EXISTS v_muse_notes_full;
CREATE VIEW v_muse_notes_full AS
SELECT
  n.*,
  p.name AS project_name,
  COALESCE((
    SELECT group_concat(t.name, ',')
    FROM muse_note_tags nt
    JOIN sys_tags t ON t.id = nt.tag_id
    WHERE nt.note_id = n.id
  ), '') AS tags_text
FROM muse_notes n
LEFT JOIN sys_projects p ON p.id = n.project_id
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

DROP VIEW IF EXISTS v_muse_tag_stats;
CREATE VIEW v_muse_tag_stats AS
SELECT
  t.id,
  t.name,
  t.color,
  COUNT(n.id) AS note_count
FROM sys_tags t
LEFT JOIN muse_note_tags nt ON nt.tag_id = t.id
LEFT JOIN muse_notes n ON n.id = nt.note_id
  AND n.deleted_at IS NULL AND n.archived_at IS NULL
WHERE t.deleted_at IS NULL
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
FROM sys_projects p
LEFT JOIN muse_notes n ON n.project_id = p.id
  AND n.deleted_at IS NULL AND n.archived_at IS NULL
WHERE p.deleted_at IS NULL
GROUP BY p.id;

-- 5) 待办私有表
CREATE TABLE IF NOT EXISTS todo_tasks (
  id             INTEGER PRIMARY KEY NOT NULL,
  title          TEXT    NOT NULL CHECK (length(title) > 0),
  description    TEXT    NOT NULL DEFAULT '',
  status         TEXT    NOT NULL DEFAULT 'todo'
                 CHECK (status IN ('todo', 'doing', 'done', 'cancelled')),
  priority       TEXT    NOT NULL DEFAULT 'none'
                 CHECK (priority IN ('high', 'medium', 'low', 'none')),
  due_at         INTEGER,
  project_id     INTEGER REFERENCES sys_projects(id) ON DELETE SET NULL,
  parent_id      INTEGER REFERENCES todo_tasks(id) ON DELETE CASCADE,
  sort_order     REAL    NOT NULL DEFAULT 0,
  completed_at   INTEGER,
  archived       INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1)),
  source         TEXT    NOT NULL DEFAULT 'manual'
                 CHECK (source IN ('manual', 'quick', 'import', 'api')),
  created_at     INTEGER NOT NULL,
  updated_at     INTEGER NOT NULL,
  deleted_at     INTEGER
);

CREATE INDEX IF NOT EXISTS idx_todo_tasks_status_due
  ON todo_tasks(status, due_at)
  WHERE deleted_at IS NULL AND archived = 0;

CREATE INDEX IF NOT EXISTS idx_todo_tasks_project
  ON todo_tasks(project_id, sort_order)
  WHERE deleted_at IS NULL AND archived = 0;

CREATE INDEX IF NOT EXISTS idx_todo_tasks_parent
  ON todo_tasks(parent_id, sort_order)
  WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_todo_tasks_due
  ON todo_tasks(due_at)
  WHERE deleted_at IS NULL AND archived = 0 AND status != 'done';

CREATE INDEX IF NOT EXISTS idx_todo_tasks_deleted
  ON todo_tasks(deleted_at);

CREATE TABLE IF NOT EXISTS todo_task_tags (
  task_id    INTEGER NOT NULL REFERENCES todo_tasks(id) ON DELETE CASCADE,
  tag_id     INTEGER NOT NULL REFERENCES sys_tags(id) ON DELETE CASCADE,
  created_at INTEGER NOT NULL,
  PRIMARY KEY (task_id, tag_id)
);

CREATE INDEX IF NOT EXISTS idx_todo_task_tags_tag ON todo_task_tags(tag_id);

-- 6) 待办默认设置
INSERT OR IGNORE INTO sys_settings(key, value, updated_at) VALUES
  ('todo.default_view', 'card', CAST(strftime('%s','now') AS INTEGER) * 1000);
