<script setup lang="ts">
import type { DateValue } from '@internationalized/date'
import type { TodoPerspective, TodoPriority, TodoStatus, TodoViewMode } from '@/lib/todo'
import { CalendarDate, getLocalTimeZone, today } from '@internationalized/date'
import {
  CalendarClock,
  CalendarDays,
  CalendarIcon,
  CalendarRange,
  CheckCircle2,
  CheckSquare2,
  Columns3,
  GitCommitVertical,
  Inbox,
  LayoutList,
  ListTodo,
  Plus,
  Search,
  Trash2,
  X,
} from '@lucide/vue'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import AttachmentSection from '@/components/attachments/AttachmentSection.vue'
import OutOfFilterHint from '@/components/OutOfFilterHint.vue'
import TodoBoardView from '@/components/todo/TodoBoardView.vue'
import TodoCaptureDialog from '@/components/todo/TodoCaptureDialog.vue'
import TodoTimelineView from '@/components/todo/TodoTimelineView.vue'
import { Button } from '@/components/ui/button'
import { Calendar } from '@/components/ui/calendar'
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover'
import { toIntlLocale } from '@/lib/config'
import { formatDate } from '@/lib/datetime'
import { relativeTime } from '@/lib/muse-format'
import {
  CAPTURE_SHORTCUT,
  isCaptureShortcut,
  isSearchShortcut,
  isTypingTarget,
  numberIndexFromKey,
  SEARCH_SHORTCUT,
} from '@/lib/shortcuts'
import { TODO_PRIORITY_FILTERS, TODO_STATUS_ORDER, TODO_VIEW_ORDER } from '@/lib/todo'
import { formatDue, formatListTimestamp, isOverdue, priorityDotClass, priorityTextClass, statusDotClass, statusTextClass } from '@/lib/todo-format'
import { cn } from '@/lib/utils'
import { useTodoStore } from '@/stores/todo'

const { t, locale } = useI18n()
const store = useTodoStore()
const sectionLabelClass = 'text-muted-foreground text-[10.5px] font-semibold tracking-widest uppercase'
const contentClass = 'bg-card border-border placeholder:text-muted-foreground focus:border-primary focus:ring-primary/15 w-full rounded-lg border px-3.5 py-3 text-[13.5px] leading-relaxed outline-none transition-colors focus:ring-3'
const fieldClass = 'bg-card border-border text-muted-foreground focus:border-primary focus:ring-primary/15 w-full rounded-lg border px-2.5 py-1.5 text-[12.5px] outline-none transition-colors focus:ring-3'
const subtaskDraft = ref('')
const tagDraft = ref('')
const tagInputOpen = ref(false)
const searchDraft = ref('')
const captureOpen = ref(false)

const titleDraft = ref('')
const descriptionDraft = ref('')
const dueDraft = ref<DateValue>()
const dueOpen = ref(false)
const defaultPlaceholder = today(getLocalTimeZone())
const calendarLocale = computed(() => toIntlLocale(locale.value))

/** 日历上方的快捷日期：今天 / 明天 / 后天 / 下周 */
const dueQuickOptions = computed(() => {
  const base = today(getLocalTimeZone())
  return [
    { id: 'today' as const, value: base },
    { id: 'tomorrow' as const, value: base.add({ days: 1 }) },
    { id: 'dayAfter' as const, value: base.add({ days: 2 }) },
    { id: 'nextWeek' as const, value: base.add({ days: 7 }) },
  ]
})

function isSameDueDay(a: DateValue | undefined, b: DateValue) {
  return !!a && a.year === b.year && a.month === b.month && a.day === b.day
}

const navItems: { id: TodoPerspective, icon: typeof Inbox }[] = [
  { id: 'inbox', icon: Inbox },
  { id: 'today', icon: CalendarDays },
  { id: 'upcoming', icon: CalendarRange },
  { id: 'all', icon: ListTodo },
  { id: 'done', icon: CheckCircle2 },
]

const priorities: TodoPriority[] = ['none', 'high', 'medium', 'low']
const viewIcons: Record<TodoViewMode, typeof LayoutList> = {
  card: LayoutList,
  board: Columns3,
  timeline: GitCommitVertical,
}

const projectSelectValue = computed({
  get: () => store.selected?.projectId ?? '__none__',
  set: (value: string) => {
    void onProjectChange(value)
  },
})

/** 优先级组右栏数字：与当前视角联动（例如在「今天」下看各优先级有几条） */
function priorityCount(priority: TodoPriority | 'all') {
  return store.countsByPriority[priority]?.[store.perspective] ?? 0
}

onMounted(() => {
  void store.refreshProjects()
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
  () => {
    tagInputOpen.value = false
    tagDraft.value = ''
    syncDetailDrafts()
  },
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
  dueDraft.value = task?.dueAt ? msToDateValue(task.dueAt) : undefined
  subtaskDraft.value = ''
}

function msToDateValue(ms: number): DateValue {
  const d = new Date(ms)
  return new CalendarDate(d.getFullYear(), d.getMonth() + 1, d.getDate())
}

/** 与原先 native date 一致：选中日的本地 18:00 */
function dateValueToMs(value: DateValue | undefined): number | null {
  if (!value)
    return null
  return new Date(value.year, value.month - 1, value.day, 18, 0, 0, 0).getTime()
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

async function onDueChange(value: DateValue | undefined) {
  dueDraft.value = value
  dueOpen.value = false
  const task = store.selected
  if (!task)
    return
  const next = dateValueToMs(value)
  const prev = task.dueAt ?? null
  if (next === prev)
    return
  await store.update(task.id, { dueAt: next })
}

async function clearDue() {
  await onDueChange(undefined)
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

function openTagInput() {
  tagInputOpen.value = true
  tagDraft.value = ''
}

async function addTag() {
  const task = store.selected
  const name = tagDraft.value.trim().replace(/^#/, '')
  tagInputOpen.value = false
  tagDraft.value = ''
  if (!task || !name || task.tags.includes(name))
    return
  await store.update(task.id, { tagNames: [...task.tags, name] })
}

async function removeTag(name: string) {
  const task = store.selected
  if (!task)
    return
  await store.update(task.id, { tagNames: task.tags.filter(tag => tag !== name) })
}
</script>

<template>
  <div class="flex h-full min-h-0 gap-4">
    <aside class="border-border flex w-[236px] shrink-0 flex-col overflow-y-auto border-r pr-3">
      <button
        type="button"
        class="bg-primary text-primary-foreground hover:bg-primary/90 mb-3 flex w-full items-center justify-center gap-2 rounded-lg px-3 py-2.5 text-[13px] font-semibold shadow-sm transition-colors"
        :title="`${t('todo.capture.title')} (${CAPTURE_SHORTCUT})`"
        @click="captureOpen = true"
      >
        <Plus class="size-4" />
        {{ t('todo.capture.button') }}
        <kbd class="ml-auto rounded bg-white/20 px-1.5 py-0.5 font-sans text-[10px] font-normal tracking-wide">
          {{ CAPTURE_SHORTCUT }}
        </kbd>
      </button>
      <nav class="flex flex-col gap-0.5">
        <button
          v-for="item in navItems"
          :key="item.id"
          type="button"
          class="flex w-full items-center gap-2.5 rounded-md px-2 py-1.5 text-left text-[13px] transition-colors"
          :class="store.perspective === item.id
            ? 'bg-primary/10 text-primary font-semibold'
            : 'text-muted-foreground hover:bg-accent hover:text-foreground'"
          @click="store.setPerspective(item.id)"
        >
          <component :is="item.icon" class="size-4 shrink-0" />
          <span class="truncate">{{ t(`todo.nav.${item.id}`) }}</span>
          <span class="ml-auto text-[11px] tabular-nums opacity-70">
            {{ store.counts[item.id] }}
          </span>
        </button>
      </nav>

      <p class="text-muted-foreground px-2 pt-4 pb-1.5 text-[10.5px] font-semibold tracking-widest uppercase">
        {{ t('todo.priorityFilter.group') }}
      </p>
      <nav class="flex flex-col gap-0.5">
        <button
          v-for="priority in TODO_PRIORITY_FILTERS"
          :key="priority"
          type="button"
          class="flex w-full items-center gap-2.5 rounded-md px-2 py-1.5 text-left text-[13px] transition-colors"
          :class="store.priorityFilter === priority
            ? 'bg-primary/10 text-primary font-semibold'
            : 'text-muted-foreground hover:bg-accent hover:text-foreground'"
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
          <span class="truncate">{{ priority === 'all' ? t('todo.priorityFilter.all') : t(`todo.priority.${priority}`) }}</span>
          <span class="ml-auto text-[11px] tabular-nums opacity-70">
            {{ priorityCount(priority) }}
          </span>
        </button>
      </nav>
    </aside>

    <section class="flex min-h-0 min-w-0 flex-1 flex-col">
      <header class="flex-none">
        <div class="mb-3 flex items-center gap-2.5">
          <h2 class="text-base font-semibold tracking-tight">
            {{ t(`todo.nav.${store.perspective}`) }}
          </h2>
          <span class="bg-muted text-muted-foreground rounded-full px-2 py-0.5 text-[11px] tabular-nums">
            {{ t('todo.count', { count: store.tasks.length }) }}
          </span>
          <div class="bg-muted ml-auto flex gap-0.5 rounded-lg p-0.5">
            <button
              v-for="mode in TODO_VIEW_ORDER"
              :key="mode"
              type="button"
              :title="t(`todo.view.${mode}`)"
              :aria-pressed="store.viewMode === mode"
              :class="cn(
                'flex h-6 w-7 items-center justify-center rounded-md transition-colors',
                store.viewMode === mode
                  ? 'bg-background text-primary shadow-sm'
                  : 'text-muted-foreground hover:text-foreground',
              )"
              @click="store.setViewMode(mode)"
            >
              <component :is="viewIcons[mode]" class="size-3.5" />
            </button>
          </div>
        </div>

        <div class="bg-muted/60 focus-within:border-primary focus-within:bg-background focus-within:ring-primary/15 mb-3 flex items-center gap-2 rounded-lg border border-transparent px-2.5 py-2 transition-colors focus-within:ring-3">
          <Search class="text-muted-foreground size-3.5 shrink-0" />
          <input
            data-todo-search
            :value="searchDraft"
            class="placeholder:text-muted-foreground min-w-0 flex-1 bg-transparent text-[12.5px] outline-none"
            :placeholder="t('todo.searchPlaceholder')"
            @input="onSearchInput(($event.target as HTMLInputElement).value)"
          >
          <kbd class="border-border text-muted-foreground shrink-0 rounded border px-1 font-sans text-[10px]">
            {{ SEARCH_SHORTCUT }}
          </kbd>
        </div>
      </header>

      <div class="border-border flex min-h-0 flex-1 flex-col border-t">
        <p v-if="store.error" class="text-destructive pt-3 text-sm">
          {{ store.error }}
        </p>
        <div
          v-if="store.loading && !store.tasks.length"
          class="text-muted-foreground flex flex-1 items-center justify-center text-sm"
        >
          {{ t('todo.loading') }}
        </div>
        <div
          v-else-if="!store.tasks.length"
          class="text-muted-foreground flex flex-1 flex-col items-center justify-center gap-2 text-sm"
        >
          <Search class="size-9 opacity-30" />
          <p>{{ t('todo.empty') }}</p>
          <p class="text-[11.5px]">
            {{ t('todo.emptyHint', { shortcut: CAPTURE_SHORTCUT }) }}
          </p>
        </div>
        <TodoBoardView v-else-if="store.viewMode === 'board'" />
        <TodoTimelineView v-else-if="store.viewMode === 'timeline'" />
        <div v-else class="flex min-h-0 flex-1 flex-col gap-2 overflow-y-auto pt-3 pb-6">
          <article
            v-for="task in store.tasks"
            :key="task.id"
            :class="cn(
              'bg-card flex cursor-pointer items-start gap-2.5 rounded-lg border border-border px-3.5 py-3 transition-colors',
              store.selectedId === task.id
                ? 'border-primary bg-accent'
                : 'hover:bg-accent/70',
            )"
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
              <div class="flex items-start gap-2">
                <div
                  class="min-w-0 flex-1 truncate text-[13.5px] leading-relaxed"
                  :class="task.status === 'done' ? 'text-muted-foreground line-through' : ''"
                >
                  {{ task.title }}
                </div>
                <span
                  v-if="task.priority !== 'none'"
                  class="inline-flex shrink-0 items-center gap-1.5 text-[11px] font-medium whitespace-nowrap"
                  :class="priorityTextClass(task.priority)"
                  :title="t(`todo.priority.${task.priority}`)"
                >
                  <span
                    class="inline-block size-1.5 rounded-full"
                    :class="priorityDotClass(task.priority)"
                  />
                  {{ t(`todo.priority.${task.priority}`) }}
                </span>
              </div>
              <div class="mt-2.5 flex flex-wrap items-center gap-2">
                <span
                  v-if="task.dueAt"
                  class="inline-flex items-center gap-1 text-[11px] tabular-nums"
                  :class="isOverdue(task.dueAt, task.status) ? 'text-destructive' : 'text-muted-foreground'"
                >
                  <CalendarClock class="size-3" />
                  {{ formatDue(task.dueAt, locale) }}
                </span>
                <span
                  v-if="task.projectName"
                  class="bg-muted text-muted-foreground rounded px-1.5 py-0.5 text-[11px] font-medium"
                >
                  @{{ task.projectName }}
                </span>
                <span
                  v-for="tag in task.tags"
                  :key="tag"
                  class="bg-primary/10 text-primary rounded px-1.5 py-0.5 text-[11px] font-medium"
                >
                  #{{ tag }}
                </span>
                <OutOfFilterHint v-if="store.retainedId === task.id" />
                <span class="text-muted-foreground ml-auto text-[11px] whitespace-nowrap tabular-nums">
                  {{ formatListTimestamp(task, locale) }}
                </span>
              </div>
            </div>
          </article>
        </div>
      </div>
    </section>

    <aside class="border-border flex w-[344px] shrink-0 flex-col gap-3.5 overflow-y-auto border-l pl-4">
      <template v-if="store.selected">
        <div class="flex items-center gap-2">
          <span
            class="inline-flex items-center gap-1.5 text-[11px] font-medium whitespace-nowrap"
            :class="statusTextClass(store.selected.status)"
          >
            <span class="size-1.5 shrink-0 rounded-full bg-current" />
            {{ t(`todo.status.${store.selected.status}`) }}
          </span>
          <span class="text-muted-foreground ml-auto text-[11px]">
            {{ t('todo.createdAt', { time: relativeTime(store.selected.createdAt, locale) }) }}
          </span>
          <button
            type="button"
            class="text-muted-foreground hover:bg-destructive/10 hover:text-destructive flex size-6 items-center justify-center rounded-md transition-colors"
            :title="t('todo.delete')"
            :aria-label="t('todo.delete')"
            @click="store.remove(store.selected.id)"
          >
            <Trash2 class="size-3.5" />
          </button>
        </div>

        <p :class="sectionLabelClass">
          {{ t('todo.fields.title') }}
        </p>
        <input
          v-model="titleDraft"
          :class="cn(contentClass, 'font-medium')"
          @keydown.enter.prevent="saveTitle"
          @blur="saveTitle"
        >

        <p :class="sectionLabelClass">
          {{ t('todo.fields.description') }}
        </p>
        <textarea
          v-model="descriptionDraft"
          :placeholder="t('todo.noDescription')"
          :class="cn(contentClass, 'min-h-24 resize-y')"
          @blur="saveDescription"
        />

        <p :class="sectionLabelClass">
          {{ t('todo.fields.status') }}
        </p>
        <div class="grid grid-cols-2 gap-1.5">
          <button
            v-for="status in TODO_STATUS_ORDER"
            :key="status"
            type="button"
            :class="cn(
              'bg-card flex items-center gap-1.5 rounded-lg border px-2.5 py-1.5 text-left text-xs transition-colors',
              store.selected.status === status
                ? cn('border-current font-semibold', statusTextClass(status))
                : 'border-border text-muted-foreground hover:border-muted-foreground/40',
            )"
            @click="onStatusChange(status)"
          >
            <span
              class="size-[7px] shrink-0 rounded-full"
              :class="statusDotClass(status)"
            />
            {{ t(`todo.status.${status}`) }}
          </button>
        </div>
        <OutOfFilterHint v-if="store.retainedId === store.selected.id" />

        <p :class="sectionLabelClass">
          {{ t('todo.fields.priority') }}
        </p>
        <div class="grid grid-cols-2 gap-1.5">
          <button
            v-for="priority in priorities"
            :key="priority"
            type="button"
            :class="cn(
              'bg-card flex items-center gap-1.5 rounded-lg border px-2.5 py-1.5 text-left text-xs transition-colors',
              store.selected.priority === priority
                ? cn('border-current font-semibold', priorityTextClass(priority))
                : 'border-border text-muted-foreground hover:border-muted-foreground/40',
            )"
            @click="onPriorityChange(priority)"
          >
            <span
              class="size-[7px] shrink-0 rounded-full"
              :class="priorityDotClass(priority)"
            />
            {{ t(`todo.priority.${priority}`) }}
          </button>
        </div>

        <p :class="sectionLabelClass">
          {{ t('todo.fields.due') }}
        </p>
        <div class="group relative w-full">
          <Popover v-model:open="dueOpen">
            <PopoverTrigger as-child>
              <button
                type="button"
                :class="cn(
                  fieldClass,
                  'flex items-center gap-2 pr-8 text-left',
                  dueDraft ? 'text-foreground' : 'text-muted-foreground',
                )"
              >
                <CalendarIcon class="size-3.5 shrink-0" />
                {{ dueDraft
                  ? formatDate(dueDraft.toDate(getLocalTimeZone()))
                  : t('todo.timeline.noDue') }}
              </button>
            </PopoverTrigger>
            <PopoverContent class="w-auto p-0" align="start">
              <div class="border-border flex flex-wrap gap-1.5 border-b p-2">
                <Button
                  v-for="option in dueQuickOptions"
                  :key="option.id"
                  type="button"
                  size="sm"
                  :variant="isSameDueDay(dueDraft, option.value) ? 'default' : 'outline'"
                  class="h-7 px-2.5 text-xs"
                  @click="onDueChange(option.value)"
                >
                  {{ t(`todo.dueQuick.${option.id}`) }}
                </Button>
                <Button
                  v-if="dueDraft"
                  type="button"
                  size="sm"
                  variant="ghost"
                  class="text-muted-foreground h-7 px-2.5 text-xs"
                  @click="clearDue"
                >
                  {{ t('todo.dueQuick.clear') }}
                </Button>
              </div>
              <Calendar
                v-model="dueDraft"
                :locale="calendarLocale"
                :default-placeholder="defaultPlaceholder"
                layout="month-and-year"
                initial-focus
                @update:model-value="onDueChange"
              />
            </PopoverContent>
          </Popover>
          <button
            v-if="dueDraft"
            type="button"
            class="text-muted-foreground hover:text-foreground absolute top-1/2 right-1.5 flex size-6 -translate-y-1/2 items-center justify-center rounded-sm opacity-0 transition-opacity group-hover:opacity-100 focus-visible:opacity-100"
            :title="t('todo.dueQuick.clear')"
            :aria-label="t('todo.dueQuick.clear')"
            @click.stop.prevent="clearDue"
          >
            <X class="size-3.5" />
          </button>
        </div>

        <p :class="sectionLabelClass">
          {{ t('todo.fields.project') }}
        </p>
        <select
          v-model="projectSelectValue"
          :class="fieldClass"
        >
          <option value="__none__">
            {{ t('todo.noProject') }}
          </option>
          <option
            v-for="project in store.projects"
            :key="project.id"
            :value="project.id"
          >
            {{ project.name }}
          </option>
        </select>

        <p :class="sectionLabelClass">
          {{ t('todo.fields.tags') }}
        </p>
        <div class="flex flex-wrap items-center gap-1.5">
          <span
            v-for="tag in store.selected.tags"
            :key="tag"
            class="bg-primary/10 text-primary inline-flex items-center gap-1 rounded-md py-1 pr-1 pl-2.5 text-[11.5px] font-medium"
          >
            #{{ tag }}
            <button
              type="button"
              class="hover:bg-primary/20 flex size-4 items-center justify-center rounded opacity-60 transition-opacity hover:opacity-100"
              :title="t('todo.removeTag')"
              @click="removeTag(tag)"
            >
              <X class="size-3" />
            </button>
          </span>
          <input
            v-if="tagInputOpen"
            v-model="tagDraft"
            class="border-primary bg-background w-[88px] rounded-md border px-2 py-1 text-[11.5px] outline-none"
            :placeholder="t('todo.tagPlaceholder')"
            autofocus
            @blur="addTag"
            @keydown.enter.prevent="addTag"
            @keydown.esc.prevent="tagInputOpen = false"
          >
          <button
            v-else
            type="button"
            class="border-border text-muted-foreground hover:border-primary hover:text-primary rounded-md border border-dashed px-2.5 py-1 text-[11.5px] transition-colors"
            @click="openTagInput"
          >
            {{ t('todo.addTag') }}
          </button>
        </div>

        <AttachmentSection owner-type="todo" :owner-id="store.selected.id" />

        <div class="flex items-center justify-between gap-2">
          <p :class="sectionLabelClass">
            {{ t('todo.fields.subtasks') }}
          </p>
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
        <input
          v-model="subtaskDraft"
          :class="fieldClass"
          :placeholder="t('todo.subtaskPlaceholder')"
          @keydown.enter.prevent="onCreateSubtask"
        >
      </template>
      <div
        v-else
        class="text-muted-foreground flex flex-1 flex-col items-center justify-center gap-2.5 px-5 text-center text-[12.5px]"
      >
        <CheckSquare2 class="size-9 opacity-30" />
        <p>{{ t('todo.selectHint') }}</p>
      </div>
    </aside>
  </div>

  <TodoCaptureDialog :open="captureOpen" @close="captureOpen = false" />
</template>
