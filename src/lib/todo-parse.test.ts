import { describe, expect, it } from 'vitest'
import { parseTodoInput } from '@/lib/todo-parse'

describe('parseTodoInput', () => {
  const now = new Date(2026, 8, 18, 10, 0, 0) // 2026-09-18 Thu

  it('parses title tags project priority and tomorrow time', () => {
    const parsed = parseTodoInput('明天 18:00 提交周报 #工作 @灵感工具 ！高', now)
    expect(parsed.title).toBe('提交周报')
    expect(parsed.tags).toEqual(['工作'])
    expect(parsed.project).toBe('灵感工具')
    expect(parsed.priority).toBe('high')
    expect(parsed.dueAt).toBe(new Date(2026, 8, 19, 18, 0, 0).getTime())
  })

  it('parses ascii priority markers', () => {
    const parsed = parseTodoInput('写周报 !high', now)
    expect(parsed.title).toBe('写周报')
    expect(parsed.priority).toBe('high')
  })

  it('parses today with default 18:00', () => {
    const parsed = parseTodoInput('今天 开会', now)
    expect(parsed.title).toBe('开会')
    expect(parsed.dueAt).toBe(new Date(2026, 8, 18, 18, 0, 0).getTime())
  })
})
