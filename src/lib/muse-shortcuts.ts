/**
 * Muse 快捷键（详细设计 4.2）。
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

/** 数字键 1–4 对应四个状态（详细设计 4.2） */
export function statusIndexFromKey(event: KeyboardEvent): number | null {
  if (hasMod(event) || event.altKey || event.shiftKey)
    return null
  const index = ['1', '2', '3', '4'].indexOf(event.key)
  return index === -1 ? null : index
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
