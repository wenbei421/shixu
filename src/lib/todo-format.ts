/** 待办到期时间的展示层格式化。 */

import type { TodoPriority, TodoStatus, TodoTask } from '@/lib/todo'
import { formatTime, formatDateTime } from '@/lib/datetime'
import { relativeTime } from '@/lib/muse-format'

function startOfDay(ms: number) {
  const d = new Date(ms)
  return new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime()
}

/** 已过期且还没收尾，列表里标红用 */
export function isOverdue(
  dueAt: number | null | undefined,
  status: TodoStatus,
  now = Date.now(),
) {
  if (dueAt == null || status === 'done' || status === 'cancelled')
    return false
  return dueAt < now
}

/** 今天只显示 `HH:mm`，否则显示 `yyyy-MM-dd HH:mm` */
export function formatDue(dueAt: number, _locale: string, now = Date.now()) {
  if (startOfDay(dueAt) === startOfDay(now))
    return formatTime(dueAt)
  return formatDateTime(dueAt)
}

/** 列表右侧时间戳：已完成显示「完成于 X」，否则显示「X 前」 */
export function formatListTimestamp(task: Pick<TodoTask, 'completedAt' | 'updatedAt' | 'status'>, locale: string, now = Date.now()) {
  const ts = task.status === 'done' && task.completedAt ? task.completedAt : task.updatedAt
  return relativeTime(ts, locale, now)
}

/** 优先级小色点用的颜色，与左栏过滤器保持一致 */
export function priorityDotClass(priority: TodoPriority) {
  switch (priority) {
    case 'high':
      return 'bg-red-500'
    case 'medium':
      return 'bg-amber-500'
    case 'low':
      return 'bg-emerald-500'
    default:
      return 'bg-muted-foreground/40'
  }
}

/** 优先级选中时的文字色，和色点同一套 */
export function priorityTextClass(priority: TodoPriority) {
  switch (priority) {
    case 'high':
      return 'text-red-500'
    case 'medium':
      return 'text-amber-500'
    case 'low':
      return 'text-emerald-500'
    default:
      return 'text-foreground'
  }
}

/** 状态色点：todo 灰、doing 蓝、done 翠绿、cancelled 红 */
export function statusDotClass(status: TodoStatus) {
  switch (status) {
    case 'doing':
      return 'bg-blue-500'
    case 'done':
      return 'bg-emerald-500'
    case 'cancelled':
      return 'bg-red-400'
    default:
      return 'bg-muted-foreground/50'
  }
}

/** 状态文字色，详情角标和选中按钮共用 */
export function statusTextClass(status: TodoStatus) {
  switch (status) {
    case 'doing':
      return 'text-blue-500'
    case 'done':
      return 'text-emerald-500'
    case 'cancelled':
      return 'text-red-400'
    default:
      return 'text-muted-foreground'
  }
}
