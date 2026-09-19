<script setup lang="ts">
import { useMuseToast } from '@/composables/useMuseToast'

const { toasts } = useMuseToast()
</script>

<template>
  <div class="pointer-events-none fixed bottom-14 left-1/2 z-[80] flex -translate-x-1/2 flex-col items-center gap-2">
    <div
      v-for="item in toasts"
      :key="item.id"
      class="bg-foreground text-background rounded-lg px-4 py-2 text-xs shadow-lg"
      role="status"
    >
      {{ item.message }}
    </div>
  </div>
</template>
