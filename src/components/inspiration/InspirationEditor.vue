<script setup lang="ts">
/**
 * 对齐 RDPMS `IcsRichEditor` + minimal-editor `examples/vue3-integration`：
 * - 外层盒子定高；theme 跟随应用实际明暗（与示例 theme 行为一致）
 * - 不锁定 page.defaultBackground，暗色下由编辑器切换正文/纸张 token
 * - isAlive / 错误捕获避免卸载期 TipTap 噪音
 */
import { MinimalAiEditor } from '@icreate/minimal-ai-editor'
import { useColorMode, usePreferredDark } from '@vueuse/core'
import {
  computed,
  onBeforeUnmount,
  onErrorCaptured,
  onMounted,
  onUnmounted,
  ref,
  watch,
} from 'vue'

const props = withDefaults(defineProps<{
  modelValue?: string
  editorKey?: string
}>(), {
  modelValue: '',
  editorKey: 'inspiration-editor',
})

const emit = defineEmits<{
  'update:modelValue': [html: string]
  'update:text': [text: string]
}>()

const isAlive = ref(true)
const mode = useColorMode({ emitAuto: true })
const preferredDark = usePreferredDark()

/** 与示例一致传 light/dark；auto 时按系统偏好解析，避免正文仍用浅色 token */
const editorTheme = computed<'light' | 'dark'>(() => {
  if (mode.value === 'dark')
    return 'dark'
  if (mode.value === 'light')
    return 'light'
  return preferredDark.value ? 'dark' : 'light'
})

const html = ref(props.modelValue)

/** 对齐示例：不写 defaultBackground，暗色主题才能切换正文色与纸张默认色 */
const pageOptions = {
  layouts: ['web'] as const,
  showLineNumber: false,
  showToc: false,
}

watch(
  () => props.modelValue,
  (v) => {
    if (!isAlive.value)
      return
    if (v !== html.value)
      html.value = v
  },
)

watch(html, (v) => {
  if (!isAlive.value)
    return
  emit('update:modelValue', v)
})

const MAX_IMAGE_BYTES = 2 * 1024 * 1024

async function uploadImage(file: File): Promise<string> {
  if (file.size > MAX_IMAGE_BYTES)
    throw new Error('Image exceeds 2MB limit')
  const dataUrl = await readAsDataUrl(file)
  return dataUrl
}

function readAsDataUrl(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => resolve(String(reader.result))
    reader.onerror = () => reject(reader.error ?? new Error('read failed'))
    reader.readAsDataURL(file)
  })
}

function onAutoSave(payload: { html: string, text: string }) {
  if (!isAlive.value)
    return
  html.value = payload.html
  emit('update:text', payload.text)
}

onBeforeUnmount(() => {
  isAlive.value = false
})

onErrorCaptured((err) => {
  const msg = err instanceof Error ? err.message : String(err ?? '')
  if (
    msg.includes('editor view is not available')
    || msg.includes('BubbleMenu')
    || msg.includes('subTree')
    || msg.includes('startsWith')
    || msg.includes('unregisterPlugin')
    || msg.includes('__vnode')
  ) {
    return false
  }
})

function handlePromiseRejection(event: PromiseRejectionEvent) {
  const msg = event.reason instanceof Error
    ? event.reason.message
    : String(event.reason ?? '')
  if (
    msg.includes('editor view is not available')
    || msg.includes('startsWith')
    || msg.includes('__vnode')
  ) {
    event.preventDefault()
  }
}

onMounted(() => {
  window.addEventListener('unhandledrejection', handlePromiseRejection)
})

onUnmounted(() => {
  window.removeEventListener('unhandledrejection', handlePromiseRejection)
})
</script>

<template>
  <div class="inspiration-editor bg-background text-foreground border-border flex min-h-0 flex-1 flex-col overflow-hidden rounded-md border">
    <div class="inspiration-editor__mount min-h-0 flex-1">
      <MinimalAiEditor
        :model-html="html"
        height="100%"
        :theme="editorTheme"
        :editor-key="editorKey"
        toolbar-preset="note"
        :page="pageOptions"
        :persist-document="false"
        :persist-ai-session="false"
        :features="{ markdown: true, search: false, export: false, page: false, table: false }"
        :file-max-size="MAX_IMAGE_BYTES"
        :upload-image="uploadImage"
        :auto-save="{ enabled: true, interval: 800 }"
        :auto-save-handler="onAutoSave"
        @update:model-html="(v) => { if (isAlive) html = v }"
      />
    </div>
  </div>
</template>

<style scoped>
.inspiration-editor {
  /* 与宿主 shadcn token 对齐，暗色下正文跟 --foreground（浅色） */
  --minimal-primary-color: var(--primary);
  --minimal-content-text-color: var(--foreground);
  --minimal-content-muted-color: var(--muted-foreground);
  --minimal-fg: var(--foreground);
  --minimal-bg: var(--background);
  --minimal-surface: var(--card);
  --minimal-border: var(--border);
  --minimal-border-color: var(--border);
}

.inspiration-editor__mount {
  width: 100%;
  height: 100%;
  min-height: 0;
}

.inspiration-editor :deep(.minimal-editor-prose) {
  min-height: 12rem;
  padding: 0.75rem 1rem !important;
  color: var(--minimal-content-text-color) !important;
}
</style>
