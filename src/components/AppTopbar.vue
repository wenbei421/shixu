<script setup lang="ts">
import { Search } from '@lucide/vue'
import { useI18n } from 'vue-i18n'
import {
  InputGroup,
  InputGroupAddon,
  InputGroupInput,
} from '@/components/ui/input-group'
import { Kbd } from '@/components/ui/kbd'
import { Separator } from '@/components/ui/separator'
import { SidebarTrigger } from '@/components/ui/sidebar'

const { t } = useI18n()
</script>

<template>
  <header
    class="border-border bg-background/80 z-20 flex h-10 w-full shrink-0 items-stretch gap-2 border-b px-3 backdrop-blur-md"
  >
    <div class="no-drag flex items-center gap-2">
      <SidebarTrigger />
      <Separator orientation="vertical" class="h-4!" />
    </div>

    <div class="flex min-w-0 flex-1 items-center gap-2" data-tauri-drag-region>
      <span class="hidden truncate text-sm font-semibold sm:inline">{{ t('brand.name') }}</span>
    </div>

    <div class="no-drag flex w-full max-w-md items-center">
      <InputGroup class="h-8!">
        <InputGroupAddon align="inline-start">
          <Search />
        </InputGroupAddon>
        <InputGroupInput
          type="search"
          :placeholder="t('topbar.searchPlaceholder')"
          spellcheck="false"
        />
        <InputGroupAddon align="inline-end" class="hidden sm:flex">
          <Kbd>Ctrl+K</Kbd>
        </InputGroupAddon>
      </InputGroup>
    </div>

    <div class="min-w-2 flex-1" data-tauri-drag-region />

    <!-- Decorum overlay window controls -->
    <div class="w-[138px] shrink-0" data-tauri-drag-region aria-hidden="true" />
  </header>
</template>
