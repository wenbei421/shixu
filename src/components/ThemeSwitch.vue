<script setup lang="ts">
import type { Component } from 'vue'
import { Monitor, Moon, Sun } from '@lucide/vue'
import { useColorMode } from '@vueuse/core'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { buttonVariants } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'
import { cn } from '@/lib/utils'
import { useSettingsStore } from '@/stores/settings'

type ThemeMode = 'light' | 'dark' | 'auto'

const { t } = useI18n()
const settingsStore = useSettingsStore()
const mode = useColorMode({ emitAuto: true })

const themeOptions: { value: ThemeMode, labelKey: string, icon: Component }[] = [
  { value: 'light', labelKey: 'settings.theme.light', icon: Sun },
  { value: 'dark', labelKey: 'settings.theme.dark', icon: Moon },
  { value: 'auto', labelKey: 'settings.theme.system', icon: Monitor },
]

const theme = computed({
  get: () => (mode.value === 'dark' || mode.value === 'light' ? mode.value : 'auto') as ThemeMode,
  set: (value: ThemeMode) => {
    mode.value = value
    void settingsStore.setSetting<string>('theme', value)
  },
})
</script>

<template>
  <RadioGroup
    v-model="theme"
    class="flex flex-row flex-wrap items-center gap-2"
    :aria-label="t('settings.theme.label')"
  >
    <div
      v-for="option in themeOptions"
      :key="option.value"
      class="relative"
    >
      <RadioGroupItem
        :id="`theme-${option.value}`"
        :value="option.value"
        class="peer sr-only"
      />
      <Label
        :for="`theme-${option.value}`"
        :class="cn(
          buttonVariants({
            variant: theme === option.value ? 'default' : 'outline',
            size: 'sm',
          }),
          'cursor-pointer gap-1.5',
        )"
      >
        <component :is="option.icon" />
        {{ t(option.labelKey) }}
      </Label>
    </div>
  </RadioGroup>
</template>
