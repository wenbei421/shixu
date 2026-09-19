<script setup lang="ts">
import type { NoteStatus } from '@/lib/muse'
import { useI18n } from 'vue-i18n'
import { STATUS_META } from '@/lib/muse'

const props = defineProps<{ status: NoteStatus }>()

const { t } = useI18n()
</script>

<template>
  <span
    class="inline-flex items-center gap-1.5 text-[11px] font-medium whitespace-nowrap"
    :style="{ color: STATUS_META[props.status].color }"
  >
    <span class="size-1.5 shrink-0 rounded-full bg-current" />
    {{ t(`muse.status.${props.status}`) }}
  </span>
</template>
