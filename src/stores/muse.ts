import type {
  CreateNotePayload,
  MuseProject,
  MuseTag,
  NavKind,
  Note,
  NoteCounts,
  NoteSource,
  NoteStatus,
  UpdateProjectPayload,
  ViewMode,
} from '@/lib/muse'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import {
  addTagToNote,
  archiveNote,
  backupDb,
  createNote,
  createProject,
  deleteNote,
  deleteProject,
  deleteTag,
  emptyTrash,
  getNoteCounts,
  listBackups,
  listNotes,
  listProjects,
  listRelatedNotes,
  listTags,
  listTrash,
  mergeTags,
  purgeNote,
  removeTagFromNote,
  renameTag,
  restartApp,
  restoreDb,
  restoreNote,
  unarchiveNote,
  updateNoteContent,
  updateNoteProject,
  updateNoteSource,
  updateNoteStatus,
  updateProject,
  updateTagColor,
} from '@/lib/muse'
import { shuffleBySalt } from '@/lib/muse-format'

/** 搜索防抖窗口（FR-07.2） */
const SEARCH_DEBOUNCE = 150

export interface MuseFilterState {
  kind: NavKind
  value: string | null
}

export const useMuseStore = defineStore('muse', () => {
  const notes = ref<Note[]>([])
  const tags = ref<MuseTag[]>([])
  const projects = ref<MuseProject[]>([])
  const trash = ref<Note[]>([])
  const counts = ref<NoteCounts>({ inbox: 0, today: 0, all: 0, unsorted: 0, archive: 0, trash: 0 })
  const related = ref<Note[]>([])

  const filter = ref<MuseFilterState>({ kind: 'inbox', value: null })
  const search = ref('')
  const statusFilter = ref<NoteStatus | 'all'>('all')
  const view = ref<ViewMode>('card')
  const selectedId = ref<string | null>(null)
  /** 回顾视图的洗牌种子 */
  const reviewSalt = ref(0)

  const loading = ref(false)
  const error = ref('')

  /** 收件箱、归档、回收站没有状态推进的语义 */
  const statusChipsVisible = computed(
    () => filter.value.kind !== 'inbox'
      && filter.value.kind !== 'archive'
      && filter.value.kind !== 'trash',
  )

  const isTrashView = computed(() => filter.value.kind === 'trash')

  /** 回顾模式下打乱顺序，其余沿用后端的时间倒序 */
  const visibleNotes = computed(() =>
    filter.value.kind === 'review' ? shuffleBySalt(notes.value, reviewSalt.value) : notes.value,
  )

  const selectedNote = computed(
    () => visibleNotes.value.find(n => n.id === selectedId.value) ?? null,
  )

  function reportError(e: unknown) {
    error.value = e instanceof Error ? e.message : String(e)
  }

  async function refreshNotes() {
    loading.value = true
    error.value = ''
    try {
      if (filter.value.kind === 'trash') {
        const keyword = search.value.trim().toLowerCase()
        const items = await listTrash()
        notes.value = keyword
          ? items.filter(n => n.content.toLowerCase().includes(keyword))
          : items
        trash.value = items
      }
      else {
        notes.value = await listNotes({
          kind: filter.value.kind,
          value: filter.value.value,
          status: statusChipsVisible.value ? statusFilter.value : 'all',
          keyword: search.value.trim() || undefined,
        })
      }

      // 选中项被筛掉时回落到首条，保证详情栏始终有内容（原型 renderList）
      if (!notes.value.some(n => n.id === selectedId.value))
        await select(notes.value[0]?.id ?? null)
    }
    catch (e) {
      reportError(e)
    }
    finally {
      loading.value = false
    }
  }

  async function refreshSidebar() {
    try {
      const [nextCounts, nextTags, nextProjects] = await Promise.all([
        getNoteCounts(),
        listTags(),
        listProjects(),
      ])
      counts.value = nextCounts
      // 保留零计数项：左栏会过滤掉，但详情栏的项目下拉需要完整列表
      tags.value = nextTags
      projects.value = nextProjects
    }
    catch (e) {
      reportError(e)
    }
  }

  async function refreshTrash() {
    trash.value = await listTrash()
  }

  async function refreshAll() {
    await Promise.all([refreshNotes(), refreshSidebar()])
  }

  async function select(id: string | null) {
    selectedId.value = id
    related.value = []
    if (!id)
      return
    try {
      related.value = await listRelatedNotes(id)
    }
    catch (e) {
      reportError(e)
    }
  }

  async function setFilter(kind: NavKind, value: string | null = null) {
    filter.value = { kind, value }
    statusFilter.value = 'all'
    if (kind === 'review')
      reviewSalt.value = Date.now()
    await refreshNotes()
  }

  function reshuffle() {
    reviewSalt.value = Date.now()
  }

  let searchTimer: ReturnType<typeof setTimeout> | undefined
  function setSearch(value: string) {
    search.value = value
    clearTimeout(searchTimer)
    searchTimer = setTimeout(() => void refreshNotes(), SEARCH_DEBOUNCE)
  }

  async function setStatusFilter(value: NoteStatus | 'all') {
    statusFilter.value = value
    await refreshNotes()
  }

  function setView(value: ViewMode) {
    view.value = value
  }

  /** 写操作统一走这里：落库后重拉列表与左栏计数，失败冒泡给调用方提示 */
  async function applyMutation(mutate: () => Promise<Note>) {
    const updated = await mutate()
    await Promise.all([refreshNotes(), refreshSidebar()])
    return updated
  }

  async function capture(payload: CreateNotePayload) {
    const note = await createNote(payload)

    // 新灵感状态固定为 new，切回收件箱才能看见它（原型 commitCapture）
    if (filter.value.kind !== 'inbox')
      filter.value = { kind: 'inbox', value: null }
    statusFilter.value = 'all'
    search.value = ''
    await Promise.all([refreshNotes(), refreshSidebar()])
    await select(note.id)
    return note
  }

  function updateContent(id: string, content: string) {
    return applyMutation(() => updateNoteContent(id, content))
  }

  function setStatus(id: string, status: NoteStatus) {
    return applyMutation(() => updateNoteStatus(id, status))
  }

  function setProject(id: string, project: string | null) {
    return applyMutation(() => updateNoteProject(id, project))
  }

  function setSource(id: string, source: NoteSource) {
    return applyMutation(() => updateNoteSource(id, source))
  }

  function addTag(id: string, tagName: string) {
    return applyMutation(() => addTagToNote(id, tagName))
  }

  function removeTag(id: string, tagName: string) {
    return applyMutation(() => removeTagFromNote(id, tagName))
  }

  function setArchived(id: string, archived: boolean) {
    return applyMutation(() => (archived ? archiveNote(id) : unarchiveNote(id)))
  }

  async function remove(id: string) {
    await deleteNote(id)
    await select(null)
    await Promise.all([refreshNotes(), refreshSidebar()])
  }

  async function restore(id: string) {
    await restoreNote(id)
    await Promise.all([refreshTrash(), refreshNotes(), refreshSidebar()])
  }

  async function purge(id: string) {
    await purgeNote(id)
    await refreshTrash()
  }

  async function clearTrash() {
    const count = await emptyTrash()
    await refreshTrash()
    return count
  }

  async function renameTagById(id: string, name: string) {
    await renameTag(id, name)
    await Promise.all([refreshSidebar(), refreshNotes()])
  }

  async function setTagColor(id: string, color: string) {
    await updateTagColor(id, color)
    await refreshSidebar()
  }

  async function mergeTag(fromId: string, toId: string) {
    await mergeTags(fromId, toId)
    await Promise.all([refreshSidebar(), refreshNotes()])
  }

  async function removeTagById(id: string) {
    await deleteTag(id)
    await Promise.all([refreshSidebar(), refreshNotes()])
  }

  async function addProject(name: string, color?: string | null) {
    const project = await createProject(name, color)
    await refreshSidebar()
    return project
  }

  async function patchProject(id: string, req: UpdateProjectPayload) {
    await updateProject(id, req)
    await Promise.all([refreshSidebar(), refreshNotes()])
  }

  async function removeProject(id: string) {
    await deleteProject(id)
    await refreshSidebar()
  }

  async function createBackup(path?: string | null) {
    return backupDb(path)
  }

  async function fetchBackups() {
    return listBackups()
  }

  async function applyRestore(path: string) {
    await restoreDb(path)
  }

  async function reboot() {
    await restartApp()
  }

  return {
    notes,
    tags,
    projects,
    trash,
    counts,
    related,
    filter,
    search,
    statusFilter,
    view,
    selectedId,
    loading,
    error,
    statusChipsVisible,
    isTrashView,
    visibleNotes,
    selectedNote,
    refreshAll,
    refreshNotes,
    refreshSidebar,
    refreshTrash,
    select,
    setFilter,
    reshuffle,
    setSearch,
    setStatusFilter,
    setView,
    capture,
    updateContent,
    setStatus,
    setProject,
    setSource,
    addTag,
    removeTag,
    setArchived,
    remove,
    restore,
    purge,
    clearTrash,
    renameTagById,
    setTagColor,
    mergeTag,
    removeTagById,
    addProject,
    patchProject,
    removeProject,
    createBackup,
    fetchBackups,
    applyRestore,
    reboot,
  }
})
