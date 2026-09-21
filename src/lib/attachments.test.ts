import { describe, expect, it } from 'vitest'
import { attachmentExt, canPreviewAttachment, formatAttachmentSize } from './attachments'

describe('attachments helpers', () => {
  it('detects previewable types', () => {
    expect(canPreviewAttachment('photo.PNG')).toBe(true)
    expect(canPreviewAttachment('a.docx')).toBe(false)
    expect(canPreviewAttachment('x.pdf')).toBe(true)
  })

  it('normalizes extension', () => {
    expect(attachmentExt('a.Markdown')).toBe('markdown')
    expect(attachmentExt('noext')).toBe(null)
    expect(attachmentExt('virus.exe')).toBe(null)
  })

  it('formats size', () => {
    expect(formatAttachmentSize(512)).toMatch(/512/)
    expect(formatAttachmentSize(2048)).toMatch(/2/)
  })
})
