<script setup lang="ts">
import type { Attachment, AttachmentOwnerType, PendingAttachment } from '@/lib/attachments'
import { Paperclip } from '@lucide/vue'
import { open } from '@tauri-apps/plugin-dialog'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  addAttachment,
  discardClipboardFile,
  MAX_ATTACHMENT_BYTES,
  readClipboardAttachments,
  shouldAttachClipboard,
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
const addLabel = computed(() => t(props.mode === 'pending' ? 'attachments.addCapture' : 'attachments.add'))

function filenameOf(path: string) {
  return path.split(/[/\\]/).pop() ?? path
}

async function acceptFile(file: PendingAttachment) {
  if (props.mode === 'pending') {
    emit('pending', file)
    return true
  }
  if (!props.ownerType || !props.ownerId) {
    emit('error', t('attachments.addFailed'))
    return false
  }
  try {
    const item = await addAttachment(props.ownerType, props.ownerId, file.path)
    if (item.sizeBytes > MAX_ATTACHMENT_BYTES) {
      emit('error', t('attachments.limitSize'))
      return false
    }
    if (file.ephemeral)
      await discardClipboardFile(file.path).catch(() => undefined)
    emit('added', item)
    return true
  }
  catch (e) {
    const message = e instanceof Error ? e.message : String(e)
    if (message.includes('10MB'))
      emit('error', t('attachments.limitSize'))
    else if (message.includes('limit is 5'))
      emit('error', t('attachments.limitCount'))
    else
      emit('error', t('attachments.addFailed'))
    return false
  }
}

async function acceptFiles(files: PendingAttachment[]) {
  if (props.disabled || props.remaining <= 0) {
    emit('error', t('attachments.limitCount'))
    for (const file of files) {
      if (file.ephemeral)
        await discardClipboardFile(file.path).catch(() => undefined)
    }
    return
  }
  let slots = props.remaining
  for (const file of files) {
    if (slots <= 0) {
      emit('error', t('attachments.limitCount'))
      if (file.ephemeral)
        await discardClipboardFile(file.path).catch(() => undefined)
      continue
    }
    if (await acceptFile(file))
      slots -= 1
  }
}

async function pick() {
  const selected = await open({
    multiple: true,
  })
  if (!selected)
    return
  const paths = Array.isArray(selected) ? selected : [selected]
  await acceptFiles(paths.map(path => ({
    path,
    filename: filenameOf(path),
    sizeBytes: 0,
  })))
}

async function pasteFromClipboard() {
  try {
    const files = await readClipboardAttachments()
    if (!files.length)
      return
    await acceptFiles(files.map(file => ({
      path: file.path,
      filename: file.filename,
      sizeBytes: 0,
      ephemeral: file.ephemeral,
    })))
  }
  catch (e) {
    emit('error', e instanceof Error ? e.message : String(e))
  }
}

function handlePaste(event: ClipboardEvent) {
  if (!shouldAttachClipboard(event))
    return
  event.preventDefault()
  window.setTimeout(() => {
    void pasteFromClipboard()
  }, 0)
}

defineExpose({ handlePaste })
</script>

<template>
  <button
    type="button"
    class="text-muted-foreground hover:bg-muted hover:text-foreground inline-flex size-7 items-center justify-center rounded-md disabled:opacity-50"
    :disabled="disabled || remaining <= 0"
    :title="addLabel"
    :aria-label="addLabel"
    @click="pick"
  >
    <Paperclip class="size-3.5" />
  </button>
</template>
