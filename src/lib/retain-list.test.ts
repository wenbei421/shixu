import { describe, expect, it } from 'vitest'
import { retainItem } from '@/lib/retain-list'

describe('retainItem', () => {
  const rows = [{ id: 'a' }, { id: 'b' }, { id: 'c' }]

  it('leaves the list alone when the item is still returned', () => {
    const result = retainItem(rows, { id: 'b' }, 1)
    expect(result.retained).toBe(false)
    expect(result.items).toEqual(rows)
  })

  it('puts a filtered-out item back at its previous index', () => {
    const result = retainItem(rows, { id: 'x' }, 1)
    expect(result.retained).toBe(true)
    expect(result.items.map(row => row.id)).toEqual(['a', 'x', 'b', 'c'])
  })

  it('appends when the previous index is unknown or past the end', () => {
    expect(retainItem(rows, { id: 'x' }, -1).items.map(row => row.id)).toEqual(['a', 'b', 'c', 'x'])
    expect(retainItem(rows, { id: 'x' }, 9).items.map(row => row.id)).toEqual(['a', 'b', 'c', 'x'])
  })
})
