import type { MuseProject } from '@/lib/muse'
import type {
  CreateTodoPayload,
  TodoCounts,
  TodoPerspective,
  TodoPriority,
  TodoTask,
  TodoViewMode,
  UpdateTodoPayload,
} from '@/lib/todo'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { listProjects } from '@/lib/muse'
import { retainItem } from '@/lib/retain-list'
import {
  completeTodoTask,
  createTodoTask,
  deleteTodoTask,
  exportTodoJson,
  getTodoCounts,
  getTodoProjectCounts,
  listTodoSubtasks,
  listTodoTasks,
  updateTodoTask,
} from '@/lib/todo'

const SEARCH_DEBOUNCE = 150

export const useTodoStore = defineStore('todo', () => {
  const tasks = ref<TodoTask[]>([])
  const subtasks = ref<TodoTask[]>([])
  const projects = ref<MuseProject[]>([])
  const perspective = ref<TodoPerspective>('inbox')
  const keyword = ref('')
  const priorityFilter = ref<TodoPriority | 'all'>('all')
  /** `all` / `none` / 项目 ID */
  const projectFilter = ref<string>('all')
  const viewMode = ref<TodoViewMode>('card')
  const selectedId = ref<string | null>(null)
  /** 日期或状态改完后已不符合当前筛选，仍留在列表里的那条 */
  const retainedId = ref<string | null>(null)
  const loading = ref(false)
  const error = ref('')

  /** 5 个视角的实时计数，依赖 priority 过滤（与左栏优先级组联动） */
  const counts = ref<TodoCounts>({ inbox: 0, today: 0, upcoming: 0, all: 0, done: 0 })

  /**
   * 每个 priority 过滤下「未完成总数」的快照，用于在优先级组右栏展示。
   * 例：切到 high 时本组「高」的右栏数字 = countsByPriority.high.all
   */
  const countsByPriority = ref<Partial<Record<TodoPriority | 'all', TodoCounts>>>({})
  /** 当前 perspective + priority 下，各项目的条数；`none` 表示无项目，`all` 为合计 */
  const countsByProject = ref<Record<string, number>>({ all: 0, none: 0 })

  let searchTimer: ReturnType<typeof setTimeout> | null = null
  let projectsLoaded = false

  const selected = computed(
    () => tasks.value.find(task => task.id === selectedId.value) ?? null,
  )

  const subtaskProgress = computed(() => {
    const total = subtasks.value.length
    const done = subtasks.value.filter(t => t.status === 'done').length
    return { total, done }
  })

  async function refreshProjects() {
    projects.value = await listProjects()
    projectsLoaded = true
  }

  /**
   * 拉取所有 5×5 组合的计数，存入 countsByPriority。
   * 数据变更后调用一次，保证切换 priority 时不需要再 round-trip。
   */
  async function refreshAllCounts() {
    const priorities: Array<TodoPriority | 'all'> = ['all', 'high', 'medium', 'low', 'none']
    try {
      const results = await Promise.all(
        priorities.map(async p => [p, await getTodoCounts({
          priority: p,
          projectId: projectFilter.value,
        })] as const),
      )
      const next: Partial<Record<TodoPriority | 'all', TodoCounts>> = {}
      for (const [p, c] of results)
        next[p] = c
      countsByPriority.value = next
      counts.value = next[priorityFilter.value] ?? counts.value

      const projectRows = await getTodoProjectCounts({
        perspective: perspective.value,
        priority: priorityFilter.value,
      })
      const byProject: Record<string, number> = { all: 0, none: 0 }
      for (const row of projectRows) {
        const key = row.projectId ?? 'none'
        byProject[key] = row.count
        byProject.all += row.count
      }
      countsByProject.value = byProject
    }
    catch (e) {
      console.error('[todo] refreshAllCounts failed', e)
    }
  }

  async function refreshSubtasks(parentId: string | null = selectedId.value) {
    if (!parentId) {
      subtasks.value = []
      return
    }
    subtasks.value = await listTodoSubtasks(parentId)
  }

  function releaseRetained() {
    const id = retainedId.value
    if (!id)
      return
    retainedId.value = null
    tasks.value = tasks.value.filter(task => task.id !== id)
  }

  async function refresh(options?: { keep?: TodoTask, keepIndex?: number }) {
    loading.value = true
    error.value = ''
    try {
      // 项目列表只在自己变动后重拉，别跟着每次搜索一起走
      const pending: Promise<unknown>[] = [
        listTodoTasks({
          perspective: perspective.value,
          keyword: keyword.value.trim() || undefined,
          priority: priorityFilter.value,
          projectId: projectFilter.value,
        }),
      ]
      if (!projectsLoaded)
        pending.push(refreshProjects())
      const [nextTasks] = await Promise.all(pending) as [TodoTask[], ...unknown[]]
      let next = nextTasks
      if (options?.keep) {
        const held = retainItem(next, options.keep, options.keepIndex ?? -1)
        next = held.items
        retainedId.value = held.retained ? options.keep.id : null
      }
      else {
        retainedId.value = null
      }
      tasks.value = next
      if (selectedId.value && !tasks.value.some(t => t.id === selectedId.value))
        selectedId.value = tasks.value[0]?.id ?? null
      await refreshSubtasks(selectedId.value)
      await refreshAllCounts()
    }
    catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
    finally {
      loading.value = false
    }
  }

  async function setPerspective(next: TodoPerspective) {
    perspective.value = next
    selectedId.value = null
    await refresh()
  }

  async function setPriorityFilter(next: TodoPriority | 'all') {
    priorityFilter.value = next
    await refresh()
  }

  async function setProjectFilter(next: string) {
    projectFilter.value = next
    await refresh()
  }

  function setViewMode(next: TodoViewMode) {
    if (next !== viewMode.value)
      releaseRetained()
    viewMode.value = next
  }

  function setKeyword(next: string) {
    keyword.value = next
    if (searchTimer)
      clearTimeout(searchTimer)
    searchTimer = setTimeout(() => {
      void refresh()
    }, SEARCH_DEBOUNCE)
  }

  async function select(id: string | null) {
    if (retainedId.value && retainedId.value !== id)
      releaseRetained()
    selectedId.value = id
    await refreshSubtasks(id)
  }

  async function create(payload: CreateTodoPayload) {
    const task = await createTodoTask(payload)
    if (payload.parentId) {
      await refreshSubtasks(payload.parentId)
      await refreshAllCounts()
      return task
    }
    await refresh()
    selectedId.value = task.id
    await refreshSubtasks(task.id)
    return task
  }

  async function update(id: string, payload: UpdateTodoPayload) {
    const task = await updateTodoTask(id, payload)
    const keep = selectedId.value
    const isSub = subtasks.value.some(t => t.id === id)
    if (isSub) {
      await refreshSubtasks(keep)
      await refreshAllCounts()
      return task
    }
    const sticky = 'status' in payload || 'dueAt' in payload || 'projectId' in payload || retainedId.value === id
    const index = tasks.value.findIndex(task => task.id === id)
    await refresh(sticky ? { keep: task, keepIndex: index } : undefined)
    if (keep && tasks.value.some(item => item.id === keep))
      selectedId.value = keep
    else if (keep === id && !tasks.value.some(item => item.id === id))
      selectedId.value = null
    return task
  }

  async function complete(id: string, done: boolean) {
    const task = await completeTodoTask(id, done)
    const isSub = subtasks.value.some(t => t.id === id) || !!task.parentId
    if (isSub) {
      await refreshSubtasks(selectedId.value)
      await refreshAllCounts()
      return task
    }
    await refresh()
    return task
  }

  async function remove(id: string) {
    const isSub = subtasks.value.some(t => t.id === id)
    await deleteTodoTask(id)
    if (isSub) {
      await refreshSubtasks(selectedId.value)
      await refreshAllCounts()
      return
    }
    if (selectedId.value === id)
      selectedId.value = null
    await refresh()
  }

  async function exportJson() {
    return exportTodoJson()
  }

  return {
    tasks,
    subtasks,
    subtaskProgress,
    projects,
    perspective,
    keyword,
    priorityFilter,
    projectFilter,
    viewMode,
    selectedId,
    retainedId,
    selected,
    loading,
    error,
    counts,
    countsByPriority,
    countsByProject,
    refresh,
    refreshProjects,
    refreshAllCounts,
    setPerspective,
    setPriorityFilter,
    setProjectFilter,
    setViewMode,
    setKeyword,
    select,
    create,
    update,
    complete,
    remove,
    exportJson,
  }
})
