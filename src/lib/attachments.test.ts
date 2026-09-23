import { describe, expect, it } from 'vitest'
import { attachmentExt, canPreviewAttachment, formatAttachmentSize, shouldAttachClipboardData } from './attachments'

describe('attachments helpers', () => {
  it('normalizes extension without type whitelist', () => {
    expect(attachmentExt('a.Markdown')).toBe('markdown')
    expect(attachmentExt('budget.XLSX')).toBe('xlsx')
    expect(attachmentExt('legacy.xls')).toBe('xls')
    expect(attachmentExt('virus.exe')).toBe('exe')
    expect(attachmentExt('noext')).toBe(null)
  })

  it('detects previewable types', () => {
    expect(canPreviewAttachment('photo.PNG')).toBe(true)
    expect(canPreviewAttachment('a.docx')).toBe(false)
    expect(canPreviewAttachment('sheet.xlsx')).toBe(false)
    expect(canPreviewAttachment('virus.exe')).toBe(false)
    expect(canPreviewAttachment('x.pdf')).toBe(true)
  })

  it('formats size', () => {
    expect(formatAttachmentSize(512)).toMatch(/512/)
    expect(formatAttachmentSize(2048)).toMatch(/2/)
  })

  it('attaches clipboard files and images, keeps plain text', () => {
    expect(shouldAttachClipboardData({ types: ['text/plain'], text: '明天开会', hasImage: false })).toBe(false)
    expect(shouldAttachClipboardData({ types: ['Files'], text: '', hasImage: false })).toBe(true)
    expect(shouldAttachClipboardData({ types: [], text: '', hasImage: true })).toBe(true)
    expect(shouldAttachClipboardData(null)).toBe(false)
  })
})
