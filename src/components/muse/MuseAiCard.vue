<script setup lang="ts">
import type { UnlistenFn } from '@tauri-apps/api/event'
import type { DeepseekProgress } from '@/lib/deepseek'
import type { Note } from '@/lib/muse'
import { Check, Copy, Sparkles } from '@lucide/vue'
import { listen } from '@tauri-apps/api/event'
import { nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useMuseToast } from '@/composables/useMuseToast'
import { executeDeepseekTask } from '@/lib/deepseek'
import { renderMarkdown } from '@/lib/markdown'
import { listNotes } from '@/lib/muse'
import { createTodoTask } from '@/lib/todo'
import { useMuseStore } from '@/stores/muse'
import MuseSourceStamp from './MuseSourceStamp.vue'

const props = defineProps<{
  note: Note
}>()

const { t } = useI18n()
const store = useMuseStore()
const { toast } = useMuseToast()

type AiAction = 'outline' | 'relate' | 'task' | 'follow'

interface ChatTurn {
  id: number
  role: 'user' | 'assistant'
  content: string
}

const running = ref<AiAction | null>(null)
const dialogOpen = ref(false)
const dialogTitle = ref('')
const messages = ref<ChatTurn[]>([])
const draft = ref('')
const errorMessage = ref('')
const relatedHits = ref<Note[]>([])
const outputEl = ref<HTMLElement | null>(null)
const copiedId = ref<number | null>(null)

let unlisten: UnlistenFn | undefined
let turnId = 0

function pushTurn(role: ChatTurn['role'], content: string) {
  turnId += 1
  messages.value.push({ id: turnId, role, content })
}

function lastAssistant() {
  for (let index = messages.value.length - 1; index >= 0; index -= 1) {
    const item = messages.value[index]
    if (item?.role === 'assistant')
      return item
  }
  return null
}

function setAssistant(content: string) {
  const last = lastAssistant()
  if (last)
    last.content = content
}

function assistantText() {
  return lastAssistant()?.content ?? ''
}

onMounted(async () => {
  unlisten = await listen<DeepseekProgress>('deepseek-task-progress', (event) => {
    if (!running.value || !event.payload.text)
      return
    setAssistant(event.payload.text)
    void scrollOutput()
  })
})

watch(messages, () => {
  void scrollOutput()
}, { deep: true })

async function scrollOutput() {
  await nextTick()
  const el = outputEl.value
  if (!el)
    return
  el.scrollTop = el.scrollHeight
}

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

function promptWithMarkdown(instruction: string, body: string) {
  return `${instruction}\n${t('muse.ai.prompts.markdown')}\n\n${body}`
}

function clip(text: string, max = 80) {
  const flat = text.replace(/\s+/g, ' ').trim()
  if (flat.length <= max)
    return flat
  return `${flat.slice(0, max)}…`
}

function takeLabel(line: string, labels: string[]) {
  const cleaned = line.replaceAll('**', '').replace(/^#{1,6}\s*/, '').trim()
  for (const label of labels) {
    for (const sep of [':', '：']) {
      const prefix = `${label}${sep}`
      if (cleaned.toLowerCase().startsWith(prefix.toLowerCase()))
        return cleaned.slice(prefix.length).trim()
    }
  }
  return null
}

function parseTaskDraft(text: string, note: Note) {
  let title = ''
  const body: string[] = []
  let inBody = false
  for (const raw of text.split('\n')) {
    const line = raw.trim()
    if (!line)
      continue
    const titleValue = takeLabel(line, ['标题', 'TITLE'])
    if (titleValue !== null) {
      title = titleValue
      inBody = false
      continue
    }
    const bodyValue = takeLabel(line, ['说明', 'BODY', '描述'])
    if (bodyValue !== null) {
      inBody = true
      if (bodyValue)
        body.push(bodyValue)
      continue
    }
    if (inBody)
      body.push(line)
  }
  if (!title)
    title = (note.content.split('\n')[0] || '').trim()
  return { title: title.slice(0, 80), description: body.join('\n') }
}

function matchRelated(text: string, candidates: Note[]) {
  const hits: Note[] = []
  for (const item of candidates) {
    if (!text.includes(item.id))
      continue
    hits.push(item)
    if (hits.length >= 3)
      break
  }
  return hits
}

async function runOutline(note: Note) {
  const prompt = promptWithMarkdown(
    t('muse.ai.prompts.outline'),
    [
      note.content,
      `${t('muse.ai.fieldTags')}${note.tags.join('、') || t('muse.uncategorized')}`,
      `${t('muse.ai.fieldStatus')}${t(`muse.status.${note.status}`)}`,
    ].join('\n'),
  )
  const response = await executeDeepseekTask(prompt)
  if (!response.success)
    throw new Error(describeError(response.error, response.message))
  setAssistant(response.result || assistantText())
  toast(t('muse.ai.toast.outline'))
}

async function runRelate(note: Note) {
  const notes = await listNotes({ kind: 'all' })
  const candidates = notes
    .filter(item => item.id !== note.id && item.content.trim())
    .slice(0, 25)
  if (!candidates.length) {
    setAssistant(t('muse.ai.noneRelated'))
    toast(t('muse.ai.toast.relate', { count: 0 }))
    return
  }
  const catalog = candidates
    .map(item => `- ${item.id} | ${clip(item.content)}`)
    .join('\n')
  const prompt = promptWithMarkdown(
    t('muse.ai.prompts.relate'),
    `${note.content}\n\n${catalog}`,
  )
  const response = await executeDeepseekTask(prompt)
  if (!response.success)
    throw new Error(describeError(response.error, response.message))
  const text = (response.result || assistantText()).trim()
  relatedHits.value = matchRelated(text, candidates)
  setAssistant(relatedHits.value.length === 0 && /^NONE$/i.test(text)
    ? t('muse.ai.noneRelated')
    : text)
  toast(t('muse.ai.toast.relate', { count: relatedHits.value.length }))
}

async function runTask(note: Note) {
  const prompt = promptWithMarkdown(t('muse.ai.prompts.task'), note.content)
  const response = await executeDeepseekTask(prompt)
  if (!response.success)
    throw new Error(describeError(response.error, response.message))
  const text = response.result || assistantText()
  setAssistant(text)
  const draft = parseTaskDraft(text, note)
  if (!draft.title)
    throw new Error(t('muse.ai.taskEmpty'))
  await createTodoTask({
    title: draft.title,
    description: draft.description || note.content,
    tagNames: note.tags,
    projectId: note.projectId,
    source: 'manual',
  })
  toast(t('muse.ai.toast.task', { title: draft.title }))
}

function closeDialog() {
  dialogOpen.value = false
}

async function onAction(action: Exclude<AiAction, 'follow'>) {
  if (running.value)
    return
  const note = props.note
  running.value = action
  dialogTitle.value = t(`muse.ai.action.${action}`)
  dialogOpen.value = true
  messages.value = []
  draft.value = ''
  errorMessage.value = ''
  relatedHits.value = []
  pushTurn('user', t(`muse.ai.action.${action}`))
  pushTurn('assistant', '')
  try {
    if (action === 'outline')
      await runOutline(note)
    else if (action === 'relate')
      await runRelate(note)
    else
      await runTask(note)
  }
  catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error)
    toast(errorMessage.value)
  }
  finally {
    running.value = null
  }
}

async function sendFollowUp() {
  const text = draft.value.trim()
  if (!text || running.value || !dialogOpen.value)
    return
  draft.value = ''
  errorMessage.value = ''
  pushTurn('user', text)
  pushTurn('assistant', '')
  running.value = 'follow'
  try {
    const response = await executeDeepseekTask(`${t('muse.ai.prompts.markdown')}\n\n${text}`)
    if (!response.success)
      throw new Error(describeError(response.error, response.message))
    setAssistant(response.result || assistantText())
  }
  catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error)
    toast(errorMessage.value)
  }
  finally {
    running.value = null
    void scrollOutput()
  }
}

async function copyText(id: number, text: string) {
  if (!text.trim())
    return
  try {
    await navigator.clipboard.writeText(text)
    copiedId.value = id
    toast(t('muse.ai.copied'))
    window.setTimeout(() => {
      if (copiedId.value === id)
        copiedId.value = null
    }, 1500)
  }
  catch {
    toast(t('muse.ai.copyFailed'))
  }
}
</script>

<template>
  <p class="text-muted-foreground flex items-center gap-1.5 text-[10.5px] font-semibold tracking-widest uppercase">
    <Sparkles class="text-primary size-3" />
    {{ t('muse.detail.sectionAi') }}
  </p>
  <div class="border-primary/20 bg-primary/5 rounded-lg border px-3.5 py-3 text-[12.5px] leading-relaxed">
    <p>
      <strong class="text-primary font-semibold">{{ t('muse.ai.keywords') }}</strong>
      {{ note.tags.length ? note.tags.join(' · ') : t('muse.uncategorized') }}
    </p>
    <p class="mt-0.5 flex items-center gap-1.5">
      <strong class="text-primary font-semibold">{{ t('muse.ai.source') }}</strong>
      <MuseSourceStamp :source="note.source" :timestamp="note.createdAt" />
    </p>
    <p class="mt-0.5">
      <strong class="text-primary font-semibold">{{ t('muse.ai.advice') }}</strong>
      {{ t(`muse.ai.suggestion.${note.status}`) }}
    </p>
    <div class="mt-2.5 flex flex-wrap gap-1.5">
      <button
        v-for="action in (['outline', 'relate', 'task'] as const)"
        :key="action"
        type="button"
        class="bg-background border-primary/25 text-primary hover:bg-primary/10 rounded-md border px-2.5 py-1 text-[11px] transition-colors disabled:cursor-not-allowed disabled:opacity-50"
        :disabled="running !== null"
        @click="onAction(action)"
      >
        {{ running === action ? t('muse.ai.running') : t(`muse.ai.action.${action}`) }}
      </button>
    </div>
  </div>

  <Teleport to="body">
    <div
      v-if="dialogOpen"
      class="fixed inset-0 z-100 flex items-center justify-center p-6"
      role="dialog"
      aria-modal="true"
      :aria-label="dialogTitle"
      @keydown.esc="closeDialog"
    >
      <button
        type="button"
        class="absolute inset-0 cursor-default border-0 bg-black/50"
        :aria-label="t('muse.ai.close')"
        @click="closeDialog"
      />
      <div class="bg-background border-border relative z-10 flex w-[min(56rem,92vw)] flex-col rounded-lg border shadow-lg">
        <div class="flex items-center gap-2 border-b px-4 py-3">
          <Sparkles class="text-primary size-4" />
          <h3 class="text-sm font-semibold">
            {{ dialogTitle }}
          </h3>
          <button
            type="button"
            class="text-muted-foreground hover:text-foreground ml-auto text-sm"
            @click="closeDialog"
          >
            {{ t('muse.ai.close') }}
          </button>
        </div>
        <div
          ref="outputEl"
          class="h-[min(40rem,72vh)] overflow-y-auto px-5 py-4"
        >
          <p
            v-if="running && messages.length === 0"
            class="text-muted-foreground text-sm"
          >
            {{ t('muse.ai.waiting') }}
          </p>
          <div class="flex flex-col gap-4">
            <div
              v-for="item in messages"
              :key="item.id"
            >
              <div
                v-if="item.role === 'user'"
                class="bg-primary/10 ml-auto w-fit max-w-[75%] rounded-lg px-3 py-2 text-sm leading-6"
              >
                {{ item.content }}
              </div>
              <div
                v-else
                class="bg-muted/40 relative rounded-lg px-4 py-3 pr-10"
              >
                <div
                  v-if="item.content"
                  class="ai-md text-sm leading-7"
                  v-html="renderMarkdown(item.content)"
                />
                <p
                  v-else
                  class="text-muted-foreground text-sm"
                >
                  {{ t('muse.ai.waiting') }}
                </p>
                <button
                  type="button"
                  class="bg-background text-muted-foreground hover:text-foreground border-border absolute top-2.5 right-2.5 inline-flex size-7 items-center justify-center rounded-md border shadow-sm disabled:pointer-events-none"
                  :aria-label="t('muse.ai.copy')"
                  :title="t('muse.ai.copy')"
                  :disabled="!item.content.trim()"
                  @click="copyText(item.id, item.content)"
                >
                  <Check v-if="copiedId === item.id" class="size-3.5" />
                  <Copy v-else class="size-3.5" />
                </button>
              </div>
            </div>
          </div>
          <p
            v-if="errorMessage"
            class="text-destructive text-sm leading-relaxed"
          >
            {{ errorMessage }}
          </p>
          <div
            v-if="relatedHits.length"
            class="mt-3 flex flex-col gap-1.5"
          >
            <button
              v-for="item in relatedHits"
              :key="item.id"
              type="button"
              class="bg-background border-border hover:border-primary/40 rounded-md border px-2.5 py-2 text-left text-[12px] leading-relaxed"
              @click="store.select(item.id)"
            >
              <span class="line-clamp-2 break-words">{{ item.content }}</span>
            </button>
          </div>
        </div>
        <form
          class="flex items-end gap-2 border-t px-4 py-3"
          @submit.prevent="sendFollowUp"
        >
          <textarea
            v-model="draft"
            rows="2"
            class="border-border bg-background placeholder:text-muted-foreground focus-visible:ring-ring min-h-10 flex-1 resize-none rounded-md border px-3 py-2 text-sm leading-6 outline-none focus-visible:ring-1 disabled:opacity-50"
            :placeholder="t('muse.ai.followPlaceholder')"
            :disabled="running !== null"
            @keydown.enter.exact.prevent="sendFollowUp"
          />
          <button
            type="submit"
            class="bg-primary text-primary-foreground rounded-md px-3 py-2 text-sm disabled:cursor-not-allowed disabled:opacity-50"
            :disabled="running !== null || !draft.trim()"
          >
            {{ running === 'follow' ? t('muse.ai.running') : t('muse.ai.send') }}
          </button>
        </form>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.ai-md :deep(p),
.ai-md :deep(li),
.ai-md :deep(h1),
.ai-md :deep(h2),
.ai-md :deep(h3),
.ai-md :deep(h4),
.ai-md :deep(h5),
.ai-md :deep(h6),
.ai-md :deep(strong),
.ai-md :deep(b) {
  font-size: 1em;
  line-height: inherit;
}

.ai-md :deep(p) {
  margin: 0.55em 0;
}

.ai-md :deep(:first-child) {
  margin-top: 0;
}

.ai-md :deep(:last-child) {
  margin-bottom: 0;
}

.ai-md :deep(h1),
.ai-md :deep(h2),
.ai-md :deep(h3),
.ai-md :deep(h4),
.ai-md :deep(h5),
.ai-md :deep(h6) {
  margin: 0.35em 0;
  font-weight: 600;
}

.ai-md :deep(strong),
.ai-md :deep(b) {
  font-weight: 600;
}

.ai-md :deep(ul),
.ai-md :deep(ol) {
  margin: 0.35em 0;
  padding-left: 1.25em;
}

.ai-md :deep(li) {
  margin: 0.15em 0;
}

.ai-md :deep(code) {
  border-radius: 0.25rem;
  background: color-mix(in oklab, currentColor 8%, transparent);
  padding: 0.05em 0.3em;
  font-size: 0.92em;
}

.ai-md :deep(pre) {
  margin: 0.5em 0;
  overflow-x: auto;
  border-radius: 0.4rem;
  background: color-mix(in oklab, currentColor 8%, transparent);
  padding: 0.6em 0.75em;
}

.ai-md :deep(pre code) {
  background: transparent;
  padding: 0;
}
</style>
