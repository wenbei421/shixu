/**
 * 灵感库与待办共用的快捷键判定（详细设计 4.2 / 12.1）。
 *
 * 目前只在应用窗口内生效；`Ctrl + Shift + Space` 要做到真正全局，
 * 需要接入 tauri-plugin-global-shortcut（后续迭代）。
 */

const isMac = typeof navigator !== 'undefined' && /mac/i.test(navigator.platform || navigator.userAgent)

export const MOD_LABEL = isMac ? '⌘' : 'Ctrl'
export const CAPTURE_SHORTCUT = `${MOD_LABEL} Shift Space`
export const SEARCH_SHORTCUT = `${MOD_LABEL} P`

function hasMod(event: KeyboardEvent) {
  return event.ctrlKey || event.metaKey
}

/** 呼出 / 关闭快速捕捉 */
export function isCaptureShortcut(event: KeyboardEvent) {
  return hasMod(event) && event.shiftKey && event.code === 'Space'
}

/** 聚焦搜索框 */
export function isSearchShortcut(event: KeyboardEvent) {
  return hasMod(event) && !event.shiftKey && event.key.toLowerCase() === 'p'
}

/** 归档当前灵感 */
export function isArchiveShortcut(event: KeyboardEvent) {
  return hasMod(event) && event.shiftKey && event.key.toLowerCase() === 'a'
}

/** 数字键 1..count 的下标；灵感库用它切状态，待办用它切视图 */
export function numberIndexFromKey(event: KeyboardEvent, count: number): number | null {
  if (hasMod(event) || event.altKey || event.shiftKey)
    return null
  const index = Number(event.key) - 1
  return Number.isInteger(index) && index >= 0 && index < count ? index : null
}

/** 输入态下不应触发列表快捷键 */
export function isTypingTarget(target: EventTarget | null) {
  if (!(target instanceof HTMLElement))
    return false
  return (
    target.isContentEditable
    || target.tagName === 'INPUT'
    || target.tagName === 'TEXTAREA'
    || target.tagName === 'SELECT'
  )
}
