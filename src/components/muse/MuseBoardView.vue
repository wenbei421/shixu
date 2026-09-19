<script setup lang="ts">
import type { NoteStatus } from '@/lib/muse'
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import OutOfFilterHint from '@/components/OutOfFilterHint.vue'
import { useMuseToast } from '@/composables/useMuseToast'
import { STATUS_ORDER } from '@/lib/muse'
import { cn } from '@/lib/utils'
import { useMuseStore } from '@/stores/muse'
import MuseStatusBadge from './MuseStatusBadge.vue'

/** 看板卡片正文截断长度，与原型一致 */
const PREVIEW_LENGTH = 70

const { t } = useI18n()
const store = useMuseStore()
const { toast } = useMuseToast()

const draggingId = ref<string | null>(null)
const dragOverStatus = ref<NoteStatus | null>(null)

function notesOf(status: NoteStatus) {
  return store.visibleNotes.filter(note => note.status === status)
}

function preview(content: string) {
  return content.length > PREVIEW_LENGTH ? `${content.slice(0, PREVIEW_LENGTH)}…` : content
}

function onDragStart(id: string, event: DragEvent) {
  draggingId.value = id
  event.dataTransfer?.setData('text/plain', id)
  if (event.dataTransfer)
    event.dataTransfer.effectAllowed = 'move'
}

function onDragEnd() {
  draggingId.value = null
  dragOverStatus.value = null
}

function onDragOver(status: NoteStatus, event: DragEvent) {
  event.preventDefault()
  if (event.dataTransfer)
    event.dataTransfer.dropEffect = 'move'
  dragOverStatus.value = status
}

function onDragLeave(status: NoteStatus, event: DragEvent) {
  const current = event.currentTarget
  const next = event.relatedTarget
  if (current instanceof HTMLElement && next instanceof Node && current.contains(next))
    return
  if (dragOverStatus.value === status)
    dragOverStatus.value = null
}

async function onDrop(status: NoteStatus, event: DragEvent) {
  event.preventDefault()
  const id = event.dataTransfer?.getData('text/plain') || draggingId.value
  onDragEnd()
  if (!id)
    return

  const note = store.visibleNotes.find(item => item.id === id)
  // 拖回原列不触发更新（BR-03-02）
  if (!note || note.status === status)
    return

  try {
    await store.setStatus(id, status)
    toast(t('muse.toast.movedTo', { status: t(`muse.status.${status}`) }))
  }
  catch (e) {
    // 失败时列表已从数据库重拉，卡片自动回到原列（BR-03-03）
    toast(e instanceof Error ? e.message : String(e))
  }
}
</script>

<template>
  <div class="grid min-h-0 flex-1 grid-cols-4 gap-2.5 pt-3 pb-4">
    <section
      v-for="status in STATUS_ORDER"
      :key="status"
      :class="cn(
        'bg-muted/50 flex min-h-0 flex-col rounded-lg border p-2 transition-colors',
        dragOverStatus === status ? 'border-primary bg-primary/5' : 'border-transparent',
      )"
      @dragenter="onDragOver(status, $event)"
      @dragover="onDragOver(status, $event)"
      @dragleave="onDragLeave(status, $event)"
      @drop="onDrop(status, $event)"
    >
      <header class="flex items-center gap-2 px-1.5 pb-2.5">
        <MuseStatusBadge :status="status" />
        <span class="bg-background text-muted-foreground ml-auto rounded-full px-1.5 text-[11px] tabular-nums">
          {{ notesOf(status).length }}
        </span>
      </header>

      <div class="flex min-h-0 flex-1 flex-col gap-1.5 overflow-y-auto pb-1">
        <article
          v-for="note in notesOf(status)"
          :key="note.id"
          draggable="true"
          :class="cn(
            'bg-card cursor-grab rounded-md border border-border px-2.5 py-2 text-[12.5px] leading-relaxed transition-colors active:cursor-grabbing',
            draggingId === note.id && 'opacity-40',
            note.id === store.selectedId
              ? 'border-primary bg-accent'
              : 'hover:bg-accent/70',
          )"
          @click="store.select(note.id)"
          @dragstart="onDragStart(note.id, $event)"
          @dragend="onDragEnd"
          @dragover.prevent
        >
          <p class="break-words">
            {{ preview(note.content) }}
          </p>
          <div v-if="note.tags.length" class="mt-1.5 flex flex-wrap gap-1">
            <span
              v-for="tag in note.tags.slice(0, 3)"
              :key="tag"
              class="bg-primary/10 text-primary rounded px-1.5 text-[10px] font-medium"
            >
              #{{ tag }}
            </span>
          </div>
          <OutOfFilterHint v-if="store.retainedId === note.id" class="mt-1.5" />
        </article>
      </div>
    </section>
  </div>
</template>
