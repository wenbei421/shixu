<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  attachmentExt,
  attachmentPreviewUrl,
  readAttachmentText,
  type Attachment,
} from '@/lib/attachments'
import { renderMarkdown } from '@/lib/markdown'

const props = defineProps<{
  open: boolean
  attachment: Attachment | null
}>()

const emit = defineEmits<{ close: [] }>()

const { t } = useI18n()
const url = ref('')
const markdown = ref('')
const error = ref('')
const loading = ref(false)

const kind = computed(() => {
  const ext = attachmentExt(props.attachment?.filename ?? '')
  if (!ext)
    return 'other'
  if (['png', 'jpg', 'jpeg', 'gif', 'webp'].includes(ext))
    return 'image'
  if (ext === 'pdf')
    return 'pdf'
  if (ext === 'md' || ext === 'markdown')
    return 'md'
  return 'other'
})

const html = computed(() => (kind.value === 'md' ? renderMarkdown(markdown.value) : ''))

watch(
  () => [props.open, props.attachment?.id] as const,
  async ([open, id]) => {
    url.value = ''
    markdown.value = ''
    error.value = ''
    if (!open || !id || !props.attachment)
      return
    loading.value = true
    try {
      if (kind.value === 'md')
        markdown.value = await readAttachmentText(id)
      else if (kind.value === 'image' || kind.value === 'pdf')
        url.value = await attachmentPreviewUrl(id)
    }
    catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
    finally {
      loading.value = false
    }
  },
)
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open && attachment"
      class="fixed inset-0 z-[100] flex items-center justify-center bg-black/50 p-6"
      @click.self="emit('close')"
    >
      <div class="bg-background flex max-h-[85vh] w-full max-w-3xl flex-col overflow-hidden rounded-xl border shadow-2xl">
        <header class="flex items-center gap-2 border-b px-4 py-3">
          <span class="min-w-0 flex-1 truncate text-sm font-medium">{{ attachment.filename }}</span>
          <button
            type="button"
            class="text-muted-foreground hover:bg-muted rounded-md px-2 py-1 text-xs"
            @click="emit('close')"
          >
            {{ t('attachments.close') }}
          </button>
        </header>
        <div class="min-h-0 flex-1 overflow-auto p-4">
          <p v-if="loading" class="text-muted-foreground text-sm">
            {{ t('attachments.preview') }}
          </p>
          <p v-else-if="error" class="text-destructive text-sm">
            {{ error }}
          </p>
          <img
            v-else-if="kind === 'image' && url"
            :src="url"
            :alt="attachment.filename"
            class="mx-auto max-h-[70vh] max-w-full object-contain"
          >
          <iframe
            v-else-if="kind === 'pdf' && url"
            :src="url"
            class="h-[70vh] w-full"
            :title="attachment.filename"
          />
          <article
            v-else-if="kind === 'md'"
            class="prose prose-sm dark:prose-invert max-w-none"
            v-html="html"
          />
        </div>
      </div>
    </div>
  </Teleport>
</template>
