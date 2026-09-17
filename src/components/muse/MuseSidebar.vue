<script setup lang="ts">
import type { Component } from 'vue'
import type { NavKind } from '@/lib/muse'
import { Archive, Folder, Inbox, Layers, List, Plus, RotateCcw, Sun, Trash2 } from '@lucide/vue'
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { tagColor } from '@/lib/muse-format'
import { CAPTURE_SHORTCUT } from '@/lib/muse-shortcuts'
import { cn } from '@/lib/utils'
import { useMuseStore } from '@/stores/muse'

const emit = defineEmits<{ capture: [] }>()

/** 标签超过该数量时折叠（详细设计 5.5.2） */
const TAG_FOLD_LIMIT = 30

const { t } = useI18n()
const store = useMuseStore()
const tagsExpanded = ref(false)

interface NavEntry {
  kind: NavKind
  icon: Component
  count?: number
}

const navEntries = computed<NavEntry[]>(() => [
  { kind: 'inbox', icon: Inbox, count: store.counts.inbox },
  { kind: 'today', icon: Sun, count: store.counts.today },
  { kind: 'all', icon: Layers, count: store.counts.all },
  { kind: 'unsorted', icon: List, count: store.counts.unsorted },
  { kind: 'review', icon: RotateCcw },
  { kind: 'archive', icon: Archive, count: store.counts.archive },
  { kind: 'trash', icon: Trash2, count: store.counts.trash },
])

/** 左栏只列出有灵感的标签与项目，解绑后不留空条目（原型 getAllTags） */
const activeTags = computed(() => store.tags.filter(tag => tag.noteCount > 0))
const activeProjects = computed(() => store.projects.filter(project => project.noteCount > 0))

const visibleTags = computed(() =>
  tagsExpanded.value ? activeTags.value : activeTags.value.slice(0, TAG_FOLD_LIMIT),
)

const hiddenTagCount = computed(() => Math.max(activeTags.value.length - TAG_FOLD_LIMIT, 0))

function isActive(kind: NavKind, value: string | null = null) {
  return store.filter.kind === kind && store.filter.value === value
}

function itemClass(active: boolean) {
  return cn(
    'flex w-full items-center gap-2.5 rounded-md px-2 py-1.5 text-left text-[13px] transition-colors',
    active
      ? 'bg-primary/10 text-primary font-semibold'
      : 'text-muted-foreground hover:bg-accent hover:text-foreground',
  )
}
</script>

<template>
  <aside class="border-border flex w-[236px] shrink-0 flex-col overflow-y-auto border-r pr-3">
    <button
      type="button"
      class="bg-primary text-primary-foreground hover:bg-primary/90 mb-3 flex w-full items-center justify-center gap-2 rounded-lg px-3 py-2.5 text-[13px] font-semibold shadow-sm transition-colors"
      @click="emit('capture')"
    >
      <Plus class="size-4" />
      {{ t('muse.capture.button') }}
      <kbd class="ml-auto rounded bg-white/20 px-1.5 py-0.5 font-sans text-[10px] font-normal tracking-wide">
        {{ CAPTURE_SHORTCUT }}
      </kbd>
    </button>

    <nav class="flex flex-col gap-0.5">
      <button
        v-for="entry in navEntries"
        :key="entry.kind"
        type="button"
        :class="itemClass(isActive(entry.kind))"
        @click="store.setFilter(entry.kind)"
      >
        <component :is="entry.icon" class="size-4 shrink-0" />
        <span class="truncate">{{ t(`muse.nav.${entry.kind}`) }}</span>
        <span v-if="entry.count !== undefined" class="ml-auto text-[11px] tabular-nums opacity-70">
          {{ entry.count }}
        </span>
      </button>
    </nav>

    <p class="text-muted-foreground px-2 pt-4 pb-1.5 text-[10.5px] font-semibold tracking-widest uppercase">
      {{ t('muse.nav.tagsGroup') }}
    </p>
    <div v-if="!activeTags.length" class="text-muted-foreground px-2 py-1 text-xs">
      {{ t('muse.nav.noTags') }}
    </div>
    <div v-else class="flex flex-col gap-0.5">
      <button
        v-for="tag in visibleTags"
        :key="tag.id"
        type="button"
        :class="itemClass(isActive('tag', tag.name))"
        @click="store.setFilter('tag', tag.name)"
      >
        <span
          class="ml-1 size-[7px] shrink-0 rounded-full"
          :style="{ background: tag.color || tagColor(tag.name) }"
        />
        <span class="truncate">{{ tag.name }}</span>
        <span class="ml-auto text-[11px] tabular-nums opacity-70">{{ tag.noteCount }}</span>
      </button>
      <button
        v-if="hiddenTagCount && !tagsExpanded"
        type="button"
        class="text-muted-foreground hover:text-foreground px-2 py-1 text-left text-xs"
        @click="tagsExpanded = true"
      >
        {{ t('muse.nav.moreTags', { count: hiddenTagCount }) }}
      </button>
    </div>

    <p class="text-muted-foreground px-2 pt-4 pb-1.5 text-[10.5px] font-semibold tracking-widest uppercase">
      {{ t('muse.nav.projectsGroup') }}
    </p>
    <div v-if="!activeProjects.length" class="text-muted-foreground px-2 py-1 text-xs">
      {{ t('muse.nav.noProjects') }}
    </div>
    <div v-else class="flex flex-col gap-0.5 pb-2">
      <button
        v-for="project in activeProjects"
        :key="project.id"
        type="button"
        :class="itemClass(isActive('project', project.name))"
        @click="store.setFilter('project', project.name)"
      >
        <Folder class="size-4 shrink-0" />
        <span class="truncate">{{ project.name }}</span>
        <span class="ml-auto text-[11px] tabular-nums opacity-70">{{ project.noteCount }}</span>
      </button>
    </div>
  </aside>
</template>
