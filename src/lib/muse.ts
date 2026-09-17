import { invoke } from '@tauri-apps/api/core'

/** 灵感状态机（详细设计 4.3） */
export type NoteStatus = 'new' | 'check' | 'doing' | 'done'

/** 灵感来源（附录 10.2） */
export type NoteSource = 'quick' | 'manual' | 'clip' | 'book' | 'pod' | 'web' | 'api' | 'import'

/** 左栏导航项，决定列表的筛选口径 */
export type NavKind = 'inbox' | 'today' | 'all' | 'unsorted' | 'review' | 'archive' | 'trash' | 'tag' | 'project'

/** 中栏的浏览方式（FR-03） */
export type ViewMode = 'card' | 'board' | 'timeline'

export interface Note {
  id: string
  content: string
  status: NoteStatus
  source: NoteSource
  sourceUrl?: string | null
  projectId?: string | null
  projectName?: string | null
  tags: string[]
  pinned: boolean
  archived: boolean
  deletedAt?: number | null
  /** Unix 毫秒 */
  createdAt: number
  /** Unix 毫秒 */
  updatedAt: number
}

export interface MuseTag {
  id: string
  name: string
  color: string
  noteCount: number
}

export interface MuseProject {
  id: string
  name: string
  color?: string | null
  status: 'active' | 'paused' | 'done' | 'archived'
  noteCount: number
  doneCount: number
}

export interface NoteCounts {
  inbox: number
  today: number
  all: number
  unsorted: number
  archive: number
  trash: number
}

export interface BackupInfo {
  path: string
  fileName: string
  size: number
  createdAt: number
}

export interface UpdateProjectPayload {
  name?: string
  color?: string | null
  status?: MuseProject['status']
  description?: string | null
}

/** 标签调色板（与后端 TAG_PALETTE 一致） */
export const TAG_PALETTE = [
  '#5b5bd6',
  '#e0872b',
  '#22a06b',
  '#e5484d',
  '#3b82f6',
  '#8b5cf6',
  '#0d9488',
  '#d946ef',
  '#0891b2',
  '#ca8a04',
] as const

export const NOTE_SOURCES: NoteSource[] = [
  'quick',
  'manual',
  'clip',
  'book',
  'pod',
  'web',
  'api',
  'import',
]

export interface NoteFilter {
  kind: NavKind
  /** `tag` / `project` 传名称 */
  value?: string | null
  /** 状态 chip，`all` 表示不限 */
  status?: NoteStatus | 'all'
  keyword?: string
}

export interface CreateNotePayload {
  content: string
  tags?: string[]
  project?: string | null
  source?: NoteSource
  sourceUrl?: string | null
}

export const STATUS_ORDER: NoteStatus[] = ['new', 'check', 'doing', 'done']

/** 状态展示：色值取自 UI 设计规范「状态色」 */
export const STATUS_META: Record<NoteStatus, { color: string, soft: string }> = {
  new: { color: '#3b82f6', soft: 'rgba(59,130,246,.12)' },
  check: { color: '#e0872b', soft: 'rgba(224,135,43,.12)' },
  doing: { color: '#8b5cf6', soft: 'rgba(139,92,246,.12)' },
  done: { color: '#22a06b', soft: 'rgba(34,160,107,.12)' },
}

/** 来源展示：图标与色值取自附录 10.2 */
export const SOURCE_META: Record<NoteSource, { short: string, color: string }> = {
  quick: { short: '⚡', color: '#5b5bd6' },
  manual: { short: '记', color: '#8b5cf6' },
  clip: { short: '剪', color: '#22a06b' },
  book: { short: '书', color: '#e0872b' },
  pod: { short: '播', color: '#3b82f6' },
  web: { short: '网', color: '#0891b2' },
  api: { short: 'API', color: '#0d9488' },
  import: { short: '导', color: '#9299a3' },
}

export function listNotes(filter: NoteFilter) {
  return invoke<Note[]>('list_muse_notes', { filter })
}

export function getNote(id: string) {
  return invoke<Note>('get_muse_note', { id })
}

export function getNoteCounts() {
  return invoke<NoteCounts>('get_muse_note_counts')
}

export function listTags() {
  return invoke<MuseTag[]>('list_muse_tags')
}

export function listProjects() {
  return invoke<MuseProject[]>('list_muse_projects')
}

export function listRelatedNotes(id: string, limit = 3) {
  return invoke<Note[]>('list_muse_related_notes', { id, limit })
}

export function createNote(payload: CreateNotePayload) {
  return invoke<Note>('create_muse_note', { req: payload })
}

export function updateNoteContent(id: string, content: string) {
  return invoke<Note>('update_muse_note_content', { id, content })
}

export function updateNoteStatus(id: string, status: NoteStatus) {
  return invoke<Note>('update_muse_note_status', { id, status })
}

/** 传 `null` 表示移出项目；项目名不存在时后端自动创建 */
export function updateNoteProject(id: string, project: string | null) {
  return invoke<Note>('update_muse_note_project', { id, project })
}

export function addTagToNote(noteId: string, tagName: string) {
  return invoke<Note>('add_muse_tag_to_note', { noteId, tagName })
}

export function removeTagFromNote(noteId: string, tagName: string) {
  return invoke<Note>('remove_muse_tag_from_note', { noteId, tagName })
}

export function archiveNote(id: string) {
  return invoke<Note>('archive_muse_note', { id })
}

export function unarchiveNote(id: string) {
  return invoke<Note>('unarchive_muse_note', { id })
}

export function deleteNote(id: string) {
  return invoke<void>('delete_muse_note', { id })
}

export function updateNoteSource(id: string, source: NoteSource) {
  return invoke<Note>('update_muse_note_source', { id, source })
}

export function listTrash() {
  return invoke<Note[]>('list_muse_trash')
}

export function restoreNote(id: string) {
  return invoke<Note>('restore_muse_note', { id })
}

export function purgeNote(id: string) {
  return invoke<void>('purge_muse_note', { id })
}

export function emptyTrash() {
  return invoke<number>('empty_muse_trash')
}

export function renameTag(id: string, name: string) {
  return invoke<MuseTag>('rename_muse_tag', { id, name })
}

export function updateTagColor(id: string, color: string) {
  return invoke<MuseTag>('update_muse_tag_color', { id, color })
}

export function mergeTags(fromId: string, toId: string) {
  return invoke<void>('merge_muse_tags', { fromId, toId })
}

export function deleteTag(id: string) {
  return invoke<void>('delete_muse_tag', { id })
}

export function createProject(name: string, color?: string | null) {
  return invoke<MuseProject>('create_muse_project', { name, color })
}

export function updateProject(id: string, req: UpdateProjectPayload) {
  return invoke<MuseProject>('update_muse_project', { id, req })
}

export function deleteProject(id: string) {
  return invoke<void>('delete_muse_project', { id })
}

export function backupDb(path?: string | null) {
  return invoke<string>('backup_muse_db', { path: path ?? null })
}

export function listBackups() {
  return invoke<BackupInfo[]>('list_muse_backups')
}

export function restoreDb(path: string) {
  return invoke<void>('restore_muse_db', { path })
}

export function restartApp() {
  return invoke<void>('restart_app')
}
