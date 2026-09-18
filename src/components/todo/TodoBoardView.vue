<script setup lang="ts">
import type { TodoStatus } from '@/lib/todo'
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { TODO_STATUS_ORDER } from '@/lib/todo'
import { cn } from '@/lib/utils'
import { useTodoStore } from '@/stores/todo'

const { t } = useI18n()
const store = useTodoStore()

const draggingId = ref<string | null>(null)
const dragOverStatus = ref<TodoStatus | null>(null)

function tasksOf(status: TodoStatus) {
  return store.tasks.filter(task => task.status === status)
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

function onDragOver(status: TodoStatus, event: DragEvent) {
  event.preventDefault()
  if (event.dataTransfer)
    event.dataTransfer.dropEffect = 'move'
  dragOverStatus.value = status
}

async function onDrop(status: TodoStatus, event: DragEvent) {
  event.preventDefault()
  const id = event.dataTransfer?.getData('text/plain') || draggingId.value
  onDragEnd()
  if (!id)
    return
  const task = store.tasks.find(item => item.id === id)
  if (!task || task.status === status)
    return
  await store.update(id, { status })
}
</script>

<template>
  <div class="grid min-h-0 flex-1 grid-cols-4 gap-2.5 p-3">
    <section
      v-for="status in TODO_STATUS_ORDER"
      :key="status"
      :class="cn(
        'bg-muted/50 flex min-h-0 flex-col rounded-lg border p-2 transition-colors',
        dragOverStatus === status ? 'border-primary bg-primary/5' : 'border-transparent',
      )"
      @dragover="onDragOver(status, $event)"
      @dragleave="dragOverStatus = null"
      @drop="onDrop(status, $event)"
    >
      <header class="mb-2 flex items-center justify-between px-1">
        <span class="text-xs font-medium">{{ t(`todo.status.${status}`) }}</span>
        <span class="text-muted-foreground text-[11px]">{{ tasksOf(status).length }}</span>
      </header>
      <div class="flex min-h-0 flex-1 flex-col gap-1.5 overflow-y-auto">
        <button
          v-for="task in tasksOf(status)"
          :key="task.id"
          type="button"
          draggable="true"
          :class="cn(
            'bg-background hover:border-border cursor-grab rounded-md border border-transparent px-2.5 py-2 text-left shadow-sm active:cursor-grabbing',
            draggingId === task.id && 'opacity-40',
            store.selectedId === task.id && 'border-primary',
          )"
          @click="store.select(task.id)"
          @dragstart="onDragStart(task.id, $event)"
          @dragend="onDragEnd"
        >
          <div class="line-clamp-2 text-sm">
            {{ task.title }}
          </div>
          <div class="text-muted-foreground mt-1 flex flex-wrap gap-1 text-[10px]">
            <span v-if="task.projectName">{{ task.projectName }}</span>
            <span v-for="tag in task.tags.slice(0, 2)" :key="tag">#{{ tag }}</span>
          </div>
        </button>
        <p
          v-if="!tasksOf(status).length"
          class="text-muted-foreground px-1 py-6 text-center text-[11px]"
        >
          {{ t('todo.empty') }}
        </p>
      </div>
    </section>
  </div>
</template>
