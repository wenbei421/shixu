<script setup lang="ts">
import { Paperclip } from '@lucide/vue'
import { open } from '@tauri-apps/plugin-dialog'
import { useI18n } from 'vue-i18n'
import {
  ATTACHMENT_EXTENSIONS,
  MAX_ATTACHMENT_BYTES,
  attachmentExt,
  type Attachment,
  type AttachmentOwnerType,
  type PendingAttachment,
  addAttachment,
} from '@/lib/attachments'

const props = defineProps<{
  disabled?: boolean
  remaining: number
  mode: 'pending' | 'owner'
  ownerType?: AttachmentOwnerType
  ownerId?: string
}>()

const emit = defineEmits<{
  added: [item: Attachment]
  pending: [item: PendingAttachment]
  error: [message: string]
}>()

const { t } = useI18n()

function filenameOf(path: string) {
  return path.split(/[/\\]/).pop() ?? path
}

async function pick() {
  if (props.disabled || props.remaining <= 0) {
    emit('error', t('attachments.limitCount'))
    return
  }
  const selected = await open({
    multiple: true,
    filters: [{ name: 'Attachments', extensions: ATTACHMENT_EXTENSIONS }],
  })
  if (!selected)
    return
  const paths = Array.isArray(selected) ? selected : [selected]
  let slots = props.remaining
  for (const path of paths) {
    if (slots <= 0) {
      emit('error', t('attachments.limitCount'))
      break
    }
    const filename = filenameOf(path)
    if (!attachmentExt(filename)) {
      emit('error', t('attachments.unsupported'))
      continue
    }
    if (props.mode === 'pending') {
      emit('pending', { path, filename, sizeBytes: 0 })
      slots -= 1
      continue
    }
    if (!props.ownerType || !props.ownerId) {
      emit('error', t('attachments.addFailed'))
      continue
    }
    try {
      const item = await addAttachment(props.ownerType, props.ownerId, path)
      if (item.sizeBytes > MAX_ATTACHMENT_BYTES) {
        emit('error', t('attachments.limitSize'))
        continue
      }
      emit('added', item)
      slots -= 1
    }
    catch (e) {
      const message = e instanceof Error ? e.message : String(e)
      if (message.includes('10MB'))
        emit('error', t('attachments.limitSize'))
      else if (message.includes('limit is 5'))
        emit('error', t('attachments.limitCount'))
      else if (message.includes('unsupported'))
        emit('error', t('attachments.unsupported'))
      else
        emit('error', t('attachments.addFailed'))
    }
  }
}
</script>

<template>
  <button
    type="button"
    class="text-muted-foreground hover:bg-muted hover:text-foreground inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-xs disabled:opacity-50"
    :disabled="disabled || remaining <= 0"
    :title="t('attachments.add')"
    @click="pick"
  >
    <Paperclip class="size-3.5" />
    {{ t('attachments.add') }}
  </button>
</template>
