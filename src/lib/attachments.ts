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
}

export const MAX_ATTACHMENT_BYTES = 10 * 1024 * 1024
export const MAX_ATTACHMENTS_PER_OWNER = 5

const PREVIEW_EXT = new Set(['png', 'jpg', 'jpeg', 'gif', 'webp', 'pdf', 'md', 'markdown'])
const ALLOWED_EXT = new Set([
  ...PREVIEW_EXT,
  'docx',
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

/** 创建 owner 后批量上传；返回成功数与错误信息 */
export async function flushPendingAttachments(
  ownerType: AttachmentOwnerType,
  ownerId: string,
  pending: PendingAttachment[],
): Promise<{ ok: number, errors: string[] }> {
  const errors: string[] = []
  let ok = 0
  for (const item of pending) {
    try {
      await addAttachment(ownerType, ownerId, item.path)
      ok++
    }
    catch (e) {
      errors.push(`${item.filename}: ${e instanceof Error ? e.message : String(e)}`)
    }
  }
  return { ok, errors }
}
