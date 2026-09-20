<script setup lang="ts">
import type { Component } from 'vue'
import type { MuseManageSection } from '@/components/muse/MuseManagePanel.vue'
import type { Language } from '@/lib/config'
import {
  FolderKanban,
  HardDrive,
  Info,
  Settings,
  Tags,
} from '@lucide/vue'
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import MuseManagePanel from '@/components/muse/MuseManagePanel.vue'
import MuseToaster from '@/components/muse/MuseToaster.vue'
import ThemeSwitch from '@/components/ThemeSwitch.vue'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { useMuseToast } from '@/composables/useMuseToast'
import { getLanguageLabel, supportedLanguages } from '@/lib/config'
import { restartApp } from '@/lib/muse'
import { resetSystem } from '@/lib/system'
import { cn } from '@/lib/utils'
import { useSettingsStore } from '@/stores/settings'

type SettingsSection = 'general' | 'backup' | 'tags' | 'projects' | 'about'

/** Keep in sync with package.json version */
const APP_VERSION = '0.0.2'

interface NavItem {
  id: SettingsSection
  icon: Component
  labelKey: string
}

interface NavGroup {
  labelKey: string
  items: NavItem[]
}

const { t, locale } = useI18n()
const availableLanguages = ref<Language[]>(supportedLanguages())
const settingsStore = useSettingsStore()
const { toast } = useMuseToast()
const activeSection = ref<SettingsSection>('general')
const resetOpen = ref(false)
const resetting = ref(false)

const navGroups = computed<NavGroup[]>(() => [
  {
    labelKey: 'settings.nav.groupSettings',
    items: [
      { id: 'general', icon: Settings, labelKey: 'settings.nav.general' },
      { id: 'backup', icon: HardDrive, labelKey: 'settings.nav.backup' },
    ],
  },
  {
    labelKey: 'settings.nav.groupData',
    items: [
      { id: 'tags', icon: Tags, labelKey: 'settings.nav.tags' },
      { id: 'projects', icon: FolderKanban, labelKey: 'settings.nav.projects' },
    ],
  },
  {
    labelKey: 'settings.nav.groupOther',
    items: [
      { id: 'about', icon: Info, labelKey: 'settings.nav.about' },
    ],
  },
])

const pageTitle = computed(() => {
  const map: Record<SettingsSection, string> = {
    general: 'settings.nav.general',
    backup: 'settings.nav.backup',
    tags: 'settings.nav.tags',
    projects: 'settings.nav.projects',
    about: 'settings.nav.about',
  }
  return t(map[activeSection.value])
})

const museSection = computed(() => {
  const section = activeSection.value
  if (section === 'tags' || section === 'projects' || section === 'backup')
    return section as MuseManageSection
  return null
})

watch(locale, (newLocale, oldLocale) => {
  if (newLocale && newLocale !== oldLocale)
    handleLanguageSelect(newLocale)
}, { immediate: true })

function handleLanguageSelect(newLocale: string) {
  if (!newLocale || !availableLanguages.value.some(sl => sl.value === newLocale))
    return
  settingsStore.setSetting<string>('language', newLocale)
}

async function confirmReset() {
  if (resetting.value)
    return
  resetting.value = true
  try {
    await settingsStore.clearSettings()
    localStorage.clear()
    sessionStorage.clear()
    await resetSystem()
    await restartApp()
  }
  catch (error) {
    resetting.value = false
    const message = error instanceof Error ? error.message : String(error)
    toast(message || t('settings.general.resetFailed'))
  }
}
</script>

<template>
  <div class="flex h-full min-h-0 overflow-hidden">
    <aside class="border-border bg-muted/30 flex w-[220px] shrink-0 flex-col gap-5 overflow-y-auto border-r px-3 py-5">
      <div
        v-for="group in navGroups"
        :key="group.labelKey"
        class="flex flex-col gap-1"
      >
        <p class="text-muted-foreground px-2 text-[11px] font-medium tracking-wide uppercase">
          {{ t(group.labelKey) }}
        </p>
        <button
          v-for="item in group.items"
          :key="item.id"
          type="button"
          :class="cn(
            'inline-flex items-center gap-2 rounded-lg px-2.5 py-2 text-sm transition-colors',
            activeSection === item.id
              ? 'bg-accent text-foreground font-medium'
              : 'text-muted-foreground hover:bg-accent/50 hover:text-foreground',
          )"
          @click="activeSection = item.id"
        >
          <component :is="item.icon" class="size-4 shrink-0" />
          {{ t(item.labelKey) }}
        </button>
      </div>
    </aside>

    <div class="flex min-w-0 flex-1 flex-col overflow-hidden">
      <header class="border-border shrink-0 border-b px-8 py-5">
        <h2 class="text-xl font-semibold tracking-tight">
          {{ pageTitle }}
        </h2>
      </header>

      <div class="min-h-0 flex-1 overflow-y-auto px-8 py-6">
        <!-- General：行式卡片，铺满右侧主栏 -->
        <div v-if="activeSection === 'general'" class="flex w-full flex-col gap-3">
          <div class="border-border bg-card flex items-center justify-between gap-6 rounded-xl border px-5 py-4">
            <div class="min-w-0">
              <Label class="text-sm font-medium" for="language-select">
                {{ t('settings.general.language') }}
              </Label>
              <p class="text-muted-foreground mt-0.5 text-xs">
                {{ t('settings.general.languageDesc') }}
              </p>
            </div>
            <Select id="language-select" v-model="locale">
              <SelectTrigger class="w-[160px] shrink-0">
                <SelectValue :placeholder="getLanguageLabel(locale)" />
              </SelectTrigger>
              <SelectContent>
                <SelectGroup>
                  <SelectItem
                    v-for="availableLanguage in availableLanguages"
                    :key="availableLanguage.value"
                    :value="availableLanguage.value"
                  >
                    {{ availableLanguage.label }}
                  </SelectItem>
                </SelectGroup>
              </SelectContent>
            </Select>
          </div>

          <div class="border-border bg-card flex items-center justify-between gap-6 rounded-xl border px-5 py-4">
            <div class="min-w-0">
              <Label class="text-sm font-medium">
                {{ t('settings.general.theme') }}
              </Label>
              <p class="text-muted-foreground mt-0.5 text-xs">
                {{ t('settings.general.themeDesc') }}
              </p>
            </div>
            <ThemeSwitch class="shrink-0" />
          </div>

          <div class="border-destructive/30 bg-card flex items-center justify-between gap-6 rounded-xl border px-5 py-4">
            <div class="min-w-0">
              <Label class="text-sm font-medium">
                {{ t('settings.general.reset') }}
              </Label>
              <p class="text-muted-foreground mt-0.5 text-xs">
                {{ t('settings.general.resetDesc') }}
              </p>
            </div>
            <button
              type="button"
              class="bg-destructive hover:bg-destructive/90 inline-flex h-8 shrink-0 items-center rounded-md px-3 text-sm text-white disabled:opacity-50"
              :disabled="resetting"
              @click="resetOpen = true"
            >
              {{ t('settings.general.resetAction') }}
            </button>
          </div>
        </div>

        <!-- Shared data: tags / projects / backup -->
        <div v-else-if="museSection" class="w-full">
          <MuseManagePanel :section="museSection" />
        </div>

        <!-- About -->
        <div v-else-if="activeSection === 'about'" class="w-full">
          <div class="border-border bg-card flex flex-col gap-2 rounded-xl border px-5 py-4">
            <h3 class="text-base font-semibold">
              {{ t('settings.about.title') }}
            </h3>
            <p class="text-muted-foreground text-sm">
              {{ t('settings.about.version', { version: APP_VERSION }) }}
            </p>
            <p class="text-muted-foreground text-xs">
              {{ t('settings.about.copyright') }}
            </p>
          </div>
        </div>
      </div>
    </div>
  </div>
  <ConfirmDialog
    v-model:open="resetOpen"
    :title="t('settings.general.resetTitle')"
    :description="t('settings.general.resetConfirm')"
    :confirm-label="t('settings.general.resetAction')"
    :cancel-label="t('settings.general.resetCancel')"
    @confirm="confirmReset"
  />
  <MuseToaster />
</template>
