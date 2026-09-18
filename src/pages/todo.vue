<script setup lang="ts">
import type { TodoPerspective, TodoPriority, TodoStatus, TodoViewMode } from '@/lib/todo'
import {
  CalendarClock,
  CalendarDays,
  CalendarRange,
  CheckCircle2,
  CheckSquare2,
  Download,
  Inbox,
  Kanban,
  ListTodo,
  Search,
  X,
  Zap,
} from '@lucide/vue'
import { save } from '@tauri-apps/plugin-dialog'
import { writeTextFile } from '@tauri-apps/plugin-fs'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import TodoBoardView from '@/components/todo/TodoBoardView.vue'
import TodoCaptureDialog from '@/components/todo/TodoCaptureDialog.vue'
import TodoTimelineView from '@/components/todo/TodoTimelineView.vue'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Textarea } from '@/components/ui/textarea'
import {
  CAPTURE_SHORTCUT,
  isCaptureShortcut,
  isSearchShortcut,
  isTypingTarget,
  numberIndexFromKey,
} from '@/lib/shortcuts'
import { TODO_PRIORITY_FILTERS, TODO_STATUS_ORDER, TODO_VIEW_ORDER } from '@/lib/todo'
import { useTodoStore } from '@/stores/todo'

const { t } = useI18n()
const store = useTodoStore()
const subtaskDraft = ref('')
const tagDraft = ref('')
const searchDraft = ref('')
const captureOpen = ref(false)
const exportError = ref('')

const titleDraft = ref('')
const descriptionDraft = ref('')
const dueDraft = ref('')

const navItems: { id: TodoPerspective, icon: typeof Inbox }[] = [
  { id: 'inbox', icon: Inbox },
  { id: 'today', icon: CalendarDays },
  { id: 'upcoming', icon: CalendarRange },
  { id: 'all', icon: ListTodo },
  { id: 'done', icon: CheckCircle2 },
]

const priorities: TodoPriority[] = ['none', 'high', 'medium', 'low']
const viewIcons: Record<TodoViewMode, typeof ListTodo> = {
  card: ListTodo,
  board: Kanban,
  timeline: CalendarClock,
}

const projectSelectValue = computed({
  get: () => store.selected?.projectId ?? '__none__',
  set: (value: string) => {
    void onProjectChange(value)
  },
})

onMounted(() => {
  void store.refresh()
  window.addEventListener('keydown', onPageKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', onPageKeydown)
})

function onPageKeydown(event: KeyboardEvent) {
  if (isCaptureShortcut(event)) {
    event.preventDefault()
    captureOpen.value = !captureOpen.value
    return
  }

  if (captureOpen.value)
    return

  if (isSearchShortcut(event)) {
    event.preventDefault()
    const el = document.querySelector<HTMLInputElement>('[data-todo-search]')
    el?.focus()
    return
  }

  if (isTypingTarget(event.target))
    return

  const viewIndex = numberIndexFromKey(event, TODO_VIEW_ORDER.length)
  if (viewIndex != null) {
    event.preventDefault()
    store.setViewMode(TODO_VIEW_ORDER[viewIndex]!)
  }
}

watch(
  () => store.selectedId,
  () => syncDetailDrafts(),
  { immediate: true },
)

watch(
  () => store.selected?.updatedAt,
  () => syncDetailDrafts(),
)

function syncDetailDrafts() {
  const task = store.selected
  titleDraft.value = task?.title ?? ''
  descriptionDraft.value = task?.description ?? ''
  dueDraft.value = task?.dueAt ? toDateInput(task.dueAt) : ''
  tagDraft.value = ''
  subtaskDraft.value = ''
}

function toDateInput(ms: number) {
  const d = new Date(ms)
  const y = d.getFullYear()
  const m = String(d.getMonth() + 1).padStart(2, '0')
  const day = String(d.getDate()).padStart(2, '0')
  return `${y}-${m}-${day}`
}

function fromDateInput(value: string): number | null {
  if (!value)
    return null
  const [y, m, d] = value.split('-').map(Number)
  if (!y || !m || !d)
    return null
  return new Date(y, m - 1, d, 18, 0, 0, 0).getTime()
}

async function onCreateSubtask() {
  const task = store.selected
  const title = subtaskDraft.value.trim()
  if (!task || !title)
    return
  subtaskDraft.value = ''
  await store.create({ title, parentId: task.id })
}

async function onToggle(id: string, done: boolean) {
  await store.complete(id, done)
}

function onSearchInput(value: string | number) {
  searchDraft.value = String(value)
  store.setKeyword(searchDraft.value)
}

async function saveTitle() {
  const task = store.selected
  if (!task)
    return
  const title = titleDraft.value.trim()
  if (!title || title === task.title)
    return
  await store.update(task.id, { title })
}

async function saveDescription() {
  const task = store.selected
  if (!task)
    return
  if (descriptionDraft.value === task.description)
    return
  await store.update(task.id, { description: descriptionDraft.value })
}

async function onStatusChange(value: string | number | bigint | Record<string, any> | null) {
  const task = store.selected
  const status = String(value ?? '')
  if (!task || !status || status === task.status)
    return
  await store.update(task.id, { status: status as TodoStatus })
}

async function onPriorityChange(value: string | number | bigint | Record<string, any> | null) {
  const task = store.selected
  const priority = String(value ?? '')
  if (!task || !priority || priority === task.priority)
    return
  await store.update(task.id, { priority: priority as TodoPriority })
}

async function onDueChange() {
  const task = store.selected
  if (!task)
    return
  const next = fromDateInput(dueDraft.value)
  const prev = task.dueAt ?? null
  if (next === prev)
    return
  await store.update(task.id, { dueAt: next })
}

async function onProjectChange(value: string) {
  const task = store.selected
  if (!task)
    return
  const projectId = value === '__none__' ? null : value
  if ((task.projectId ?? null) === projectId)
    return
  await store.update(task.id, { projectId })
}

async function addTag() {
  const task = store.selected
  if (!task)
    return
  const name = tagDraft.value.trim().replace(/^#/, '')
  if (!name)
    return
  if (task.tags.includes(name)) {
    tagDraft.value = ''
    return
  }
  tagDraft.value = ''
  await store.update(task.id, { tagNames: [...task.tags, name] })
}

async function removeTag(name: string) {
  const task = store.selected
  if (!task)
    return
  await store.update(task.id, { tagNames: task.tags.filter(tag => tag !== name) })
}

async function onExport() {
  exportError.value = ''
  try {
    const data = await store.exportJson()
    const path = await save({
      defaultPath: `todo_export_${Date.now()}.json`,
      filters: [{ name: 'JSON', extensions: ['json'] }],
    })
    if (!path)
      return
    await writeTextFile(path, `${JSON.stringify(data, null, 2)}\n`)
  }
  catch (e) {
    exportError.value = e instanceof Error ? e.message : String(e)
  }
}
</script>

<template>
  <div class="flex h-full min-h-0 w-full overflow-hidden">
    <aside class="border-border flex w-52 shrink-0 flex-col gap-1 overflow-y-auto border-r p-3">
      <div class="text-muted-foreground mb-2 px-2 text-xs font-medium tracking-wide uppercase">
        {{ t('todo.title') }}
      </div>
      <button
        v-for="item in navItems"
        :key="item.id"
        type="button"
        class="hover:bg-muted flex items-center gap-2 rounded-md px-2 py-1.5 text-sm transition-colors"
        :class="store.perspective === item.id ? 'bg-muted text-foreground' : 'text-muted-foreground'"
        @click="store.setPerspective(item.id)"
      >
        <component :is="item.icon" class="size-4 shrink-0" />
        {{ t(`todo.nav.${item.id}`) }}
      </button>

      <div class="text-muted-foreground mt-4 mb-1 px-2 text-[10.5px] font-semibold tracking-widest uppercase">
        {{ t('todo.priorityFilter.group') }}
      </div>
      <button
        v-for="priority in TODO_PRIORITY_FILTERS"
        :key="priority"
        type="button"
        class="hover:bg-muted flex items-center gap-2 rounded-md px-2 py-1.5 text-sm transition-colors"
        :class="store.priorityFilter === priority ? 'bg-muted text-foreground' : 'text-muted-foreground'"
        @click="store.setPriorityFilter(priority)"
      >
        <span
          class="size-2 shrink-0 rounded-full"
          :class="{
            'bg-muted-foreground/40': priority === 'all' || priority === 'none',
            'bg-red-500': priority === 'high',
            'bg-amber-500': priority === 'medium',
            'bg-emerald-500': priority === 'low',
          }"
        />
        {{ priority === 'all' ? t('todo.priorityFilter.all') : t(`todo.priority.${priority}`) }}
      </button>
    </aside>

    <section class="border-border flex min-w-0 flex-1 flex-col border-r">
      <header class="flex items-center gap-2 border-b px-4 py-3">
        <CheckSquare2 class="text-muted-foreground size-4" />
        <h1 class="text-sm font-medium">
          {{ t(`todo.nav.${store.perspective}`) }}
        </h1>
        <span class="text-muted-foreground text-xs">{{ store.tasks.length }}</span>
        <div class="ml-auto flex items-center gap-1.5">
          <Button
            size="sm"
            class="h-7 gap-1 px-2.5"
            :title="`${t('todo.capture.title')} (${CAPTURE_SHORTCUT})`"
            @click="captureOpen = true"
          >
            <Zap class="size-3.5" />
            {{ t('todo.add') }}
          </Button>
          <Button
            size="sm"
            variant="ghost"
            class="h-7 px-2"
            :title="t('todo.export')"
            @click="onExport"
          >
            <Download class="size-3.5" />
          </Button>
          <Button
            v-for="mode in TODO_VIEW_ORDER"
            :key="mode"
            size="sm"
            variant="ghost"
            class="h-7 px-2"
            :class="store.viewMode === mode ? 'bg-muted' : ''"
            :title="t(`todo.view.${mode}`)"
            @click="store.setViewMode(mode)"
          >
            <component :is="viewIcons[mode]" class="size-3.5" />
          </Button>
        </div>
      </header>

      <div class="border-border flex items-center gap-2 border-b px-4 py-2">
        <div class="relative min-w-0 flex-1">
          <Search class="text-muted-foreground pointer-events-none absolute top-1/2 left-2 size-3.5 -translate-y-1/2" />
          <Input
            :model-value="searchDraft"
            data-todo-search
            class="h-8 pl-7"
            :placeholder="t('todo.searchPlaceholder')"
            @update:model-value="onSearchInput"
          />
        </div>
      </div>

      <TodoBoardView v-if="store.viewMode === 'board'" />
      <TodoTimelineView v-else-if="store.viewMode === 'timeline'" />

      <div v-else class="min-h-0 flex-1 overflow-y-auto p-2">
        <p v-if="store.loading" class="text-muted-foreground px-2 py-6 text-center text-sm">
          {{ t('todo.loading') }}
        </p>
        <p v-else-if="store.error" class="text-destructive px-2 py-6 text-center text-sm">
          {{ store.error }}
        </p>
        <p v-else-if="!store.tasks.length" class="text-muted-foreground px-2 py-10 text-center text-sm">
          <span class="mb-3 block">{{ t('todo.empty') }}</span>
          <Button size="sm" variant="outline" class="gap-1" @click="captureOpen = true">
            <Zap class="size-3.5" />
            {{ t('todo.add') }}
          </Button>
          <span class="text-muted-foreground mt-2 block text-[11px]">{{ CAPTURE_SHORTCUT }}</span>
        </p>
        <ul v-else class="flex flex-col gap-1">
          <li
            v-for="task in store.tasks"
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
                <span v-if="task.projectName">{{ task.projectName }}</span>
                <span v-for="tag in task.tags" :key="tag">#{{ tag }}</span>
              </div>
            </div>
          </li>
        </ul>
      </div>
    </section>

    <aside class="flex w-80 shrink-0 flex-col gap-3 overflow-y-auto p-4">
      <template v-if="store.selected">
        <div class="space-y-1.5">
          <Label>{{ t('todo.fields.title') }}</Label>
          <Input
            v-model="titleDraft"
            class="h-9"
            @keydown.enter.prevent="saveTitle"
            @blur="saveTitle"
          />
        </div>

        <div class="space-y-1.5">
          <Label>{{ t('todo.fields.description') }}</Label>
          <Textarea
            v-model="descriptionDraft"
            :placeholder="t('todo.noDescription')"
            class="min-h-24"
            @blur="saveDescription"
          />
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div class="space-y-1.5">
            <Label>{{ t('todo.fields.status') }}</Label>
            <Select :model-value="store.selected.status" @update:model-value="onStatusChange">
              <SelectTrigger class="h-9 w-full">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem v-for="status in TODO_STATUS_ORDER" :key="status" :value="status">
                  {{ t(`todo.status.${status}`) }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div class="space-y-1.5">
            <Label>{{ t('todo.fields.priority') }}</Label>
            <Select :model-value="store.selected.priority" @update:model-value="onPriorityChange">
              <SelectTrigger class="h-9 w-full">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem v-for="priority in priorities" :key="priority" :value="priority">
                  {{ t(`todo.priority.${priority}`) }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>

        <div class="space-y-1.5">
          <Label>{{ t('todo.fields.due') }}</Label>
          <Input
            v-model="dueDraft"
            type="date"
            class="h-9"
            @change="onDueChange"
          />
        </div>

        <div class="space-y-1.5">
          <Label>{{ t('todo.fields.project') }}</Label>
          <Select v-model="projectSelectValue">
            <SelectTrigger class="h-9 w-full">
              <SelectValue :placeholder="t('todo.noProject')" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="__none__">
                {{ t('todo.noProject') }}
              </SelectItem>
              <SelectItem
                v-for="project in store.projects"
                :key="project.id"
                :value="project.id"
              >
                {{ project.name }}
              </SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div class="space-y-1.5">
          <Label>{{ t('todo.fields.tags') }}</Label>
          <div class="flex flex-wrap gap-1.5">
            <button
              v-for="tag in store.selected.tags"
              :key="tag"
              type="button"
              class="bg-muted hover:bg-muted/80 inline-flex items-center gap-1 rounded-md px-2 py-0.5 text-xs"
              @click="removeTag(tag)"
            >
              #{{ tag }}
              <X class="size-3" />
            </button>
          </div>
          <Input
            v-model="tagDraft"
            class="h-8"
            :placeholder="t('todo.tagPlaceholder')"
            @keydown.enter.prevent="addTag"
          />
        </div>

        <div class="space-y-1.5">
          <div class="flex items-center justify-between">
            <Label>{{ t('todo.fields.subtasks') }}</Label>
            <span
              v-if="store.subtaskProgress.total"
              class="text-muted-foreground text-[11px]"
            >
              {{ store.subtaskProgress.done }}/{{ store.subtaskProgress.total }}
            </span>
          </div>
          <ul class="space-y-1">
            <li
              v-for="sub in store.subtasks"
              :key="sub.id"
              class="hover:bg-muted/50 flex items-center gap-2 rounded-md px-1 py-1"
            >
              <input
                type="checkbox"
                :checked="sub.status === 'done'"
                @change="onToggle(sub.id, ($event.target as HTMLInputElement).checked)"
              >
              <span
                class="min-w-0 flex-1 truncate text-sm"
                :class="sub.status === 'done' ? 'text-muted-foreground line-through' : ''"
              >
                {{ sub.title }}
              </span>
              <button
                type="button"
                class="text-muted-foreground hover:text-destructive p-0.5"
                :title="t('todo.delete')"
                @click="store.remove(sub.id)"
              >
                <X class="size-3.5" />
              </button>
            </li>
          </ul>
          <Input
            v-model="subtaskDraft"
            class="h-8"
            :placeholder="t('todo.subtaskPlaceholder')"
            @keydown.enter.prevent="onCreateSubtask"
          />
        </div>

        <Button
          variant="destructive"
          size="sm"
          class="mt-auto"
          @click="store.remove(store.selected.id)"
        >
          {{ t('todo.delete') }}
        </Button>
      </template>
      <p v-else class="text-muted-foreground text-sm">
        {{ t('todo.selectHint') }}
      </p>
    </aside>
  </div>

  <TodoCaptureDialog :open="captureOpen" @close="captureOpen = false" />
</template>
