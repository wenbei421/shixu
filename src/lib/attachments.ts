import { convertFileSrc, invoke } from '@tauri-apps/api/core'

export type AttachmentOwnerType = 'todo' | 'muse'

export interface Attachment {
  id: string
  ownerType: AttachmentOwnerType
  ownerId: string
  filename: string
  mimeType: string
  sizeBytes: number
  hashSha256: string
  createdAt: number
  missing: boolean
}

/** 捕获阶段尚未落库的本地文件 */
export interface PendingAttachment {
  path: string
  filename: string
  sizeBytes: number
  /** 截图写入的临时文件，添加成功后删除 */
  ephemeral?: boolean
}

export interface ClipboardAttachmentData {
  types: string[]
  text: string
  hasImage: boolean
}

/** 剪贴板是文件或图片时拦截粘贴；纯文字留给输入框。 */
export function shouldAttachClipboardData(data: ClipboardAttachmentData | null): boolean {
  if (!data)
    return false
  if (data.types.includes('Files') || data.hasImage)
    return true
  return data.text.length === 0
}

export function shouldAttachClipboard(event: ClipboardEvent): boolean {
  const data = event.clipboardData
  if (!data)
    return false
  return shouldAttachClipboardData({
    types: Array.from(data.types),
    text: data.getData('text/plain'),
    hasImage: Array.from(data.items).some(item => item.type.startsWith('image/')),
  })
}

export const MAX_ATTACHMENT_BYTES = 10 * 1024 * 1024
export const MAX_ATTACHMENTS_PER_OWNER = 5

const PREVIEW_EXT = new Set(['png', 'jpg', 'jpeg', 'gif', 'webp', 'pdf', 'md', 'markdown'])
const ALLOWED_EXT = new Set([
  ...PREVIEW_EXT,
  'docx',
  'xlsx',
  'xls',
  'zip',
  '7z',
  'rar',
])

export const ATTACHMENT_EXTENSIONS = [...ALLOWED_EXT]

export function attachmentExt(filename: string): string | null {
  const base = filename.split(/[/\\]/).pop() ?? filename
  const i = base.lastIndexOf('.')
  if (i <= 0)
    return null
  const ext = base.slice(i + 1).toLowerCase()
  return ALLOWED_EXT.has(ext) ? ext : null
}

export function canPreviewAttachment(filename: string): boolean {
  const ext = attachmentExt(filename)
  return !!ext && PREVIEW_EXT.has(ext)
}

export function formatAttachmentSize(bytes: number): string {
  if (bytes < 1024)
    return `${bytes} B`
  if (bytes < 1024 * 1024)
    return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

export function listAttachments(ownerType: AttachmentOwnerType, ownerId: string) {
  return invoke<Attachment[]>('list_attachments', { ownerType, ownerId })
}

export function addAttachment(ownerType: AttachmentOwnerType, ownerId: string, sourcePath: string) {
  return invoke<Attachment>('add_attachment', { ownerType, ownerId, sourcePath })
}

export function removeAttachment(id: string) {
  return invoke<void>('remove_attachment', { id })
}

export function getAttachmentPath(id: string) {
  return invoke<string>('get_attachment_path', { id })
}

export function openAttachment(id: string) {
  return invoke<void>('open_attachment', { id })
}

export function readAttachmentText(id: string) {
  return invoke<string>('read_attachment_text', { id })
}

export async function attachmentPreviewUrl(id: string): Promise<string> {
  const path = await getAttachmentPath(id)
  return convertFileSrc(path)
}

export function readClipboardAttachments() {
  return invoke<Array<Pick<PendingAttachment, 'path' | 'filename' | 'ephemeral'>>>('read_clipboard_attachments')
}

export function discardClipboardFile(path: string) {
  return invoke<void>('discard_clipboard_file', { path })
}

export async function discardPendingFiles(pending: PendingAttachment[]) {
  await Promise.all(pending.filter(item => item.ephemeral).map(item => discardClipboardFile(item.path).catch(() => undefined)))
}

/** 创建 owner 后批量上传；返回成功数与仍未写入的项 */
export async function flushPendingAttachments(
  ownerType: AttachmentOwnerType,
  ownerId: string,
  pending: PendingAttachment[],
): Promise<{ ok: number, errors: string[], failed: PendingAttachment[] }> {
  const errors: string[] = []
  const failed: PendingAttachment[] = []
  let ok = 0
  for (const item of pending) {
    try {
      await addAttachment(ownerType, ownerId, item.path)
      ok++
      if (item.ephemeral)
        await discardClipboardFile(item.path).catch(() => undefined)
    }
    catch (e) {
      errors.push(`${item.filename}: ${e instanceof Error ? e.message : String(e)}`)
      failed.push(item)
    }
  }
  return { ok, errors, failed }
}
