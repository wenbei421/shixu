<script setup lang="ts">
import type { NoteSource, NoteStatus } from '@/lib/muse'
import { Archive, ArchiveRestore, Check, RotateCcw, Sparkles, Trash2, X } from '@lucide/vue'
import { computed, nextTick, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import AttachmentSection from '@/components/attachments/AttachmentSection.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import OutOfFilterHint from '@/components/OutOfFilterHint.vue'
import { useMuseToast } from '@/composables/useMuseToast'
import { NOTE_SOURCES, STATUS_META, STATUS_ORDER } from '@/lib/muse'
import { relativeTime } from '@/lib/muse-format'
import { cn } from '@/lib/utils'
import { useMuseStore } from '@/stores/muse'
import MuseAiCard from './MuseAiCard.vue'
import MuseStatusBadge from './MuseStatusBadge.vue'

/** 项目下拉里「新建项目」的哨兵值 */
const NEW_PROJECT_OPTION = '__new__'

const { t, locale } = useI18n()
const store = useMuseStore()
const attachmentSection = ref<{ handlePaste: (event: ClipboardEvent) => void } | null>(null)

function onAttachmentPaste(event: ClipboardEvent) {
  attachmentSection.value?.handlePaste(event)
}
const { toast } = useMuseToast()

const contentEl = ref<HTMLElement | null>(null)
const tagInputOpen = ref(false)
const tagDraft = ref('')
const projectDraft = ref('')
const projectInputOpen = ref(false)
const confirmDeleteOpen = ref(false)
const confirmPurgeOpen = ref(false)

const note = computed(() => store.selectedNote)
const isTrash = computed(() => store.isTrashView)

function reportFailure(e: unknown) {
  toast(e instanceof Error ? e.message : String(e))
}

// 切换选中项时把正文同步进 contenteditable：内容由 DOM 持有，不能靠模板插值
watch(
  () => note.value?.id,
  () => {
    tagInputOpen.value = false
    projectInputOpen.value = false
    void nextTick(() => {
      if (contentEl.value)
        contentEl.value.innerText = note.value?.content ?? ''
    })
  },
  { immediate: true },
)

// 正文被别处改动（如捕捉后重拉）时也要刷新 DOM，但编辑中不要打断用户
watch(
  () => note.value?.content,
  (content) => {
    if (!contentEl.value || document.activeElement === contentEl.value)
      return
    contentEl.value.innerText = content ?? ''
  },
)

function restoreContent() {
  if (contentEl.value)
    contentEl.value.innerText = note.value?.content ?? ''
}

async function commitContent() {
  const current = note.value
  if (!current || !contentEl.value)
    return

  const next = contentEl.value.innerText.trim()
  // 清空后失焦视为放弃修改（BR-04-02）
  if (!next || next === current.content) {
    restoreContent()
    return
  }

  try {
    await store.updateContent(current.id, next)
    toast(t('muse.toast.saved'))
  }
  catch (e) {
    reportFailure(e)
    restoreContent()
  }
}

function onContentKeydown(event: KeyboardEvent) {
  if (event.key !== 'Escape')
    return
  // Esc 取消编辑并恢复原值（BR-04-01）
  event.preventDefault()
  restoreContent()
  contentEl.value?.blur()
}

async function setStatus(status: NoteStatus) {
  const current = note.value
  if (!current || current.status === status)
    return
  try {
    await store.setStatus(current.id, status)
    toast(t('muse.toast.statusChanged', { status: t(`muse.status.${status}`) }))
  }
  catch (e) {
    reportFailure(e)
  }
}

function openTagInput() {
  tagInputOpen.value = true
  tagDraft.value = ''
}

async function commitTag() {
  const current = note.value
  const name = tagDraft.value.trim().replace(/^#/, '')
  tagInputOpen.value = false
  tagDraft.value = ''
  if (!current || !name || current.tags.includes(name))
    return

  try {
    await store.addTag(current.id, name)
    toast(t('muse.toast.tagAdded', { tag: name }))
  }
  catch (e) {
    reportFailure(e)
  }
}

async function removeTag(name: string) {
  const current = note.value
  if (!current)
    return
  try {
    await store.removeTag(current.id, name)
  }
  catch (e) {
    reportFailure(e)
  }
}

async function applyProject(name: string | null) {
  const current = note.value
  if (!current)
    return
  try {
    await store.setProject(current.id, name)
    toast(name ? t('muse.toast.projectSet', { project: name }) : t('muse.toast.projectCleared'))
  }
  catch (e) {
    reportFailure(e)
  }
}

async function applySource(source: NoteSource) {
  const current = note.value
  if (!current || current.source === source)
    return
  try {
    await store.setSource(current.id, source)
    toast(t('muse.toast.sourceChanged', { source: t(`muse.source.${source}`) }))
  }
  catch (e) {
    reportFailure(e)
  }
}

function onSourceChange(event: Event) {
  const value = (event.target as HTMLSelectElement).value as NoteSource
  void applySource(value)
}

function onProjectChange(event: Event) {
  const value = (event.target as HTMLSelectElement).value
  if (value === NEW_PROJECT_OPTION) {
    projectInputOpen.value = true
    projectDraft.value = ''
    return
  }
  void applyProject(value || null)
}

async function commitNewProject() {
  const name = projectDraft.value.trim()
  projectInputOpen.value = false
  projectDraft.value = ''
  if (name)
    await applyProject(name)
}

async function markDone() {
  const current = note.value
  if (!current)
    return
  try {
    await store.setStatus(current.id, 'done')
    toast(t('muse.toast.markedDone'))
  }
  catch (e) {
    reportFailure(e)
  }
}

async function toggleArchive() {
  const current = note.value
  if (!current)
    return
  try {
    await store.setArchived(current.id, !current.archived)
    toast(current.archived ? t('muse.toast.unarchived') : t('muse.toast.archived'))
  }
  catch (e) {
    reportFailure(e)
  }
}

async function confirmDelete() {
  const current = note.value
  confirmDeleteOpen.value = false
  if (!current)
    return
  try {
    await store.remove(current.id)
    toast(t('muse.toast.deleted'))
  }
  catch (e) {
    reportFailure(e)
  }
}

async function restoreFromTrash() {
  const current = note.value
  if (!current)
    return
  try {
    await store.restore(current.id)
    toast(t('muse.manage.toast.restored'))
  }
  catch (e) {
    reportFailure(e)
  }
}

async function confirmPurge() {
  const current = note.value
  confirmPurgeOpen.value = false
  if (!current)
    return
  try {
    await store.purge(current.id)
    await store.refreshNotes()
    await store.refreshSidebar()
    toast(t('muse.manage.toast.purged'))
  }
  catch (e) {
    reportFailure(e)
  }
}

defineExpose({
  toggleArchive,
  setStatus,
  requestDelete: () => {
    if (isTrash.value)
      confirmPurgeOpen.value = true
    else
      confirmDeleteOpen.value = true
  },
})
</script>

<template>
  <aside class="border-border flex w-[344px] shrink-0 flex-col gap-3.5 overflow-y-auto border-l pl-4" @paste="onAttachmentPaste">
    <div
      v-if="!note"
      class="text-muted-foreground flex flex-1 flex-col items-center justify-center gap-2.5 px-5 text-center text-[12.5px]"
    >
      <Sparkles class="size-9 opacity-30" />
      <p>{{ t('muse.detail.empty') }}</p>
    </div>

    <template v-else>
      <!-- 回收站：只读 + 恢复 / 彻底删除 -->
      <template v-if="isTrash">
        <div class="flex items-center gap-2">
          <MuseStatusBadge :status="note.status" />
          <span class="text-muted-foreground ml-auto text-[11px]">
            {{ t('muse.manage.trash.deletedAt', { time: relativeTime(note.deletedAt ?? note.updatedAt, locale) }) }}
          </span>
        </div>

        <div class="bg-card border-border text-muted-foreground rounded-lg border px-3.5 py-3 text-[13.5px] leading-relaxed break-words whitespace-pre-wrap">
          {{ note.content }}
        </div>

        <p class="text-muted-foreground text-[11px]">
          {{ t('muse.manage.trash.hint') }}
        </p>

        <div class="border-border mt-auto flex flex-wrap gap-2 border-t pt-3.5">
          <button
            type="button"
            class="bg-primary text-primary-foreground hover:bg-primary/90 flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-xs font-medium shadow-sm transition-colors"
            @click="restoreFromTrash"
          >
            <RotateCcw class="size-3.5" />
            {{ t('muse.manage.trash.restore') }}
          </button>
          <button
            type="button"
            class="bg-destructive text-destructive-foreground hover:bg-destructive/90 flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-xs font-medium transition-colors"
            @click="confirmPurgeOpen = true"
          >
            <Trash2 class="size-3.5" />
            {{ t('muse.manage.trash.purge') }}
          </button>
        </div>
      </template>

      <template v-else>
        <div class="flex items-center gap-2">
          <MuseStatusBadge :status="note.status" />
          <span class="text-muted-foreground ml-auto text-[11px]">
            {{ t('muse.detail.createdAt', { time: relativeTime(note.createdAt, locale) }) }}
          </span>
          <button
            type="button"
            class="text-muted-foreground hover:bg-destructive/10 hover:text-destructive flex size-6 items-center justify-center rounded-md transition-colors"
            :title="t('muse.detail.delete')"
            @click="confirmDeleteOpen = true"
          >
            <Trash2 class="size-3.5" />
          </button>
        </div>

        <div
          ref="contentEl"
          class="bg-card border-border focus:border-primary focus:ring-primary/15 rounded-lg border px-3.5 py-3 text-[13.5px] leading-relaxed break-words whitespace-pre-wrap outline-none transition-colors focus:ring-3"
          contenteditable="true"
          spellcheck="false"
          role="textbox"
          :aria-label="t('muse.detail.contentLabel')"
          @blur="commitContent"
          @keydown="onContentKeydown"
        />

        <p class="text-muted-foreground text-[10.5px] font-semibold tracking-widest uppercase">
          {{ t('muse.detail.sectionStatus') }}
        </p>
        <div class="grid grid-cols-2 gap-1.5">
          <button
            v-for="status in STATUS_ORDER"
            :key="status"
            type="button"
            :class="cn(
              'bg-card flex items-center gap-1.5 rounded-lg border px-2.5 py-1.5 text-left text-xs transition-colors',
              note.status === status
                ? 'font-semibold'
                : 'border-border text-muted-foreground hover:border-muted-foreground/40',
            )"
            :style="note.status === status
              ? {
                color: STATUS_META[status].color,
                borderColor: STATUS_META[status].color,
                boxShadow: `0 0 0 3px ${STATUS_META[status].soft}`,
              }
              : undefined"
            @click="setStatus(status)"
          >
            <span
              class="size-[7px] shrink-0 rounded-full"
              :style="{ background: STATUS_META[status].color }"
            />
            {{ t(`muse.status.${status}`) }}
          </button>
        </div>
        <OutOfFilterHint v-if="store.retainedId === note.id" />

        <p class="text-muted-foreground text-[10.5px] font-semibold tracking-widest uppercase">
          {{ t('muse.detail.sectionTags') }}
        </p>
        <div class="flex flex-wrap items-center gap-1.5">
          <span
            v-for="tag in note.tags"
            :key="tag"
            class="bg-primary/10 text-primary inline-flex items-center gap-1 rounded-md py-1 pr-1 pl-2.5 text-[11.5px] font-medium"
          >
            #{{ tag }}
            <button
              type="button"
              class="hover:bg-primary/20 flex size-4 items-center justify-center rounded opacity-60 transition-opacity hover:opacity-100"
              :title="t('muse.detail.removeTag')"
              @click="removeTag(tag)"
            >
              <X class="size-3" />
            </button>
          </span>

          <input
            v-if="tagInputOpen"
            v-model="tagDraft"
            class="border-primary bg-background w-[88px] rounded-md border px-2 py-1 text-[11.5px] outline-none"
            :placeholder="t('muse.detail.tagPlaceholder')"
            autofocus
            @blur="commitTag"
            @keydown.enter.prevent="commitTag"
            @keydown.esc.prevent="tagInputOpen = false"
          >
          <button
            v-else
            type="button"
            class="border-border text-muted-foreground hover:border-primary hover:text-primary rounded-md border border-dashed px-2.5 py-1 text-[11.5px] transition-colors"
            @click="openTagInput"
          >
            {{ t('muse.detail.addTag') }}
          </button>
        </div>

        <AttachmentSection
          v-if="!isTrash"
          ref="attachmentSection"
          owner-type="muse"
          :owner-id="note.id"
        />

        <p class="text-muted-foreground text-[10.5px] font-semibold tracking-widest uppercase">
          {{ t('muse.detail.sectionProject') }}
        </p>
        <input
          v-if="projectInputOpen"
          v-model="projectDraft"
          class="border-primary bg-background w-full rounded-lg border px-2.5 py-1.5 text-[12.5px] outline-none"
          :placeholder="t('muse.detail.projectPlaceholder')"
          autofocus
          @blur="commitNewProject"
          @keydown.enter.prevent="commitNewProject"
          @keydown.esc.prevent="projectInputOpen = false"
        >
        <select
          v-else
          class="bg-card border-border text-muted-foreground focus:border-primary focus:ring-primary/15 w-full rounded-lg border px-2.5 py-1.5 text-[12.5px] outline-none transition-colors focus:ring-3"
          :value="note.projectName ?? ''"
          @change="onProjectChange"
        >
          <option value="">
            {{ t('muse.detail.noProject') }}
          </option>
          <option v-for="project in store.projects" :key="project.id" :value="project.name">
            {{ project.name }}
          </option>
          <option :value="NEW_PROJECT_OPTION">
            {{ t('muse.detail.newProject') }}
          </option>
        </select>

        <p class="text-muted-foreground text-[10.5px] font-semibold tracking-widest uppercase">
          {{ t('muse.detail.sectionSource') }}
        </p>
        <select
          class="border-input bg-background focus:ring-ring h-8 w-full rounded-md border px-2 text-[12.5px] outline-none focus:ring-1"
          :value="note.source"
          @change="onSourceChange"
        >
          <option v-for="item in NOTE_SOURCES" :key="item" :value="item">
            {{ t(`muse.source.${item}`) }}
          </option>
        </select>

        <MuseAiCard :note="note" />

        <template v-if="store.related.length">
          <p class="text-muted-foreground text-[10.5px] font-semibold tracking-widest uppercase">
            {{ t('muse.detail.sectionRelated', { count: store.related.length }) }}
          </p>
          <button
            v-for="item in store.related"
            :key="item.id"
            type="button"
            class="bg-card border-border hover:border-muted-foreground/40 text-muted-foreground flex flex-col gap-1.5 rounded-lg border px-3 py-2.5 text-left text-xs leading-relaxed transition-colors"
            @click="store.select(item.id)"
          >
            <span class="line-clamp-2 break-words">{{ item.content }}</span>
            <span class="text-muted-foreground/70 text-[10.5px]">
              {{ item.tags.map(tag => `#${tag}`).join(' ') }} · {{ relativeTime(item.createdAt, locale) }}
            </span>
          </button>
        </template>

        <div class="border-border mt-auto flex flex-wrap gap-2 border-t pt-3.5">
          <button
            type="button"
            class="bg-primary text-primary-foreground hover:bg-primary/90 flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-xs font-medium shadow-sm transition-colors"
            @click="markDone"
          >
            <Check class="size-3.5" />
            {{ t('muse.detail.markDone') }}
          </button>
          <button
            type="button"
            class="border-border bg-card text-muted-foreground hover:text-foreground flex items-center gap-1.5 rounded-lg border px-3 py-1.5 text-xs font-medium transition-colors"
            @click="toggleArchive"
          >
            <component :is="note.archived ? ArchiveRestore : Archive" class="size-3.5" />
            {{ note.archived ? t('muse.detail.unarchive') : t('muse.detail.archive') }}
          </button>
        </div>
      </template>
    </template>
  </aside>

  <ConfirmDialog
    :open="confirmDeleteOpen"
    :title="t('muse.detail.delete')"
    :description="t('muse.detail.confirmDelete')"
    :confirm-label="t('muse.detail.delete')"
    :cancel-label="t('muse.detail.cancel')"
    @confirm="confirmDelete"
    @cancel="confirmDeleteOpen = false"
  />
  <ConfirmDialog
    :open="confirmPurgeOpen"
    :title="t('muse.manage.trash.purge')"
    :description="t('muse.manage.trash.confirmPurge')"
    :confirm-label="t('muse.manage.trash.purge')"
    :cancel-label="t('muse.detail.cancel')"
    @confirm="confirmPurge"
    @cancel="confirmPurgeOpen = false"
  />
</template>
