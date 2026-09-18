<script setup lang="ts">
import type { ViewMode } from '@/lib/muse'
import { Columns3, GitCommitVertical, LayoutList, Search, Shuffle, Trash2 } from '@lucide/vue'
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import { useMuseToast } from '@/composables/useMuseToast'
import { STATUS_ORDER } from '@/lib/muse'
import { CAPTURE_SHORTCUT, SEARCH_SHORTCUT } from '@/lib/shortcuts'
import { cn } from '@/lib/utils'
import { useMuseStore } from '@/stores/muse'
import MuseBoardView from './MuseBoardView.vue'
import MuseCardView from './MuseCardView.vue'
import MuseTimelineView from './MuseTimelineView.vue'

const { t } = useI18n()
const store = useMuseStore()
const { toast } = useMuseToast()
const searchInput = ref<HTMLInputElement | null>(null)
const confirmEmptyOpen = ref(false)

const viewOptions: { mode: ViewMode, icon: typeof LayoutList }[] = [
  { mode: 'card', icon: LayoutList },
  { mode: 'board', icon: Columns3 },
  { mode: 'timeline', icon: GitCommitVertical },
]

const title = computed(() => {
  const { kind, value } = store.filter
  if (kind === 'tag')
    return `#${value}`
  if (kind === 'project')
    return `@${value}`
  if (kind === 'review')
    return t('muse.list.reviewTitle')
  return t(`muse.nav.${kind}`)
})

function focusSearch() {
  searchInput.value?.focus()
  searchInput.value?.select()
}

async function confirmEmpty() {
  confirmEmptyOpen.value = false
  try {
    const count = await store.clearTrash()
    await store.refreshNotes()
    await store.refreshSidebar()
    toast(t('muse.manage.toast.emptied', { count }))
  }
  catch (e) {
    toast(e instanceof Error ? e.message : String(e))
  }
}

defineExpose({ focusSearch })
</script>

<template>
  <section class="flex min-h-0 min-w-0 flex-1 flex-col">
    <header class="flex-none">
      <div class="mb-3 flex items-center gap-2.5">
        <h2 class="text-base font-semibold tracking-tight">
          {{ title }}
        </h2>
        <span class="bg-muted text-muted-foreground rounded-full px-2 py-0.5 text-[11px] tabular-nums">
          {{ t('muse.list.count', { count: store.visibleNotes.length }) }}
        </span>
        <button
          v-if="store.filter.kind === 'review'"
          type="button"
          class="border-border text-muted-foreground hover:text-foreground flex items-center gap-1 rounded-md border px-2 py-1 text-[11px]"
          @click="store.reshuffle()"
        >
          <Shuffle class="size-3" />
          {{ t('muse.list.reshuffle') }}
        </button>
        <button
          v-if="store.isTrashView && store.visibleNotes.length"
          type="button"
          class="text-destructive hover:bg-destructive/10 ml-1 flex items-center gap-1 rounded-md px-2 py-1 text-[11px]"
          @click="confirmEmptyOpen = true"
        >
          <Trash2 class="size-3" />
          {{ t('muse.manage.trash.empty') }}
        </button>

        <div v-if="!store.isTrashView" class="bg-muted ml-auto flex gap-0.5 rounded-lg p-0.5">
          <button
            v-for="option in viewOptions"
            :key="option.mode"
            type="button"
            :title="t(`muse.list.view.${option.mode}`)"
            :aria-pressed="store.view === option.mode"
            :class="cn(
              'flex h-6 w-7 items-center justify-center rounded-md transition-colors',
              store.view === option.mode
                ? 'bg-background text-primary shadow-sm'
                : 'text-muted-foreground hover:text-foreground',
            )"
            @click="store.setView(option.mode)"
          >
            <component :is="option.icon" class="size-3.5" />
          </button>
        </div>
      </div>

      <div class="bg-muted/60 focus-within:border-primary focus-within:bg-background focus-within:ring-primary/15 mb-3 flex items-center gap-2 rounded-lg border border-transparent px-2.5 py-2 transition-colors focus-within:ring-3">
        <Search class="text-muted-foreground size-3.5 shrink-0" />
        <input
          ref="searchInput"
          :value="store.search"
          class="placeholder:text-muted-foreground min-w-0 flex-1 bg-transparent text-[12.5px] outline-none"
          :placeholder="t('muse.list.searchPlaceholder')"
          @input="store.setSearch(($event.target as HTMLInputElement).value)"
        >
        <kbd class="border-border text-muted-foreground shrink-0 rounded border px-1 font-sans text-[10px]">
          {{ SEARCH_SHORTCUT }}
        </kbd>
      </div>

      <div v-if="store.statusChipsVisible" class="flex flex-wrap gap-1.5 pb-1">
        <button
          v-for="status in (['all', ...STATUS_ORDER] as const)"
          :key="status"
          type="button"
          :class="cn(
            'rounded-full px-2.5 py-1 text-[11.5px] transition-colors',
            store.statusFilter === status
              ? 'bg-primary/10 text-primary font-semibold'
              : 'bg-muted text-muted-foreground hover:text-foreground',
          )"
          @click="store.setStatusFilter(status)"
        >
          {{ status === 'all' ? t('muse.statusFilter.all') : t(`muse.status.${status}`) }}
        </button>
      </div>
    </header>

    <div class="border-border flex min-h-0 flex-1 flex-col border-t">
      <p v-if="store.error" class="text-destructive pt-3 text-sm">
        {{ store.error }}
      </p>

      <div
        v-if="store.loading && !store.visibleNotes.length"
        class="text-muted-foreground flex flex-1 items-center justify-center text-sm"
      >
        {{ t('muse.list.loading') }}
      </div>

      <div
        v-else-if="!store.visibleNotes.length"
        class="text-muted-foreground flex flex-1 flex-col items-center justify-center gap-2 text-sm"
      >
        <component :is="store.isTrashView ? Trash2 : Search" class="size-9 opacity-30" />
        <p>
          {{ store.isTrashView
            ? t('muse.manage.trash.emptyState')
            : store.search
              ? t('muse.list.emptySearch')
              : t('muse.list.empty') }}
        </p>
        <p v-if="!store.isTrashView" class="text-[11.5px]">
          {{ t('muse.list.emptyHint', { shortcut: CAPTURE_SHORTCUT }) }}
        </p>
        <p v-else class="text-[11.5px]">
          {{ t('muse.manage.trash.hint') }}
        </p>
      </div>

      <MuseCardView v-else-if="store.isTrashView || store.view === 'card'" />
      <MuseBoardView v-else-if="store.view === 'board'" />
      <MuseTimelineView v-else />
    </div>
  </section>

  <ConfirmDialog
    v-model:open="confirmEmptyOpen"
    :title="t('muse.manage.trash.empty')"
    :description="t('muse.manage.trash.confirmEmpty')"
    :confirm-label="t('muse.manage.trash.empty')"
    :cancel-label="t('muse.detail.cancel')"
    @confirm="confirmEmpty"
  />
</template>
