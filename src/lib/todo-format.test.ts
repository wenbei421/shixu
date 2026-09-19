import { describe, expect, it } from 'vitest'
import { formatDue, isOverdue } from '@/lib/todo-format'

describe('isOverdue', () => {
  const now = new Date(2026, 8, 18, 10, 0, 0).getTime()

  it('marks past due unfinished tasks', () => {
    expect(isOverdue(now - 1000, 'todo', now)).toBe(true)
    expect(isOverdue(now - 1000, 'doing', now)).toBe(true)
  })

  it('ignores future, finished and undated tasks', () => {
    expect(isOverdue(now + 1000, 'todo', now)).toBe(false)
    expect(isOverdue(now - 1000, 'done', now)).toBe(false)
    expect(isOverdue(now - 1000, 'cancelled', now)).toBe(false)
    expect(isOverdue(null, 'todo', now)).toBe(false)
  })
})

describe('formatDue', () => {
  const now = new Date(2026, 8, 18, 10, 0, 0).getTime()

  it('drops the date for today and uses HH:mm', () => {
    const due = new Date(2026, 8, 18, 18, 30).getTime()
    expect(formatDue(due, 'en-US', now)).toBe('18:30')
  })

  it('uses yyyy-MM-dd HH:mm within the same year', () => {
    const due = new Date(2026, 8, 20, 18, 30).getTime()
    expect(formatDue(due, 'en-US', now)).toBe('2026-09-20 18:30')
  })

  it('uses yyyy-MM-dd HH:mm across years', () => {
    const due = new Date(2027, 0, 5, 18, 30).getTime()
    expect(formatDue(due, 'en-US', now)).toBe('2027-01-05 18:30')
  })
})
