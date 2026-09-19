<script setup lang="ts">
import type { UnlistenFn } from '@tauri-apps/api/event'
import type { DeepseekProgress } from '@/lib/deepseek'
import type { TodoTask } from '@/lib/todo'
import { listen } from '@tauri-apps/api/event'
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { executeDeepseekTask, openDeepseekWindow } from '@/lib/deepseek'
import { listNotes } from '@/lib/muse'
import { listTodoTasks } from '@/lib/todo'

const props = defineProps<{
  titleKey: string
  promptKey: string
}>()

const { t } = useI18n({ useScope: 'global' })

const prompt = ref(t(props.promptKey))
const attachContext = ref(true)
const status = ref<'idle' | 'opening' | 'running' | 'completed' | 'error'>('idle')
const result = ref('')
const errorMessage = ref('')
const loading = ref(false)

let unlisten: UnlistenFn | undefined

const statusText = computed(() => t(`deepseek.status.${status.value}`))
const canRun = computed(() => !loading.value && prompt.value.trim().length > 0)

onMounted(async () => {
  unlisten = await listen<DeepseekProgress>('deepseek-task-progress', (event) => {
    const payload = event.payload
    if (payload.type === 'waiting')
      status.value = 'opening'
    else if (payload.type === 'error')
      status.value = 'error'
    else if (payload.type === 'completed')
      status.value = 'completed'
    else
      status.value = 'running'

    if (payload.text)
      result.value = payload.text
    if (payload.type === 'error')
      errorMessage.value = describeError(payload.error, payload.message)
  })
})

onUnmounted(() => {
  unlisten?.()
})

function describeError(code?: string | null, message?: string | null) {
  const key = code || message
  if (key) {
    const translated = t(`deepseek.errors.${key}`)
    if (translated !== `deepseek.errors.${key}`)
      return translated
  }
  return message || code || t('deepseek.errors.generic')
}

function clip(text: string, max = 180) {
  const flat = text.replace(/\s+/g, ' ').trim()
  if (flat.length <= max)
    return flat
  return `${flat.slice(0, max)}…`
}

async function buildPrompt() {
  const instruction = prompt.value.trim()
  if (!attachContext.value)
    return instruction

  const sections: string[] = []

  try {
    const [today, inbox] = await Promise.all([
      listTodoTasks({ perspective: 'today' }),
      listTodoTasks({ perspective: 'inbox' }),
    ])
    const seen = new Set<string>()
    const tasks: TodoTask[] = []
    for (const task of [...today, ...inbox]) {
      if (seen.has(task.id))
        continue
      seen.add(task.id)
      tasks.push(task)
    }
    const lines = tasks.slice(0, 30).map((task) => {
      const statusLabel = t(`todo.status.${task.status}`)
      const priority = task.priority === 'none' ? '' : ` ${t(`todo.priority.${task.priority}`)}`
      return `- [${statusLabel}]${priority} ${task.title}`
    })
    if (lines.length)
      sections.push(`${t('deepseek.contextTodos')}\n${lines.join('\n')}`)
  }
  catch {
    // 本地待办读不到时，仍然只发送提示词。
  }

  try {
    const notes = await listNotes({ kind: 'all' })
    const lines = notes
      .slice(0, 20)
      .map(note => clip(note.content))
      .filter(text => text.length > 0)
      .map(text => `- ${text}`)
    if (lines.length)
      sections.push(`${t('deepseek.contextNotes')}\n${lines.join('\n')}`)
  }
  catch {
    // 灵感读不到时，仍然只发送提示词。
  }

  if (!sections.length)
    return instruction
  return `${instruction}\n\n${sections.join('\n\n')}`
}

async function openWindow() {
  errorMessage.value = ''
  try {
    await openDeepseekWindow()
  }
  catch (error) {
    status.value = 'error'
    errorMessage.value = describeError(null, String(error))
  }
}

async function run() {
  if (!canRun.value)
    return
  loading.value = true
  status.value = 'opening'
  result.value = ''
  errorMessage.value = ''
  try {
    const fullPrompt = await buildPrompt()
    const response = await executeDeepseekTask(fullPrompt)
    if (response.success) {
      result.value = response.result || result.value
      status.value = 'completed'
    }
    else {
      status.value = 'error'
      errorMessage.value = describeError(response.error, response.message)
    }
  }
  catch (error) {
    status.value = 'error'
    errorMessage.value = describeError(null, String(error))
  }
  finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="flex h-full min-h-0 flex-col gap-4">
    <div class="flex flex-col gap-2">
      <h1 class="text-2xl font-bold tracking-tight">
        {{ t(titleKey) }}
      </h1>
      <p class="text-muted-foreground max-w-3xl text-sm leading-6">
        {{ t('deepseek.description') }}
      </p>
    </div>

    <Textarea
      v-model="prompt"
      class="min-h-28"
      :disabled="loading"
      :placeholder="t('deepseek.promptPlaceholder')"
    />

    <div class="flex flex-wrap items-center gap-3">
      <label class="text-muted-foreground flex items-center gap-2 text-sm">
        <input
          v-model="attachContext"
          type="checkbox"
          class="accent-primary size-4"
          :disabled="loading"
        >
        {{ t('deepseek.attachContext') }}
      </label>
      <div class="ml-auto flex items-center gap-2">
        <Button
          type="button"
          variant="outline"
          :disabled="loading"
          @click="openWindow"
        >
          {{ t('deepseek.open') }}
        </Button>
        <Button
          type="button"
          :disabled="!canRun"
          @click="run"
        >
          {{ loading ? t('deepseek.running') : t('deepseek.run') }}
        </Button>
      </div>
    </div>

    <p class="text-sm">
      <span class="text-muted-foreground">{{ t('deepseek.statusLabel') }}</span>
      {{ statusText }}
    </p>
    <p
      v-if="errorMessage"
      class="text-destructive text-sm"
    >
      {{ errorMessage }}
    </p>

    <section class="bg-muted/40 min-h-0 flex-1 overflow-auto rounded-lg border p-4">
      <h2 class="mb-2 text-sm font-medium">
        {{ t('deepseek.result') }}
      </h2>
      <p
        v-if="result"
        class="text-sm leading-6 whitespace-pre-wrap"
      >
        {{ result }}
      </p>
      <p
        v-else
        class="text-muted-foreground text-sm"
      >
        {{ t('deepseek.resultEmpty') }}
      </p>
    </section>
  </div>
</template>
