<script setup lang="ts">
import { AtSign, Hash, Zap } from '@lucide/vue'
import { computed, nextTick, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { formatDateTimeSeconds } from '@/lib/datetime'
import { parseTodoInput } from '@/lib/todo-parse'
import { useTodoStore } from '@/stores/todo'
import AttachmentList from '@/components/attachments/AttachmentList.vue'
import AttachmentPicker from '@/components/attachments/AttachmentPicker.vue'
import { flushPendingAttachments, type PendingAttachment } from '@/lib/attachments'

const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{ close: [] }>()

const { t } = useI18n()
const store = useTodoStore()

const draft = ref('')
const saving = ref(false)
const error = ref('')
const inputEl = ref<HTMLTextAreaElement | null>(null)
const pendingFiles = ref<PendingAttachment[]>([])

const preview = computed(() => parseTodoInput(draft.value))
const canSave = computed(() => !!preview.value.title && !saving.value)

watch(
  () => props.open,
  async (open) => {
    if (!open) {
      window.removeEventListener('keydown', onDialogKeydown, true)
      return
    }
    draft.value = ''
    error.value = ''
    pendingFiles.value = []
    // 设置页可能刚建了项目，打开捕捉时重拉一次
    void store.refreshProjects()
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
  error.value = ''
  pendingFiles.value = []
  emit('close')
}

function onDialogKeydown(event: KeyboardEvent) {
  if (!props.open)
    return
  if (event.key === 'Escape') {
    event.preventDefault()
    close()
    return
  }
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault()
    void save()
  }
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

async function resolveProjectId(name: string | null) {
  if (!name)
    return null
  const existing = store.projects.find(p => p.name === name)
  if (existing)
    return existing.id
  const { useMuseStore } = await import('@/stores/muse')
  const created = await useMuseStore().addProject(name)
  await store.refreshProjects()
  return created.id
}

async function save() {
  if (!canSave.value)
    return
  saving.value = true
  error.value = ''
  try {
    const parsed = preview.value
    const projectId = await resolveProjectId(parsed.project)
    const task = await store.create({
      title: parsed.title,
      dueAt: parsed.dueAt,
      priority: parsed.priority,
      projectId,
      tagNames: parsed.tags,
      source: 'quick',
    })
    if (pendingFiles.value.length && task?.id) {
      const { errors } = await flushPendingAttachments('todo', task.id, pendingFiles.value)
      if (errors.length)
        error.value = t('attachments.partialFailed')
    }
    if (!error.value)
      close()
  }
  catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  }
  finally {
    saving.value = false
  }
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="fixed inset-0 z-[90] flex items-start justify-center bg-black/40 pt-[18vh] backdrop-blur-sm"
      @click.self="close"
    >
      <div
        class="bg-background border-border relative z-10 w-full max-w-xl overflow-hidden rounded-xl border shadow-2xl"
        role="dialog"
        :aria-label="t('todo.capture.title')"
        @click.stop
      >
        <div class="flex items-center gap-2 border-b px-4 py-3">
          <Zap class="text-primary size-4" />
          <span class="text-sm font-medium">{{ t('todo.capture.title') }}</span>
          <span class="text-muted-foreground ml-auto text-[11px]">{{ t('todo.capture.hint') }}</span>
        </div>

        <div class="space-y-3 p-4">
          <textarea
            ref="inputEl"
            v-model="draft"
            rows="3"
            class="placeholder:text-muted-foreground w-full resize-none bg-transparent text-sm outline-none"
            :placeholder="t('todo.capture.placeholder', { hash: '#', at: '@' })"
          />

          <div class="text-muted-foreground flex flex-wrap gap-2 text-[11px]">
            <span v-if="preview.dueAt">
              {{ t('todo.fields.due') }}:
              {{ formatDateTimeSeconds(preview.dueAt) }}
            </span>
            <span v-if="preview.priority !== 'none'">
              {{ t('todo.fields.priority') }}: {{ t(`todo.priority.${preview.priority}`) }}
            </span>
            <span v-if="preview.project">@{{ preview.project }}</span>
            <span v-for="tag in preview.tags" :key="tag">#{{ tag }}</span>
          </div>

          <div class="space-y-1">
            <AttachmentList
              :pending="pendingFiles"
              @remove-pending="path => pendingFiles = pendingFiles.filter(item => item.path !== path)"
            />
            <AttachmentPicker
              mode="pending"
              :remaining="5 - pendingFiles.length"
              @pending="item => pendingFiles.push(item)"
              @error="message => error = message"
            />
          </div>

          <p v-if="error" class="text-destructive text-xs">
            {{ error }}
          </p>

          <div class="flex items-center gap-2">
            <button
              type="button"
              class="text-muted-foreground hover:bg-muted rounded-md p-1.5"
              :title="t('todo.capture.insertTag')"
              @click="insertSymbol('#')"
            >
              <Hash class="size-4" />
            </button>
            <button
              type="button"
              class="text-muted-foreground hover:bg-muted rounded-md p-1.5"
              :title="t('todo.capture.insertProject')"
              @click="insertSymbol('@')"
            >
              <AtSign class="size-4" />
            </button>
            <div class="ml-auto flex gap-2">
              <button
                type="button"
                class="text-muted-foreground hover:bg-muted rounded-md px-3 py-1.5 text-sm"
                @click="close"
              >
                {{ t('todo.capture.close') }}
              </button>
              <button
                type="button"
                class="bg-primary text-primary-foreground rounded-md px-3 py-1.5 text-sm disabled:opacity-50"
                :disabled="!canSave"
                @click="save"
              >
                {{ t('todo.capture.save') }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>
