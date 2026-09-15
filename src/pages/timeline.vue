<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const { t } = useI18n({ useScope: 'global' })
</script>

<template>
  <div class="flex flex-col gap-2">
    <h1 class="text-2xl font-bold tracking-tight">
      {{ t('nav.timeline') }}
    </h1>
    <p class="text-muted-foreground text-sm">
      {{ t('pages.placeholder', { name: t('nav.timeline') }) }}
    </p>
  </div>
</template>
