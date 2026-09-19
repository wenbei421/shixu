<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
</script>

<template>
  <span class="text-muted-foreground block text-[11px] leading-tight">
    {{ t('common.outOfFilter') }}
  </span>
</template>
