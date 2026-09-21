<script setup lang="ts">
import type { NoteSource } from '@/lib/muse'
import { AtSign, ClipboardPaste, Hash, Zap } from '@lucide/vue'
import { computed, nextTick, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useMuseToast } from '@/composables/useMuseToast'
import { NOTE_SOURCES } from '@/lib/muse'
import { cn } from '@/lib/utils'
import { useMuseStore } from '@/stores/muse'
import AttachmentList from '@/components/attachments/AttachmentList.vue'
import AttachmentPicker from '@/components/attachments/AttachmentPicker.vue'
import { flushPendingAttachments, type PendingAttachment } from '@/lib/attachments'

const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{ close: [] }>()

const SUGGESTION_LIMIT = 5

const { t } = useI18n()
const store = useMuseStore()
const { toast } = useMuseToast()

const draft = ref('')
const pickedTags = ref<string[]>([])
const source = ref<NoteSource>('quick')
const saving = ref(false)
const inputEl = ref<HTMLTextAreaElement | null>(null)
const pendingFiles = ref<PendingAttachment[]>([])

const canSave = computed(() => !!draft.value.trim() && !saving.value)

const suggestions = computed(() => {
  const picked = new Set(pickedTags.value)
  return store.tags
    .map(tag => tag.name)
    .filter(name => !picked.has(name))
    .slice(0, SUGGESTION_LIMIT)
})

watch(
  () => props.open,
  async (open) => {
    if (!open) {
      window.removeEventListener('keydown', onDialogKeydown, true)
      return
    }
    draft.value = ''
    pickedTags.value = []
    source.value = 'quick'
    pendingFiles.value = []
    // 设置页可能刚建了标签/项目，打开捕捉时同步一次侧栏数据
    void store.refreshSidebar()
    // 捕获阶段监听：焦点在按钮、建议标签上时 Esc/Enter 仍然生效
    window.addEventListener('keydown', onDialogKeydown, true)
    await nextTick()
    inputEl.value?.focus()
  },
)

onUnmounted(() => {
  window.removeEventListener('keydown', onDialogKeydown, true)
})

function close() {
  draft.value = ''
  pickedTags.value = []
  source.value = 'quick'
  pendingFiles.value = []
  emit('close')
}

function toggleTag(name: string) {
  if (pickedTags.value.includes(name))
    pickedTags.value = pickedTags.value.filter(tag => tag !== name)
  else
    pickedTags.value = [...pickedTags.value, name]
}

function insertSymbol(symbol: string) {
  const el = inputEl.value
  if (!el) {
    draft.value += symbol
    return
  }
  const start = el.selectionStart ?? draft.value.length
  const end = el.selectionEnd ?? start
  draft.value = `${draft.value.slice(0, start)}${symbol}${draft.value.slice(end)}`
  void nextTick(() => {
    el.focus()
    const cursor = start + symbol.length
    el.setSelectionRange(cursor, cursor)
  })
}

async function pasteClipboard() {
  try {
    const text = await navigator.clipboard.readText()
    if (!text.trim()) {
      toast(t('muse.capture.clipboardEmpty'))
      return
    }
    if (draft.value && !draft.value.endsWith('\n'))
      draft.value += '\n'
    draft.value += text.trim()
    if (source.value === 'quick')
      source.value = 'clip'
    await nextTick()
    inputEl.value?.focus()
  }
  catch {
    toast(t('muse.capture.clipboardDenied'))
  }
}

async function save() {
  if (!canSave.value)
    return
  saving.value = true
  try {
    const note = await store.capture({
      content: draft.value,
      tags: [...pickedTags.value],
      source: source.value,
    })
    if (pendingFiles.value.length && note?.id) {
      const { errors } = await flushPendingAttachments('muse', note.id, pendingFiles.value)
      if (errors.length)
        toast(t('attachments.partialFailed'))
    }
    close()
    toast(t('muse.capture.saved'))
  }
  catch (e) {
    toast(e instanceof Error ? e.message : String(e))
  }
  finally {
    saving.value = false
  }
}

/** 浮层打开期间全局生效：Esc 关闭，Enter 保存，Shift+Enter 留给输入框换行 */
function onDialogKeydown(event: KeyboardEvent) {
  if (!props.open)
    return

  if (event.key === 'Escape') {
    event.preventDefault()
    event.stopPropagation()
    close()
    return
  }

  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault()
    event.stopPropagation()
    void save()
  }
}
</script>

<template>
  <div
    v-if="open"
    class="fixed inset-0 z-[90] flex items-start justify-center bg-black/30 pt-[13vh] backdrop-blur-sm"
    role="dialog"
    aria-modal="true"
    :aria-label="t('muse.capture.mode')"
  >
    <button
      type="button"
      class="absolute inset-0 cursor-default border-0 bg-transparent"
      :aria-label="t('muse.capture.close')"
      @click="close"
    />

    <div class="bg-background relative z-10 w-full max-w-[620px] overflow-hidden rounded-xl shadow-2xl">
      <header class="flex items-center gap-2 px-3.5 pt-3">
        <span class="bg-primary/10 text-primary flex items-center gap-1 rounded-md px-2 py-1 text-[11px] font-semibold">
          <Zap class="size-3" />
          {{ t('muse.capture.mode') }}
        </span>
        <select
          v-model="source"
          class="border-input bg-background text-muted-foreground h-7 rounded-md border px-2 text-[11px]"
          :title="t('muse.capture.source')"
        >
          <option v-for="item in NOTE_SOURCES" :key="item" :value="item">
            {{ t(`muse.source.${item}`) }}
          </option>
        </select>
        <span class="text-muted-foreground ml-auto text-[11px]">{{ t('muse.capture.hint') }}</span>
      </header>

      <textarea
        ref="inputEl"
        v-model="draft"
        class="placeholder:text-muted-foreground max-h-[260px] min-h-24 w-full resize-none bg-transparent px-4 py-3.5 text-[15px] leading-relaxed outline-none"
        :placeholder="t('muse.capture.placeholder')"
      />

      <div class="flex min-h-6 flex-wrap items-center gap-1.5 px-4 pb-2.5">
        <span class="text-muted-foreground flex items-center gap-1.5 text-[10.5px]">
          <span class="bg-primary size-1.5 rounded-full" />
          {{ t('muse.capture.suggest') }}
        </span>
        <button
          v-for="tag in pickedTags"
          :key="`picked-${tag}`"
          type="button"
          class="bg-primary/10 text-primary rounded px-1.5 py-0.5 text-[11px] font-medium"
          @click="toggleTag(tag)"
        >
          #{{ tag }}
        </button>
        <button
          v-for="tag in suggestions"
          :key="`suggest-${tag}`"
          type="button"
          class="bg-muted text-muted-foreground hover:text-foreground rounded px-1.5 py-0.5 text-[11px] font-medium transition-colors"
          @click="toggleTag(tag)"
        >
          + {{ tag }}
        </button>
      </div>

      <div class="space-y-1 px-4 pb-2">
        <AttachmentList
          :pending="pendingFiles"
          @remove-pending="path => pendingFiles = pendingFiles.filter(item => item.path !== path)"
        />
        <AttachmentPicker
          mode="pending"
          :remaining="5 - pendingFiles.length"
          @pending="item => pendingFiles.push(item)"
          @error="toast"
        />
      </div>

      <footer class="border-border bg-muted/40 flex items-center gap-1 border-t px-3 py-2.5">
        <button
          type="button"
          class="text-muted-foreground hover:bg-accent hover:text-foreground flex size-7 items-center justify-center rounded-md transition-colors"
          :title="t('muse.capture.insertTag')"
          @click="insertSymbol('#')"
        >
          <Hash class="size-3.5" />
        </button>
        <button
          type="button"
          class="text-muted-foreground hover:bg-accent hover:text-foreground flex size-7 items-center justify-center rounded-md transition-colors"
          :title="t('muse.capture.insertProject')"
          @click="insertSymbol('@')"
        >
          <AtSign class="size-3.5" />
        </button>
        <button
          type="button"
          class="text-muted-foreground hover:bg-accent hover:text-foreground flex size-7 items-center justify-center rounded-md transition-colors"
          :title="t('muse.capture.pasteClipboard')"
          @click="pasteClipboard"
        >
          <ClipboardPaste class="size-3.5" />
        </button>

        <span class="text-muted-foreground ml-auto mr-2 text-[11px]">
          <kbd class="border-border rounded border px-1 font-sans text-[10px]">Esc</kbd>
          {{ t('muse.capture.close') }}
        </span>
        <button
          type="button"
          :disabled="!canSave"
          :class="cn(
            'bg-primary text-primary-foreground flex items-center gap-1.5 rounded-lg px-3.5 py-1.5 text-[12.5px] font-semibold shadow-sm transition-colors',
            canSave ? 'hover:bg-primary/90' : 'cursor-not-allowed opacity-45 shadow-none',
          )"
          @click="save"
        >
          {{ t('muse.capture.save') }}
          <span class="opacity-70">Enter</span>
        </button>
      </footer>
    </div>
  </div>
</template>
