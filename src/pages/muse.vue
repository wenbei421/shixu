<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import MuseCaptureDialog from '@/components/muse/MuseCaptureDialog.vue'
import MuseDetailPane from '@/components/muse/MuseDetailPane.vue'
import MuseListPane from '@/components/muse/MuseListPane.vue'
import MuseSidebar from '@/components/muse/MuseSidebar.vue'
import MuseToaster from '@/components/muse/MuseToaster.vue'
import { useMuseToast } from '@/composables/useMuseToast'
import { STATUS_ORDER } from '@/lib/muse'
import {
  CAPTURE_SHORTCUT,
  isArchiveShortcut,
  isCaptureShortcut,
  isSearchShortcut,
  isTypingTarget,
  numberIndexFromKey,
} from '@/lib/shortcuts'
import { useMuseStore } from '@/stores/muse'

/** 首次进入后提示捕捉快捷键（原型 init） */
const WELCOME_DELAY = 700

const { t } = useI18n()
const store = useMuseStore()
const { toast } = useMuseToast()

const captureOpen = ref(false)
const listPane = ref<InstanceType<typeof MuseListPane> | null>(null)
const detailPane = ref<InstanceType<typeof MuseDetailPane> | null>(null)

/** ↑ / ↓ 在当前列表里移动选中项 */
function moveSelection(step: number) {
  const notes = store.visibleNotes
  if (!notes.length)
    return
  const current = notes.findIndex(note => note.id === store.selectedId)
  const next = Math.min(Math.max(current + step, 0), notes.length - 1)
  void store.select(notes[next]?.id ?? null)
}

async function onKeydown(event: KeyboardEvent) {
  if (isCaptureShortcut(event)) {
    event.preventDefault()
    captureOpen.value = !captureOpen.value
    return
  }

  if (isSearchShortcut(event)) {
    event.preventDefault()
    listPane.value?.focusSearch()
    return
  }

  // 浮层打开时，其余快捷键交给浮层自己处理
  if (captureOpen.value)
    return

  if (isArchiveShortcut(event) && store.selectedNote) {
    event.preventDefault()
    await detailPane.value?.toggleArchive()
    return
  }

  if (isTypingTarget(event.target))
    return

  if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
    event.preventDefault()
    moveSelection(event.key === 'ArrowDown' ? 1 : -1)
    return
  }

  if (event.key === 'Delete' && store.selectedNote) {
    event.preventDefault()
    detailPane.value?.requestDelete()
    return
  }

  const statusIndex = numberIndexFromKey(event, STATUS_ORDER.length)
  const status = statusIndex === null ? undefined : STATUS_ORDER[statusIndex]
  if (status && store.selectedNote) {
    event.preventDefault()
    await detailPane.value?.setStatus(status)
  }
}

onMounted(async () => {
  window.addEventListener('keydown', onKeydown)
  await store.refreshAll()
  setTimeout(() => toast(t('muse.toast.welcome', { shortcut: CAPTURE_SHORTCUT })), WELCOME_DELAY)
})

onUnmounted(() => {
  window.removeEventListener('keydown', onKeydown)
})
</script>

<template>
  <div class="flex h-full min-h-0 gap-4">
    <MuseSidebar @capture="captureOpen = true" />
    <MuseListPane ref="listPane" />
    <MuseDetailPane ref="detailPane" />
  </div>

  <MuseCaptureDialog :open="captureOpen" @close="captureOpen = false" />
  <MuseToaster />
</template>
