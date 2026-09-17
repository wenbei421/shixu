<script setup lang="ts">
import { highlightSegments } from '@/lib/muse-format'
import { cn } from '@/lib/utils'
import { useMuseStore } from '@/stores/muse'
import MuseSourceStamp from './MuseSourceStamp.vue'
import MuseStatusBadge from './MuseStatusBadge.vue'

const store = useMuseStore()
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col gap-2 overflow-y-auto pt-3 pb-6">
    <article
      v-for="note in store.visibleNotes"
      :key="note.id"
      :class="cn(
        'bg-card cursor-pointer rounded-lg border px-3.5 py-3 transition-colors',
        note.id === store.selectedId
          ? 'border-primary ring-primary/15 ring-3'
          : 'border-border hover:border-muted-foreground/40',
      )"
      @click="store.select(note.id)"
    >
      <p class="text-foreground mb-2.5 line-clamp-3 text-[13.5px] leading-relaxed break-words">
        <template v-for="(segment, index) in highlightSegments(note.content, store.search)" :key="index">
          <mark v-if="segment.hit" class="bg-yellow-200 text-inherit dark:bg-yellow-500/40">{{ segment.text }}</mark>
          <template v-else>
            {{ segment.text }}
          </template>
        </template>
      </p>

      <div class="flex flex-wrap items-center gap-2">
        <span
          v-for="tag in note.tags"
          :key="tag"
          class="bg-primary/10 text-primary rounded px-1.5 py-0.5 text-[11px] font-medium"
        >
          #{{ tag }}
        </span>
        <span
          v-if="note.projectName"
          class="bg-muted text-muted-foreground rounded px-1.5 py-0.5 text-[11px] font-medium"
        >
          @{{ note.projectName }}
        </span>
        <MuseStatusBadge :status="note.status" />
        <MuseSourceStamp class="ml-auto" :source="note.source" :timestamp="note.createdAt" />
      </div>
    </article>
  </div>
</template>
