import DOMPurify from 'dompurify'
import { marked } from 'marked'

marked.use({
  async: false,
  breaks: true,
  gfm: true,
})

/** DeepSeek 回复按 Markdown 解析，并去掉脚本等不安全标签。 */
export function renderMarkdown(source: string) {
  const text = source.trim()
  if (!text)
    return ''

  const raw = marked.parse(text)
  const html = typeof raw === 'string' ? raw : ''
  return DOMPurify.sanitize(html, {
    USE_PROFILES: { html: true },
  })
}
