<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Badge } from '@/components/ui/badge'
import { Separator } from '@/components/ui/separator'
import { getDbStatus } from '@/lib/db'

const { t } = useI18n()
const clock = ref('--:--')
const fragmentCount = ref(0)
const dbOk = ref(false)
let timer: ReturnType<typeof setInterval> | undefined

function tick() {
  clock.value = new Date().toLocaleTimeString([], {
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
  })
}

async function refreshDbStatus() {
  try {
    const status = await getDbStatus()
    dbOk.value = status.ok
    fragmentCount.value = status.fragmentCount
  }
  catch {
    dbOk.value = false
  }
}

onMounted(() => {
  tick()
  void refreshDbStatus()
  timer = setInterval(tick, 30_000)
})

onUnmounted(() => {
  if (timer)
    clearInterval(timer)
})
</script>

<template>
  <footer class="border-border bg-background text-muted-foreground z-20 flex h-8 w-full shrink-0 items-center gap-2 border-t px-3 text-xs">
    <Badge variant="secondary" class="gap-1.5 font-normal">
      <span class="bg-chart-2 size-1.5 animate-pulse rounded-full" aria-hidden="true" />
      {{ t('statusbar.contextIdle') }}
    </Badge>

    <span class="hidden sm:inline">{{ t('statusbar.active') }} —</span>
    <span class="hidden md:inline">{{ t('statusbar.focus') }} —</span>

    <div class="ml-auto flex items-center gap-2">
      <span>{{ t('statusbar.fragments', { count: fragmentCount }) }}</span>
      <Separator orientation="vertical" class="h-4!" />
      <Badge variant="outline" class="gap-1.5 font-normal">
        <span
          class="size-1.5 rounded-full"
          :class="dbOk ? 'bg-chart-2' : 'bg-destructive'"
          aria-hidden="true"
        />
        {{ dbOk ? t('statusbar.sqliteReady') : t('statusbar.sqliteOffline') }}
      </Badge>
      <Separator orientation="vertical" class="h-4!" />
      <Badge variant="outline" class="gap-1.5 font-normal">
        <span class="bg-chart-2 size-1.5 rounded-full" aria-hidden="true" />
        {{ t('statusbar.aiIdle') }}
      </Badge>
      <Separator orientation="vertical" class="h-4!" />
      <span class="font-mono tabular-nums">{{ clock }}</span>
    </div>
  </footer>
</template>
