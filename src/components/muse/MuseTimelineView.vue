<script setup lang="ts">
import type { Note } from '@/lib/muse'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import OutOfFilterHint from '@/components/OutOfFilterHint.vue'
import { dateGroupKey, highlightSegments } from '@/lib/muse-format'
import { cn } from '@/lib/utils'
import { useMuseStore } from '@/stores/muse'
import MuseSourceStamp from './MuseSourceStamp.vue'
import MuseStatusBadge from './MuseStatusBadge.vue'

const { locale } = useI18n()
const store = useMuseStore()

/** 按「今天 / 昨天 / N 天前 / 年月日」分组（详细设计 5.3.4） */
const groups = computed(() => {
  const buckets = new Map<string, Note[]>()
  for (const note of store.visibleNotes) {
    const key = dateGroupKey(note.createdAt, locale.value)
    const bucket = buckets.get(key)
    if (bucket)
      bucket.push(note)
    else
      buckets.set(key, [note])
  }
  return [...buckets.entries()].map(([label, notes]) => ({ label, notes }))
})
</script>

<template>
  <div class="min-h-0 flex-1 overflow-y-auto pt-4 pb-7">
    <section
      v-for="group in groups"
      :key="group.label"
      class="border-border relative mb-5 ml-1.5 border-l pl-5 last:mb-0 last:border-transparent"
    >
      <h3 class="text-muted-foreground relative mb-2.5 text-[11.5px] font-semibold">
        <span class="bg-background border-primary absolute top-1 -left-[26px] size-2.5 rounded-full border-2" />
        {{ group.label }}
      </h3>

      <article
        v-for="note in group.notes"
        :key="note.id"
        :class="cn(
          'bg-card mb-1.5 cursor-pointer rounded-lg border px-3.5 py-2.5 text-[13px] leading-relaxed transition-colors',
          note.id === store.selectedId
            ? 'border-primary ring-primary/15 ring-3'
            : 'border-border hover:border-muted-foreground/40',
        )"
        @click="store.select(note.id)"
      >
        <p class="break-words">
          <template v-for="(segment, index) in highlightSegments(note.content, store.search)" :key="index">
            <mark v-if="segment.hit" class="bg-yellow-200 text-inherit dark:bg-yellow-500/40">{{ segment.text }}</mark>
            <template v-else>
              {{ segment.text }}
            </template>
          </template>
        </p>

        <div class="mt-2 flex flex-wrap items-center gap-2">
          <MuseStatusBadge :status="note.status" />
          <OutOfFilterHint v-if="store.retainedId === note.id" />
          <span
            v-for="tag in note.tags"
            :key="tag"
            class="bg-primary/10 text-primary rounded px-1.5 py-0.5 text-[11px] font-medium"
          >
            #{{ tag }}
          </span>
          <MuseSourceStamp class="ml-auto" :source="note.source" :timestamp="note.createdAt" compact />
        </div>
      </article>
    </section>
  </div>
</template>
