<script setup lang="ts">
import type { Component } from 'vue'
import type { Note } from '@/lib/muse'
import type { TodoTask } from '@/lib/todo'
import { CalendarCheck, CheckSquare2, Clock3, Sparkles } from '@lucide/vue'
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { toIntlLocale } from '@/lib/config'
import { getDbStatus } from '@/lib/db'
import { getNoteCounts, listNotes } from '@/lib/muse'
import { getTodoCounts, listTodoTasks } from '@/lib/todo'

const { t, locale } = useI18n({ useScope: 'global' })

const WEEKS = 16
const DAY_MS = 24 * 60 * 60 * 1000
const heatmapCells = Array.from({ length: WEEKS * 7 }, (_, index) => index)
const emptyBars = [0, 0, 0, 0, 0, 0, 0]

const todoToday = ref<number | null>(null)
const museToday = ref<number | null>(null)
const fragmentTotal = ref<number | null>(null)
const planRatio = ref<{ done: number, total: number } | null>(null)
const todoBars = ref<number[]>([...emptyBars])
const museBars = ref<number[]>([...emptyBars])

const heading = computed(() => {
  return new Intl.DateTimeFormat(toIntlLocale(locale.value), {
    month: 'long',
    day: 'numeric',
    weekday: 'long',
  }).format(new Date())
})

const summary = computed(() => {
  if (todoToday.value === null || museToday.value === null)
    return t('home.summary')
  return t('home.summaryLive', { todo: todoToday.value, notes: museToday.value })
})

/** 热力纵轴：周一到周日，隔行标字，避免七个字挤在一起 */
const dayLabels = computed(() => {
  const format = new Intl.DateTimeFormat(toIntlLocale(locale.value), { weekday: 'narrow' })
  return Array.from({ length: 7 }, (_, index) => {
    if (index % 2 === 1)
      return ''
    return format.format(new Date(2024, 0, 1 + index))
  })
})

const cards = computed(() => {
  const plan = planRatio.value
  const planText = plan && plan.total > 0
    ? `${Math.round(plan.done / plan.total * 100)}%`
    : t('home.emptyValue')
  return [
    {
      key: 'fragments',
      icon: Clock3,
      titleKey: 'home.cards.fragments',
      value: fragmentTotal.value === null ? t('home.emptyValue') : String(fragmentTotal.value),
      known: fragmentTotal.value !== null,
      hint: fragmentTotal.value === null
        ? t('home.cards.fragmentsHint')
        : t('home.cards.fragmentsHintLive', { count: fragmentTotal.value }),
      bars: emptyBars,
    },
    {
      key: 'todo',
      icon: CheckSquare2,
      titleKey: 'home.cards.todo',
      value: todoToday.value === null ? t('home.emptyValue') : String(todoToday.value),
      known: todoToday.value !== null,
      hint: todoToday.value === null ? t('home.cards.todoHint') : t('home.cards.todoHintLive'),
      bars: todoBars.value,
    },
    {
      key: 'muse',
      icon: Sparkles,
      titleKey: 'home.cards.muse',
      value: museToday.value === null ? t('home.emptyValue') : String(museToday.value),
      known: museToday.value !== null,
      hint: museToday.value === null ? t('home.cards.museHint') : t('home.cards.museHintLive'),
      bars: museBars.value,
    },
    {
      key: 'plan',
      icon: CalendarCheck,
      titleKey: 'home.cards.plan',
      value: planText,
      known: plan !== null && plan.total > 0,
      hint: plan === null
        ? t('home.cards.planHint')
        : plan.total === 0
          ? t('home.cards.planHintNone')
          : t('home.cards.planHintLive', { done: plan.done, total: plan.total }),
      bars: emptyBars,
    },
  ] satisfies Array<{
    key: string
    icon: Component
    titleKey: string
    value: string
    known: boolean
    hint: string
    bars: number[]
  }>
})

function startOfDay(ms: number) {
  const date = new Date(ms)
  return new Date(date.getFullYear(), date.getMonth(), date.getDate()).getTime()
}

/** 含今天在内的 7 个本地日，从左到右变新 */
function recentDays() {
  const today = startOfDay(Date.now())
  return Array.from({ length: 7 }, (_, index) => today - (6 - index) * DAY_MS)
}

function bucket(timestamps: number[], days: number[]) {
  const counts = days.map(() => 0)
  for (const ts of timestamps) {
    const index = days.indexOf(startOfDay(ts))
    if (index >= 0)
      counts[index] += 1
  }
  return counts
}

function barsHaveData(bars: number[]) {
  return bars.some(count => count > 0)
}

function barHeight(count: number, bars: number[]) {
  const max = Math.max(...bars)
  if (max <= 0)
    return '6px'
  if (count <= 0)
    return '2px'
  return `${Math.max(6, Math.round(count / max * 28))}px`
}

onMounted(async () => {
  const days = recentDays()
  const todayStart = days[6]!
  const todayEnd = todayStart + DAY_MS

  const [todoCounts, noteCounts, notes, openTasks, doneTasks, status] = await Promise.all([
    getTodoCounts().catch(() => null),
    getNoteCounts().catch(() => null),
    listNotes({ kind: 'all' }).catch(() => null),
    listTodoTasks({ perspective: 'all' }).catch(() => null),
    listTodoTasks({ perspective: 'done' }).catch(() => null),
    getDbStatus().catch(() => null),
  ])

  if (todoCounts)
    todoToday.value = todoCounts.today
  if (noteCounts)
    museToday.value = noteCounts.today
  if (status)
    fragmentTotal.value = status.fragmentCount
  if (notes)
    museBars.value = bucket(notes.map((note: Note) => note.createdAt), days)
  if (openTasks && doneTasks)
    fillTodoTrend(openTasks, doneTasks, days, todayStart, todayEnd)
})

function fillTodoTrend(openTasks: TodoTask[], doneTasks: TodoTask[], days: number[], todayStart: number, todayEnd: number) {
  const tasks = [...openTasks, ...doneTasks].filter(task => task.dueAt != null && task.status !== 'cancelled')
  todoBars.value = bucket(tasks.map(task => task.dueAt!), days)
  const dueToday = tasks.filter(task => task.dueAt! >= todayStart && task.dueAt! < todayEnd)
  planRatio.value = {
    done: dueToday.filter(task => task.status === 'done').length,
    total: dueToday.length,
  }
}
</script>

<template>
  <div class="flex h-full min-h-0 flex-col gap-4 overflow-auto">
    <header>
      <h1 class="text-[22px] font-semibold tracking-tight">
        {{ heading }}
      </h1>
      <p class="text-muted-foreground mt-1.5 text-[13px]">
        {{ summary }}
      </p>
    </header>

    <section class="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-4" :aria-label="t('home.statsLabel')">
      <article
        v-for="card in cards"
        :key="card.key"
        class="border-border bg-card rounded-[10px] border px-3.5 pt-3.5 pb-3"
      >
        <div class="text-muted-foreground flex items-center gap-2 text-xs">
          <span class="bg-muted grid size-6 place-items-center rounded-md">
            <component :is="card.icon" class="size-3.5" />
          </span>
          {{ t(card.titleKey) }}
        </div>
        <p
          class="mt-2.5 text-[26px] leading-none font-semibold tracking-tight tabular-nums"
          :class="card.known ? 'text-foreground' : 'text-muted-foreground'"
        >
          {{ card.value }}
        </p>
        <p class="text-muted-foreground mt-1.5 text-xs">
          {{ card.hint }}
        </p>
        <div class="mt-3.5 flex h-7 items-end gap-1" aria-hidden="true">
          <i
            v-for="(count, index) in card.bars"
            :key="index"
            class="flex-1 rounded-xs"
            :class="barsHaveData(card.bars) ? 'bg-foreground/80' : 'bg-muted'"
            :style="{ height: barHeight(count, card.bars) }"
          />
        </div>
      </article>
    </section>

    <section class="border-border bg-card rounded-[10px] border px-3.5 py-3.5" aria-labelledby="home-heatmap-title">
      <div class="mb-3.5 flex items-baseline justify-between gap-3">
        <h2 id="home-heatmap-title" class="text-[13px] font-semibold">
          {{ t('home.heatmap.title') }}
        </h2>
        <span class="text-muted-foreground text-xs">{{ t('home.heatmap.range') }}</span>
      </div>
      <div class="flex items-center gap-2 overflow-x-auto">
        <div class="text-muted-foreground grid shrink-0 grid-rows-7 gap-[3px] text-[10px] leading-3">
          <span v-for="(label, index) in dayLabels" :key="index" class="h-3">{{ label }}</span>
        </div>
        <div class="grid grid-flow-col grid-rows-7 gap-[3px]" :style="{ gridAutoColumns: '12px' }">
          <i
            v-for="cell in heatmapCells"
            :key="cell"
            class="bg-muted size-3 rounded-[2px]"
          />
        </div>
      </div>
      <div class="mt-3 flex items-center justify-between gap-3">
        <span class="text-muted-foreground text-xs">{{ t('home.heatmap.empty') }}</span>
        <div class="text-muted-foreground flex items-center gap-1 text-xs" aria-hidden="true">
          <span>{{ t('home.heatmap.less') }}</span>
          <i v-for="step in 4" :key="step" class="bg-muted size-3 rounded-[2px]" />
          <span>{{ t('home.heatmap.more') }}</span>
        </div>
      </div>
    </section>

    <section class="grid grid-cols-1 gap-3 lg:grid-cols-2">
      <article class="border-border bg-card rounded-[10px] border px-3.5 py-3.5">
        <div class="mb-3.5 flex items-baseline justify-between gap-3">
          <h2 class="text-[13px] font-semibold">
            {{ t('home.allocation.title') }}
          </h2>
          <span class="text-muted-foreground text-xs">{{ t('home.allocation.source') }}</span>
        </div>
        <div class="bg-muted/45 flex min-h-44 items-center justify-center rounded-lg border border-dashed px-6 py-6 text-center">
          <div>
            <div class="border-muted mx-auto mb-3 size-24 rounded-full border-[10px]" aria-hidden="true" />
            <p class="text-sm font-semibold">
              {{ t('home.allocation.emptyTitle') }}
            </p>
            <p class="text-muted-foreground mt-1 text-xs">
              {{ t('home.allocation.emptyBody') }}
            </p>
          </div>
        </div>
      </article>
      <article class="border-border bg-card rounded-[10px] border px-3.5 py-3.5">
        <div class="mb-3.5 flex items-baseline justify-between gap-3">
          <h2 class="text-[13px] font-semibold">
            {{ t('home.apps.title') }}
          </h2>
          <span class="text-muted-foreground text-xs">{{ t('home.apps.source') }}</span>
        </div>
        <div class="bg-muted/45 flex min-h-44 items-center justify-center rounded-lg border border-dashed px-6 py-6 text-center">
          <div>
            <p class="text-sm font-semibold">
              {{ t('home.apps.emptyTitle') }}
            </p>
            <p class="text-muted-foreground mt-1 text-xs">
              {{ t('home.apps.emptyBody') }}
            </p>
          </div>
        </div>
      </article>
    </section>
  </div>
</template>
