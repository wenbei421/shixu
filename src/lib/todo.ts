import { invoke } from '@tauri-apps/api/core'

export type TodoStatus = 'todo' | 'doing' | 'done' | 'cancelled'
export type TodoPriority = 'high' | 'medium' | 'low' | 'none'
export type TodoPerspective = 'inbox' | 'today' | 'upcoming' | 'all' | 'done'
export type TodoViewMode = 'card' | 'board' | 'timeline'

export const TODO_STATUS_ORDER: TodoStatus[] = ['todo', 'doing', 'done', 'cancelled']
export const TODO_PRIORITY_FILTERS: Array<TodoPriority | 'all'> = ['all', 'high', 'medium', 'low', 'none']
export const TODO_VIEW_ORDER: TodoViewMode[] = ['card', 'board', 'timeline']

export interface TodoExportPayload {
  exportedAt: number
  tasks: TodoTask[]
}

export interface TodoTask {
  id: string
  title: string
  description: string
  status: TodoStatus
  priority: TodoPriority
  dueAt?: number | null
  projectId?: string | null
  projectName?: string | null
  parentId?: string | null
  sortOrder: number
  completedAt?: number | null
  source: string
  tags: string[]
  createdAt: number
  updatedAt: number
}

export interface ListTodoQuery {
  perspective?: TodoPerspective
  keyword?: string
  priority?: TodoPriority | 'all'
}

export interface CreateTodoPayload {
  title: string
  description?: string
  status?: TodoStatus
  priority?: TodoPriority
  dueAt?: number | null
  projectId?: string | null
  parentId?: string | null
  tagNames?: string[]
  source?: string
}

export interface UpdateTodoPayload {
  title?: string
  description?: string
  status?: TodoStatus
  priority?: TodoPriority
  dueAt?: number | null
  projectId?: string | null
  tagNames?: string[]
}

export function listTodoTasks(query: ListTodoQuery = {}) {
  return invoke<TodoTask[]>('list_todo_tasks', { query })
}

export function listTodoSubtasks(parentId: string) {
  return invoke<TodoTask[]>('list_todo_subtasks', { parentId })
}

export function createTodoTask(req: CreateTodoPayload) {
  return invoke<TodoTask>('create_todo_task', { req })
}

export function updateTodoTask(id: string, req: UpdateTodoPayload) {
  return invoke<TodoTask>('update_todo_task', { id, req })
}

export function completeTodoTask(id: string, done: boolean) {
  return invoke<TodoTask>('complete_todo_task', { id, done })
}

export function deleteTodoTask(id: string) {
  return invoke<void>('delete_todo_task', { id })
}

export function exportTodoJson() {
  return invoke<TodoExportPayload>('export_todo_json')
}
