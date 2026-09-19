import { describe, expect, it } from 'vitest'
import { formatDate, formatDateTime, formatDateTimeSeconds, formatTime, formatTimeSeconds } from '@/lib/datetime'

describe('datetime formatters', () => {
  // 2026-09-19 18:30:45 本地时间
  const sample = new Date(2026, 8, 19, 18, 30, 45).getTime()

  it('formatDate produces yyyy-MM-dd with zero padding', () => {
    const early = new Date(2026, 0, 3, 9, 5, 7).getTime()
    expect(formatDate(early)).toBe('2026-01-03')
    expect(formatDate(sample)).toBe('2026-09-19')
  })

  it('formatDateTime produces yyyy-MM-dd HH:mm', () => {
    const early = new Date(2026, 0, 3, 9, 5, 7).getTime()
    expect(formatDateTime(early)).toBe('2026-01-03 09:05')
    expect(formatDateTime(sample)).toBe('2026-09-19 18:30')
  })

  it('formatDateTimeSeconds produces yyyy-MM-dd HH:mm:ss', () => {
    expect(formatDateTimeSeconds(sample)).toBe('2026-09-19 18:30:45')
  })

  it('formatTime produces HH:mm', () => {
    expect(formatTime(sample)).toBe('18:30')
    const early = new Date(2026, 0, 3, 9, 5, 7).getTime()
    expect(formatTime(early)).toBe('09:05')
  })

  it('formatTimeSeconds produces HH:mm:ss', () => {
    expect(formatTimeSeconds(sample)).toBe('18:30:45')
  })

  it('accepts both number and Date inputs', () => {
    expect(formatDateTime(new Date(sample))).toBe('2026-09-19 18:30')
    expect(formatDateTime(sample)).toBe('2026-09-19 18:30')
  })
})
