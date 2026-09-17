<script setup lang="ts">
import type { Component } from 'vue'
import {
  BarChart3,
  CalendarCheck,
  ChevronsUpDown,
  Clock3,
  FileText,
  Home,
  Lightbulb,
  Settings,
  Sparkles,
} from '@lucide/vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'
import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuBadge,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarRail,
  useSidebar,
} from '@/components/ui/sidebar'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()
const { isMobile } = useSidebar()

interface NavItem {
  titleKey: string
  path: string
  icon: Component
  badge?: string
}

const primaryItems: NavItem[] = [
  { titleKey: 'nav.home', path: '/home', icon: Home },
  { titleKey: 'nav.inspiration', path: '/inspiration', icon: Lightbulb },
  { titleKey: 'nav.muse', path: '/muse', icon: Sparkles },
  { titleKey: 'nav.timeline', path: '/timeline', icon: Clock3 },
  { titleKey: 'nav.insight', path: '/insight', icon: BarChart3 },
  { titleKey: 'nav.report', path: '/report', icon: FileText },
  { titleKey: 'nav.plan', path: '/plan', icon: CalendarCheck, badge: '2' },
]

const secondaryItems: NavItem[] = [
  { titleKey: 'nav.settings', path: '/settings', icon: Settings },
]

const currentUser = {
  nameKey: 'brand.name' as const,
  email: 'local@shixu.os',
  initials: '序',
}

function isActive(path: string) {
  return route.path === path
}
</script>

<template>
  <Sidebar collapsible="icon" variant="sidebar">
    <SidebarContent>
      <SidebarGroup>
        <SidebarGroupLabel>{{ t('sidebar.workspace') }}</SidebarGroupLabel>
        <SidebarGroupContent>
          <SidebarMenu>
            <SidebarMenuItem v-for="item in primaryItems" :key="item.path">
              <SidebarMenuButton
                as-child
                :is-active="isActive(item.path)"
                :tooltip="t(item.titleKey)"
              >
                <RouterLink :to="item.path">
                  <component :is="item.icon" />
                  <span>{{ t(item.titleKey) }}</span>
                </RouterLink>
              </SidebarMenuButton>
              <SidebarMenuBadge v-if="item.badge">
                {{ item.badge }}
              </SidebarMenuBadge>
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarGroupContent>
      </SidebarGroup>

      <SidebarGroup class="mt-auto">
        <SidebarGroupContent>
          <SidebarMenu>
            <SidebarMenuItem v-for="item in secondaryItems" :key="item.path">
              <SidebarMenuButton
                as-child
                :is-active="isActive(item.path)"
                :tooltip="t(item.titleKey)"
              >
                <RouterLink :to="item.path">
                  <component :is="item.icon" />
                  <span>{{ t(item.titleKey) }}</span>
                </RouterLink>
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarGroupContent>
      </SidebarGroup>
    </SidebarContent>

    <SidebarFooter>
      <SidebarMenu>
        <SidebarMenuItem>
          <DropdownMenu>
            <DropdownMenuTrigger as-child>
              <SidebarMenuButton
                size="lg"
                class="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
                :tooltip="t(currentUser.nameKey)"
              >
                <Avatar class="size-8 rounded-lg">
                  <AvatarFallback class="rounded-lg">
                    {{ currentUser.initials }}
                  </AvatarFallback>
                </Avatar>
                <div class="grid min-w-0 flex-1 text-left text-sm leading-tight">
                  <span class="truncate font-medium">{{ t(currentUser.nameKey) }}</span>
                  <span class="text-muted-foreground truncate text-xs">{{ currentUser.email }}</span>
                </div>
                <ChevronsUpDown class="ml-auto" />
              </SidebarMenuButton>
            </DropdownMenuTrigger>
            <DropdownMenuContent
              class="w-(--reka-dropdown-menu-trigger-width) min-w-56 rounded-lg"
              :side="isMobile ? 'bottom' : 'right'"
              align="end"
              :side-offset="4"
            >
              <DropdownMenuLabel class="p-0 font-normal">
                <div class="flex items-center gap-2 px-1 py-1.5 text-left text-sm">
                  <Avatar class="size-8 rounded-lg">
                    <AvatarFallback class="rounded-lg">
                      {{ currentUser.initials }}
                    </AvatarFallback>
                  </Avatar>
                  <div class="grid min-w-0 flex-1 text-left text-sm leading-tight">
                    <span class="truncate font-medium">{{ t(currentUser.nameKey) }}</span>
                    <span class="text-muted-foreground truncate text-xs">{{ currentUser.email }}</span>
                  </div>
                </div>
              </DropdownMenuLabel>
              <DropdownMenuSeparator />
              <DropdownMenuGroup>
                <DropdownMenuItem @click="router.push('/settings')">
                  <Settings />
                  {{ t('nav.settings') }}
                </DropdownMenuItem>
              </DropdownMenuGroup>
            </DropdownMenuContent>
          </DropdownMenu>
        </SidebarMenuItem>
      </SidebarMenu>
    </SidebarFooter>

    <SidebarRail />
  </Sidebar>
</template>
