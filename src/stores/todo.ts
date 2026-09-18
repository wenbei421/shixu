import type { MuseProject } from '@/lib/muse'
import type {
  CreateTodoPayload,
  TodoPerspective,
  TodoPriority,
  TodoTask,
  TodoViewMode,
  UpdateTodoPayload,
} from '@/lib/todo'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { listProjects } from '@/lib/muse'
import {
  completeTodoTask,
  createTodoTask,
  deleteTodoTask,
  exportTodoJson,
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
  const viewMode = ref<TodoViewMode>('card')
  const selectedId = ref<string | null>(null)
  const loading = ref(false)
  const error = ref('')

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

  async function refreshSubtasks(parentId: string | null = selectedId.value) {
    if (!parentId) {
      subtasks.value = []
      return
    }
    subtasks.value = await listTodoSubtasks(parentId)
  }

  async function refresh() {
    loading.value = true
    error.value = ''
    try {
      // 项目列表只在自己变动后重拉，别跟着每次搜索一起走
      const pending: Promise<unknown>[] = [
        listTodoTasks({
          perspective: perspective.value,
          keyword: keyword.value.trim() || undefined,
          priority: priorityFilter.value,
        }),
      ]
      if (!projectsLoaded)
        pending.push(refreshProjects())
      const [nextTasks] = await Promise.all(pending) as [TodoTask[], ...unknown[]]
      tasks.value = nextTasks
      if (selectedId.value && !tasks.value.some(t => t.id === selectedId.value)) {
        selectedId.value = tasks.value[0]?.id ?? null
      }
      await refreshSubtasks(selectedId.value)
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

  function setViewMode(next: TodoViewMode) {
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
    selectedId.value = id
    await refreshSubtasks(id)
  }

  async function create(payload: CreateTodoPayload) {
    const task = await createTodoTask(payload)
    if (payload.parentId) {
      await refreshSubtasks(payload.parentId)
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
      return task
    }
    await refresh()
    if (keep && tasks.value.some(t => t.id === keep))
      selectedId.value = keep
    else if (keep === id && !tasks.value.some(t => t.id === id))
      selectedId.value = null
    return task
  }

  async function complete(id: string, done: boolean) {
    const task = await completeTodoTask(id, done)
    const isSub = subtasks.value.some(t => t.id === id) || !!task.parentId
    if (isSub) {
      await refreshSubtasks(selectedId.value)
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
    viewMode,
    selectedId,
    selected,
    loading,
    error,
    refresh,
    refreshProjects,
    setPerspective,
    setPriorityFilter,
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
