<script setup lang="ts">
import { ConfigProvider } from 'reka-ui'
import AppSidebar from '@/components/AppSidebar.vue'
import AppStatusbar from '@/components/AppStatusbar.vue'
import AppTopbar from '@/components/AppTopbar.vue'
import { SidebarInset, SidebarProvider } from '@/components/ui/sidebar'
</script>

<template>
  <ConfigProvider>
    <SidebarProvider class="flex h-dvh! min-h-0! flex-col overflow-hidden">
      <AppTopbar />

      <div class="flex min-h-0 min-w-0 flex-1 overflow-hidden">
        <AppSidebar />
        <SidebarInset class="min-h-0 min-w-0 overflow-hidden">
          <div class="flex min-h-0 flex-1 flex-col overflow-hidden p-6">
            <router-view />
          </div>
        </SidebarInset>
      </div>

      <AppStatusbar />
    </SidebarProvider>
  </ConfigProvider>
</template>
