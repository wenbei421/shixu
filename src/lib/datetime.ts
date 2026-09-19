/**
 * 应用统一时间格式：yyyy-MM-dd / yyyy-MM-dd HH:mm / yyyy-MM-dd HH:mm:ss
 *
 * 使用本地时区（不强行 UTC），因为拾序是个本地化个人工具，跟用户的作息一致。
 * locale 不影响格式串结构，只影响数字字符本身（locale-agnostic for ASCII digits）。
 *
 * 用 `Intl.DateTimeFormat` 的 `formatToParts` 手工拼装，避免 `toISOString` 强制 UTC。
 */

/** 把数字补零到指定宽度 */
function pad(n: number, width = 2) {
  return String(n).padStart(width, '0')
}

function parts(ms: number) {
  const d = new Date(ms)
  return {
    y: d.getFullYear(),
    m: d.getMonth() + 1,
    d: d.getDate(),
    hh: d.getHours(),
    mm: d.getMinutes(),
    ss: d.getSeconds(),
  }
}

/** 仅日期：`2026-09-19` */
export function formatDate(ms: number | Date): string {
  const t = typeof ms === 'number' ? ms : ms.getTime()
  const p = parts(t)
  return `${p.y}-${pad(p.m)}-${pad(p.d)}`
}

/** 日期 + 分钟：`2026-09-19 18:30` */
export function formatDateTime(ms: number | Date): string {
  const t = typeof ms === 'number' ? ms : ms.getTime()
  const p = parts(t)
  return `${formatDate(t)} ${pad(p.hh)}:${pad(p.mm)}`
}

/** 日期 + 秒：`2026-09-19 18:30:45` */
export function formatDateTimeSeconds(ms: number | Date): string {
  const t = typeof ms === 'number' ? ms : ms.getTime()
  const p = parts(t)
  return `${formatDate(t)} ${pad(p.hh)}:${pad(p.mm)}:${pad(p.ss)}`
}

/** 仅时间（HH:mm）：用于行内紧凑场景，状态栏/时间线条目 */
export function formatTime(ms: number | Date): string {
  const t = typeof ms === 'number' ? ms : ms.getTime()
  const p = parts(t)
  return `${pad(p.hh)}:${pad(p.mm)}`
}

/** 仅时间（HH:mm:ss）：用于精度要求高的场景 */
export function formatTimeSeconds(ms: number | Date): string {
  const t = typeof ms === 'number' ? ms : ms.getTime()
  const p = parts(t)
  return `${pad(p.hh)}:${pad(p.mm)}:${pad(p.ss)}`
}
