/** 服务端筛掉当前记录时，把它插回列表里的原位置。已在结果中则不插。 */
export function retainItem<T extends { id: string }>(
  items: T[],
  item: T,
  index: number,
): { items: T[], retained: boolean } {
  if (items.some(row => row.id === item.id))
    return { items, retained: false }
  const next = items.slice()
  const at = index < 0 ? next.length : Math.min(index, next.length)
  next.splice(at, 0, item)
  return { items: next, retained: true }
}
