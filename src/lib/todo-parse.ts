import type { TodoPriority } from '@/lib/todo'

export interface ParsedTodoInput {
  title: string
  dueAt: number | null
  priority: TodoPriority
  tags: string[]
  project: string | null
}

const WEEKDAY_MAP: Record<string, number> = {
  周日: 0,
  星期日: 0,
  周天: 0,
  星期一: 1,
  周一: 1,
  星期二: 2,
  周二: 2,
  星期三: 3,
  周三: 3,
  星期四: 4,
  周四: 4,
  星期五: 5,
  周五: 5,
  星期六: 6,
  周六: 6,
}

function startOfLocalDay(base = new Date()) {
  return new Date(base.getFullYear(), base.getMonth(), base.getDate())
}

function atLocalTime(day: Date, hours: number, minutes: number) {
  return new Date(
    day.getFullYear(),
    day.getMonth(),
    day.getDate(),
    hours,
    minutes,
    0,
    0,
  ).getTime()
}

function nextWeekday(target: number, from = new Date()) {
  const day = startOfLocalDay(from)
  const current = day.getDay()
  let delta = (target - current + 7) % 7
  if (delta === 0)
    delta = 7
  day.setDate(day.getDate() + delta)
  return day
}

/**
 * 解析快速添加自然语言（详细设计 12.2）
 * 例：`明天 18:00 提交周报 #工作 @灵感工具 ！高`
 */
export function parseTodoInput(raw: string, now = new Date()): ParsedTodoInput {
  let text = raw.trim()
  const tags: string[] = []
  let project: string | null = null
  let priority: TodoPriority = 'none'
  let dueAt: number | null = null
  let hour = 9
  let minute = 0
  let day: Date | null = null

  // 标签 #xxx
  text = text.replace(/(^|\s)#([^\s#@！!]+)/g, (_, lead: string, name: string) => {
    tags.push(name)
    return lead
  })

  // 项目 @xxx
  text = text.replace(/(^|\s)@([^\s#@！!]+)/g, (_, lead: string, name: string) => {
    project = name
    return lead
  })

  // 优先级
  // `!` 是非单词字符，前面不能用 \b，否则 `!high` 永不匹配
  if (/[！!](?:高|high\b)/i.test(text)) {
    priority = 'high'
    text = text.replace(/[！!](?:高|high\b)/gi, ' ')
  }
  else if (/[！!](?:中|medium\b)/i.test(text)) {
    priority = 'medium'
    text = text.replace(/[！!](?:中|medium\b)/gi, ' ')
  }
  else if (/[！!](?:低|low\b)/i.test(text)) {
    priority = 'low'
    text = text.replace(/[！!](?:低|low\b)/gi, ' ')
  }

  // 时间 HH:mm（可覆盖默认）
  const timeMatch = text.match(/(?:^|\s)(\d{1,2})[:：](\d{2})(?:\s|$)/)
  if (timeMatch) {
    hour = Number(timeMatch[1])
    minute = Number(timeMatch[2])
    if (hour >= 0 && hour <= 23 && minute >= 0 && minute <= 59) {
      text = text.replace(timeMatch[0], ' ')
    }
    else {
      hour = 9
      minute = 0
    }
  }

  // 相对/星期日期
  if (/(?:^|\s)今天(?:\s|$)/.test(text)) {
    day = startOfLocalDay(now)
    hour = timeMatch ? hour : 18
    text = text.replace(/(?:^|\s)今天(?=\s|$)/g, ' ')
  }
  else if (/(?:^|\s)明天(?:\s|$)/.test(text)) {
    day = startOfLocalDay(now)
    day.setDate(day.getDate() + 1)
    text = text.replace(/(?:^|\s)明天(?=\s|$)/g, ' ')
  }
  else if (/(?:^|\s)后天(?:\s|$)/.test(text)) {
    day = startOfLocalDay(now)
    day.setDate(day.getDate() + 2)
    text = text.replace(/(?:^|\s)后天(?=\s|$)/g, ' ')
  }
  else if (/(?:^|\s)下周(?:\s|$)/.test(text)) {
    day = startOfLocalDay(now)
    day.setDate(day.getDate() + 7)
    text = text.replace(/(?:^|\s)下周(?=\s|$)/g, ' ')
  }
  else {
    for (const [label, weekday] of Object.entries(WEEKDAY_MAP)) {
      const re = new RegExp(`(?:^|\\s)${label}(?=\\s|$)`)
      if (re.test(text)) {
        day = nextWeekday(weekday, now)
        text = text.replace(re, ' ')
        break
      }
    }
  }

  if (day)
    dueAt = atLocalTime(day, hour, minute)
  else if (timeMatch)
    dueAt = atLocalTime(startOfLocalDay(now), hour, minute)

  const title = text.replace(/\s+/g, ' ').trim()

  return {
    title,
    dueAt,
    priority,
    tags,
    project,
  }
}
