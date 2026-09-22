<script setup lang="ts">
import type { Attachment, PendingAttachment } from '@/lib/attachments'
import { FileText, Image, Paperclip, X } from '@lucide/vue'
import { useI18n } from 'vue-i18n'
import {
  attachmentExt,
  canPreviewAttachment,
  formatAttachmentSize,
  openAttachment,
} from '@/lib/attachments'

defineProps<{
  saved?: Attachment[]
  pending?: PendingAttachment[]
}>()

const emit = defineEmits<{
  removeSaved: [id: string]
  removePending: [path: string]
  preview: [item: Attachment]
  error: [message: string]
}>()

const { t } = useI18n()

function iconFor(filename: string) {
  const ext = attachmentExt(filename)
  if (ext && ['png', 'jpg', 'jpeg', 'gif', 'webp'].includes(ext))
    return Image
  if (ext && ['pdf', 'md', 'markdown', 'docx', 'xlsx', 'xls'].includes(ext))
    return FileText
  return Paperclip
}

async function onSavedClick(item: Attachment) {
  if (item.missing) {
    emit('error', t('attachments.missing'))
    return
  }
  if (canPreviewAttachment(item.filename)) {
    emit('preview', item)
    return
  }
  try {
    await openAttachment(item.id)
  }
  catch (e) {
    emit('error', e instanceof Error ? e.message : String(e))
  }
}
</script>

<template>
  <ul v-if="(saved?.length || pending?.length)" class="space-y-1">
    <li
      v-for="item in pending"
      :key="item.path"
      class="bg-muted/40 flex items-center gap-2 rounded-md px-2 py-1 text-xs"
    >
      <component :is="iconFor(item.filename)" class="text-muted-foreground size-3.5 shrink-0" />
      <span class="min-w-0 flex-1 truncate">{{ item.filename }}</span>
      <button
        type="button"
        class="text-muted-foreground hover:text-destructive p-0.5"
        :title="t('attachments.remove')"
        @click="emit('removePending', item.path)"
      >
        <X class="size-3.5" />
      </button>
    </li>
    <li
      v-for="item in saved"
      :key="item.id"
      class="hover:bg-muted/50 flex items-center gap-2 rounded-md px-2 py-1 text-xs"
    >
      <button
        type="button"
        class="flex min-w-0 flex-1 items-center gap-2 text-left"
        @click="onSavedClick(item)"
      >
        <component :is="iconFor(item.filename)" class="text-muted-foreground size-3.5 shrink-0" />
        <span class="min-w-0 flex-1 truncate" :class="item.missing ? 'text-destructive' : ''">
          {{ item.filename }}
        </span>
        <span class="text-muted-foreground shrink-0 tabular-nums">
          {{ item.missing ? t('attachments.missing') : formatAttachmentSize(item.sizeBytes) }}
        </span>
      </button>
      <button
        type="button"
        class="text-muted-foreground hover:text-destructive p-0.5"
        :title="t('attachments.remove')"
        @click="emit('removeSaved', item.id)"
      >
        <X class="size-3.5" />
      </button>
    </li>
  </ul>
</template>
