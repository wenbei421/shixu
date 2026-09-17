<script setup lang="ts">
import type { NoteSource } from '@/lib/muse'
import { useI18n } from 'vue-i18n'
import { SOURCE_META } from '@/lib/muse'
import { relativeTime } from '@/lib/muse-format'

const props = withDefaults(defineProps<{
  source: NoteSource
  timestamp: number
  /** 紧凑模式只显示图标与时间，用于时间线 */
  compact?: boolean
}>(), { compact: false })

const { t, locale } = useI18n()
</script>

<template>
  <span class="text-muted-foreground flex items-center gap-1.5 text-[11px] whitespace-nowrap">
    <span
      class="inline-flex size-3.5 shrink-0 items-center justify-center rounded text-[8px] leading-none font-bold text-white"
      :style="{ background: SOURCE_META[props.source].color }"
      :title="t(`muse.source.${props.source}`)"
    >
      {{ SOURCE_META[props.source].short }}
    </span>
    <span v-if="!props.compact">{{ t(`muse.source.${props.source}`) }} ·</span>
    <span>{{ relativeTime(props.timestamp, locale) }}</span>
  </span>
</template>
