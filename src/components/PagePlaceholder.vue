<script setup lang="ts">
defineProps<{
  title: string
  description: string
}>()
</script>

<template>
  <div class="flex max-w-3xl flex-col gap-2">
    <h1 class="text-2xl font-bold tracking-tight">
      {{ title }}
    </h1>
    <p class="text-muted-foreground text-sm">
      {{ description }}
    </p>
  </div>
</template>
