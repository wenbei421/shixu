-- 共用附件元数据（文件落盘见 app_data/attachments/）
CREATE TABLE IF NOT EXISTS sys_attachments (
  id INTEGER PRIMARY KEY NOT NULL,
  owner_type TEXT NOT NULL CHECK (owner_type IN ('todo', 'muse')),
  owner_id INTEGER NOT NULL,
  filename TEXT NOT NULL,
  stored_name TEXT NOT NULL,
  mime_type TEXT NOT NULL,
  size_bytes INTEGER NOT NULL,
  hash_sha256 TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  deleted_at INTEGER
);

CREATE INDEX IF NOT EXISTS idx_sys_attachments_owner
  ON sys_attachments(owner_type, owner_id)
  WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_sys_attachments_hash
  ON sys_attachments(hash_sha256);
