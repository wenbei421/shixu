<script setup lang="ts">
import { MinimalDocumentPreview } from '@icreate/minimal-ai-editor'
import { useColorMode, usePreferredDark } from '@vueuse/core'
import { computed, onErrorCaptured } from 'vue'

withDefaults(defineProps<{
  html?: string
  height?: string
}>(), {
  html: '',
  height: '100%',
})

const mode = useColorMode({ emitAuto: true })
const preferredDark = usePreferredDark()

const editorTheme = computed<'light' | 'dark'>(() => {
  if (mode.value === 'dark')
    return 'dark'
  if (mode.value === 'light')
    return 'light'
  return preferredDark.value ? 'dark' : 'light'
})

/** 对齐示例：不锁定 defaultBackground */
const pageOptions = {
  layouts: ['web'] as const,
}

onErrorCaptured((err) => {
  const msg = err instanceof Error ? err.message : String(err ?? '')
  if (
    msg.includes('editor view is not available')
    || msg.includes('BubbleMenu')
    || msg.includes('subTree')
  ) {
    return false
  }
})
</script>

<template>
  <div class="inspiration-preview bg-background text-foreground border-border min-h-0 flex-1 overflow-hidden rounded-md border">
    <div class="inspiration-preview__mount h-full min-h-0 w-full">
      <MinimalDocumentPreview
        :html="html"
        layout="web"
        :theme="editorTheme"
        :page="pageOptions"
        :height="height"
      />
    </div>
  </div>
</template>

<style scoped>
.inspiration-preview {
  --minimal-content-text-color: var(--foreground);
  --minimal-content-muted-color: var(--muted-foreground);
  --minimal-fg: var(--foreground);
  --minimal-bg: var(--background);
  --minimal-surface: var(--card);
  --minimal-border: var(--border);
  --minimal-border-color: var(--border);
}

.inspiration-preview :deep(.minimal-document-preview__surface),
.inspiration-preview :deep(.minimal-document-preview__prose) {
  box-shadow: none !important;
  color: var(--minimal-content-text-color) !important;
}
</style>
