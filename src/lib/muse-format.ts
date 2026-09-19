/** Muse 灵感库的展示层格式化：相对时间、时间线分组、标签取色、搜索高亮。 */

import { formatDate, formatDateTime } from './datetime'

const MINUTE = 60_000
const HOUR = 3_600_000
const DAY = 86_400_000

/** 与原型 `PALETTE` 一致，后端新建标签时也用同一张表 */
const TAG_PALETTE = [
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
]

/** 标签未存色值时按名称哈希兜底，保证同名标签颜色稳定 */
export function tagColor(name: string): string {
  let hash = 0
  for (let i = 0; i < name.length; i++)
    hash = (hash * 31 + name.charCodeAt(i)) >>> 0
  return TAG_PALETTE[hash % TAG_PALETTE.length] ?? '#5b5bd6'
}

function startOfDay(ts: number): number {
  const d = new Date(ts)
  d.setHours(0, 0, 0, 0)
  return d.getTime()
}

/** 相对时间：「刚刚 / 3 分钟前 / 2 天前」，超过 7 天显示 `yyyy-MM-dd HH:mm` */
export function relativeTime(ts: number, locale: string, now = Date.now()): string {
  const diff = Math.max(now - ts, 0)
  const rtf = new Intl.RelativeTimeFormat(locale, { numeric: 'auto' })

  if (diff < MINUTE)
    return rtf.format(0, 'minute')
  if (diff < HOUR)
    return rtf.format(-Math.floor(diff / MINUTE), 'minute')
  if (diff < DAY)
    return rtf.format(-Math.floor(diff / HOUR), 'hour')
  if (diff < 7 * DAY)
    return rtf.format(-Math.floor(diff / DAY), 'day')

  return formatDateTime(ts)
}

/** 时间线分组名：今天 / 昨天 / N 天前 / yyyy-MM-dd（详细设计 5.3.4） */
export function dateGroupKey(ts: number, locale: string, now = Date.now()): string {
  const diffDays = Math.round((startOfDay(now) - startOfDay(ts)) / DAY)
  if (diffDays <= 0)
    return new Intl.RelativeTimeFormat(locale, { numeric: 'auto' }).format(0, 'day')
  if (diffDays < 7)
    return new Intl.RelativeTimeFormat(locale, { numeric: 'auto' }).format(-diffDays, 'day')

  return formatDate(ts)
}

export interface TextSegment {
  text: string
  hit: boolean
}

/**
 * 把正文按关键词切成片段，命中的片段由模板渲染为 `<mark>`。
 * 用分段而非 `v-html`，避免把用户内容当 HTML 注入。
 */
export function highlightSegments(text: string, keyword: string): TextSegment[] {
  const q = keyword.trim()
  if (!q)
    return [{ text, hit: false }]

  const lower = text.toLowerCase()
  const needle = q.toLowerCase()
  const segments: TextSegment[] = []
  let cursor = 0

  while (cursor < text.length) {
    const found = lower.indexOf(needle, cursor)
    if (found === -1)
      break
    if (found > cursor)
      segments.push({ text: text.slice(cursor, found), hit: false })
    segments.push({ text: text.slice(found, found + needle.length), hit: true })
    cursor = found + needle.length
  }

  if (cursor < text.length)
    segments.push({ text: text.slice(cursor), hit: false })

  return segments.length ? segments : [{ text, hit: false }]
}

/** 回顾视图的稳定洗牌：同一 salt 下顺序固定，换 salt 即重新洗牌 */
export function shuffleBySalt<T extends { id: string }>(items: T[], salt: number): T[] {
  const weight = (id: string) => {
    let hash = 2166136261
    const key = `${id}${salt}`
    for (let i = 0; i < key.length; i++) {
      hash ^= key.charCodeAt(i)
      hash = Math.imul(hash, 16777619)
    }
    return hash >>> 0
  }
  return [...items].sort((a, b) => weight(a.id) - weight(b.id))
}
