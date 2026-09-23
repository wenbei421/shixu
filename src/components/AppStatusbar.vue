<script setup lang="ts">
import {
  CheckSquare2,
  Clock,
  Database,
  FileText,
  Home,
  Settings,
  Sparkles,
  TrendingUp,
} from '@lucide/vue'
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute } from 'vue-router'
import { getDbStatus, revealDataDir } from '@/lib/db'
import { formatDateTimeSeconds } from '@/lib/datetime'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip'

const { t } = useI18n()
const route = useRoute()

/** 应用版本（与 package.json 保持一致，编译期内联） */
const APP_VERSION = '0.0.3'

const clockText = ref('--:--:--')
const now = ref(new Date())
const dbOk = ref(false)
const dbPath = ref('')
const dataDir = ref('')
const fragmentCount = ref(0)

let timer: ReturnType<typeof setInterval> | undefined

/** 当前路由 → 图标 + 标题（与侧边栏对齐） */
const routeView = computed<{ icon: typeof Home, titleKey: string }>(() => {
  const map: Record<string, { icon: typeof Home, titleKey: string }> = {
    '/home': { icon: Home, titleKey: 'nav.home' },
    '/todo': { icon: CheckSquare2, titleKey: 'nav.todo' },
    '/muse': { icon: Sparkles, titleKey: 'nav.muse' },
    '/timeline': { icon: Clock, titleKey: 'nav.timeline' },
    '/insight': { icon: TrendingUp, titleKey: 'nav.insight' },
    '/report': { icon: FileText, titleKey: 'nav.report' },
    '/plan': { icon: CheckSquare2, titleKey: 'nav.plan' },
    '/settings': { icon: Settings, titleKey: 'nav.settings' },
  }
  return map[route.path] ?? { icon: Home, titleKey: 'nav.home' }
})

function tick() {
  now.value = new Date()
  clockText.value = formatDateTimeSeconds(now.value)
}

async function refreshDb() {
  try {
    const status = await getDbStatus()
    dbOk.value = status.ok
    dbPath.value = status.dbPath
    dataDir.value = status.dataDir
    fragmentCount.value = status.fragmentCount
  }
  catch {
    dbOk.value = false
  }
}

async function openDataDir() {
  try {
    await revealDataDir()
  }
  catch {
    // 静默失败：状态栏不应该弹错误
  }
}

onMounted(() => {
  tick()
  void refreshDb()
  timer = setInterval(tick, 1_000)
})

onUnmounted(() => {
  if (timer)
    clearInterval(timer)
})
</script>

<template>
  <TooltipProvider :delay-duration="200" :skip-delay-duration="400">
    <footer
      class="border-border bg-background/80 text-muted-foreground z-20 flex h-7 w-full shrink-0 items-center gap-1.5 border-t px-3 text-[11px] backdrop-blur-md"
      role="contentinfo"
    >
      <!-- 左：当前路由上下文 -->
      <Tooltip>
        <TooltipTrigger as-child>
          <div class="hover:text-foreground flex items-center gap-1.5 rounded px-1.5 py-0.5 transition-colors">
            <component :is="routeView.icon" class="size-3.5 shrink-0" />
            <span class="font-medium">{{ t(routeView.titleKey) }}</span>
          </div>
        </TooltipTrigger>
        <TooltipContent side="top">
          <p>{{ t('statusbar.routeTooltip', { path: route.path }) }}</p>
        </TooltipContent>
      </Tooltip>

      <span aria-hidden="true" class="bg-border mx-1 h-3 w-px" />

      <!-- 中：真实数据状态 -->
      <span class="tabular-nums" :title="t('statusbar.fragments', { count: fragmentCount })">
        {{ t('statusbar.fragments', { count: fragmentCount }) }}
      </span>

      <div class="ml-auto flex items-center gap-1">
        <!-- 版本号 -->
        <Tooltip>
          <TooltipTrigger as-child>
            <span class="hover:text-foreground cursor-default rounded px-1.5 py-0.5 transition-colors">
              {{ t('statusbar.version', { version: APP_VERSION }) }}
            </span>
          </TooltipTrigger>
          <TooltipContent side="top">
            <p>{{ t('statusbar.versionTooltip', { version: APP_VERSION }) }}</p>
          </TooltipContent>
        </Tooltip>

        <span aria-hidden="true" class="bg-border mx-1 h-3 w-px" />

        <!-- SQLite 状态（可点击打开数据目录） -->
        <Tooltip>
          <TooltipTrigger as-child>
            <button
              type="button"
              class="hover:text-foreground hover:bg-muted/60 inline-flex items-center gap-1.5 rounded px-1.5 py-0.5 transition-colors"
              :aria-label="dbOk ? t('statusbar.sqliteOpenDir') : t('statusbar.sqliteOffline')"
              :disabled="!dbOk"
              @click="openDataDir"
            >
              <Database class="size-3.5 shrink-0" />
              <span
                class="size-1.5 shrink-0 rounded-full"
                :class="dbOk ? 'bg-emerald-500' : 'bg-destructive'"
                aria-hidden="true"
              />
              <span>{{ dbOk ? t('statusbar.sqliteReady') : t('statusbar.sqliteOffline') }}</span>
            </button>
          </TooltipTrigger>
          <TooltipContent side="top" class="max-w-sm">
            <p class="whitespace-pre-line">
              {{ dbOk ? t('statusbar.sqliteTooltip', { dbPath }) : t('statusbar.sqliteOffline') }}
            </p>
          </TooltipContent>
        </Tooltip>

        <span aria-hidden="true" class="bg-border mx-1 h-3 w-px" />

        <!-- 时钟 -->
        <Tooltip>
          <TooltipTrigger as-child>
            <span class="hover:text-foreground cursor-default rounded px-1.5 py-0.5 font-mono tabular-nums transition-colors">
              {{ clockText }}
            </span>
          </TooltipTrigger>
          <TooltipContent side="top">
            <p>{{ t('statusbar.clockTooltip', { datetime: formatDateTimeSeconds(now) }) }}</p>
          </TooltipContent>
        </Tooltip>
      </div>
    </footer>
  </TooltipProvider>
</template>
