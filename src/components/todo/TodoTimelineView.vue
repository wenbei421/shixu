<script setup lang="ts">
import type { TodoTask } from '@/lib/todo'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { cn } from '@/lib/utils'
import { useTodoStore } from '@/stores/todo'

interface TimelineGroup {
  key: string
  label: string
  tone: 'overdue' | 'today' | 'normal' | 'none'
  tasks: TodoTask[]
}

function startOfDay(ms = Date.now()) {
  const d = new Date(ms)
  return new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime()
}

function addDays(dayMs: number, days: number) {
  const d = new Date(dayMs)
  d.setDate(d.getDate() + days)
  return d.getTime()
}

function dayKey(ms: number) {
  const d = new Date(ms)
  const y = d.getFullYear()
  const m = String(d.getMonth() + 1).padStart(2, '0')
  const day = String(d.getDate()).padStart(2, '0')
  return `${y}-${m}-${day}`
}

const { t } = useI18n()
const store = useTodoStore()

const groups = computed<TimelineGroup[]>(() => {
  const today = startOfDay()
  const tomorrow = addDays(today, 1)
  const buckets = new Map<string, TimelineGroup>()

  const ensure = (key: string, label: string, tone: TimelineGroup['tone']) => {
    let group = buckets.get(key)
    if (!group) {
      group = { key, label, tone, tasks: [] }
      buckets.set(key, group)
    }
    return group
  }

  for (const task of store.tasks) {
    if (task.status === 'done' || task.status === 'cancelled') {
      if (store.perspective !== 'done')
        continue
    }

    if (task.dueAt == null) {
      ensure('none', t('todo.timeline.noDue'), 'none').tasks.push(task)
      continue
    }

    const dueDay = startOfDay(task.dueAt)
    if (dueDay < today && task.status !== 'done') {
      ensure('overdue', t('todo.timeline.overdue'), 'overdue').tasks.push(task)
      continue
    }
    if (dueDay === today) {
      ensure('today', t('todo.timeline.today'), 'today').tasks.push(task)
      continue
    }
    if (dueDay === tomorrow) {
      ensure('tomorrow', t('todo.timeline.tomorrow'), 'normal').tasks.push(task)
      continue
    }

    const key = dayKey(dueDay)
    const label = new Date(dueDay).toLocaleDateString(undefined, {
      month: 'short',
      day: 'numeric',
      weekday: 'short',
    })
    ensure(key, label, 'normal').tasks.push(task)
  }

  const order = ['overdue', 'today', 'tomorrow']
  return [...buckets.values()].sort((a, b) => {
    if (a.key === 'none')
      return 1
    if (b.key === 'none')
      return -1
    const ai = order.indexOf(a.key)
    const bi = order.indexOf(b.key)
    if (ai !== -1 || bi !== -1)
      return (ai === -1 ? 99 : ai) - (bi === -1 ? 99 : bi)
    return a.key.localeCompare(b.key)
  })
})

function onToggle(id: string, done: boolean) {
  void store.complete(id, done)
}
</script>

<template>
  <div class="min-h-0 flex-1 overflow-y-auto p-3">
    <p v-if="!groups.length" class="text-muted-foreground py-10 text-center text-sm">
      {{ t('todo.empty') }}
    </p>
    <section v-for="group in groups" :key="group.key" class="mb-4">
      <header
        :class="cn(
          'mb-2 flex items-center gap-2 px-1 text-xs font-medium',
          group.tone === 'overdue' && 'text-destructive',
          group.tone === 'today' && 'text-primary',
          group.tone === 'none' && 'text-muted-foreground',
        )"
      >
        <span>{{ group.label }}</span>
        <span class="text-muted-foreground font-normal">{{ group.tasks.length }}</span>
      </header>
      <ul class="flex flex-col gap-1">
        <li
          v-for="task in group.tasks"
          :key="task.id"
          class="hover:bg-muted/60 flex cursor-pointer items-start gap-2 rounded-md px-2 py-2"
          :class="store.selectedId === task.id ? 'bg-muted' : ''"
          @click="store.select(task.id)"
        >
          <input
            type="checkbox"
            class="mt-1"
            :checked="task.status === 'done'"
            @click.stop
            @change="onToggle(task.id, ($event.target as HTMLInputElement).checked)"
          >
          <div class="min-w-0 flex-1">
            <div
              class="truncate text-sm"
              :class="task.status === 'done' ? 'text-muted-foreground line-through' : ''"
            >
              {{ task.title }}
            </div>
            <div class="text-muted-foreground mt-0.5 flex flex-wrap gap-2 text-[11px]">
              <span
                v-if="task.dueAt"
                :class="group.tone === 'overdue' ? 'text-destructive' : ''"
              >
                {{ new Date(task.dueAt).toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' }) }}
              </span>
              <span v-if="task.projectName">{{ task.projectName }}</span>
              <span v-for="tag in task.tags" :key="tag">#{{ tag }}</span>
            </div>
          </div>
        </li>
      </ul>
    </section>
  </div>
</template>
