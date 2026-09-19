<script setup lang="ts">
import type { BackupInfo, MuseProject, MuseTag } from '@/lib/muse'
import {
  HardDrive,
  Merge,
  MoreHorizontal,
  Trash2,
} from '@lucide/vue'
import { open, save } from '@tauri-apps/plugin-dialog'
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { useMuseToast } from '@/composables/useMuseToast'
import { TAG_PALETTE } from '@/lib/muse'
import { relativeTime } from '@/lib/muse-format'
import { useMuseStore } from '@/stores/muse'

export type MuseManageSection = 'tags' | 'projects' | 'backup'

const props = defineProps<{
  section: MuseManageSection
}>()

const { t, locale } = useI18n()
const store = useMuseStore()
const { toast } = useMuseToast()

const loading = ref(false)
const backups = ref<BackupInfo[]>([])

const renameDraft = ref('')
const renamingId = ref<string | null>(null)
const renameInputEl = ref<HTMLInputElement | null>(null)
const mergeFromId = ref<string | null>(null)
const mergeToId = ref('')

const projectNameDraft = ref('')
const projectRenameId = ref<string | null>(null)
const projectRenameDraft = ref('')
const projectRenameInputEl = ref<HTMLInputElement | null>(null)

const confirmDeleteTagId = ref<string | null>(null)
const confirmDeleteProjectId = ref<string | null>(null)
const confirmRestorePath = ref<string | null>(null)
/** 当前打开的色板弹出层（标签/项目 id） */
const colorMenuId = ref<string | null>(null)

const allTags = computed(() => store.tags)
const allProjects = computed(() => store.projects)

onMounted(() => {
  void refreshPanel()
})

watch(() => props.section, () => {
  resetInlineEditors()
  void refreshPanel()
})

function resetInlineEditors() {
  renamingId.value = null
  renameDraft.value = ''
  mergeFromId.value = null
  mergeToId.value = ''
  projectRenameId.value = null
  projectRenameDraft.value = ''
  projectNameDraft.value = ''
  colorMenuId.value = null
}

function onColorMenuOpenChange(id: string, open: boolean) {
  colorMenuId.value = open ? id : null
}

async function refreshPanel() {
  loading.value = true
  try {
    if (props.section === 'backup')
      backups.value = await store.fetchBackups()
    else
      await store.refreshSidebar()
  }
  catch (e) {
    report(e)
  }
  finally {
    loading.value = false
  }
}

function report(e: unknown) {
  toast(e instanceof Error ? e.message : String(e))
}

function formatSize(bytes: number) {
  if (bytes < 1024)
    return `${bytes} B`
  if (bytes < 1024 * 1024)
    return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

async function startRename(tag: MuseTag) {
  renamingId.value = tag.id
  renameDraft.value = tag.name
  mergeFromId.value = null
  await nextTick()
  renameInputEl.value?.focus()
  renameInputEl.value?.select()
}

function cancelRename() {
  renamingId.value = null
}

async function commitRename(tag: MuseTag) {
  if (renamingId.value !== tag.id)
    return
  const name = renameDraft.value.trim()
  renamingId.value = null
  if (!name || name === tag.name)
    return
  try {
    await store.renameTagById(tag.id, name)
    toast(t('muse.manage.toast.tagRenamed'))
  }
  catch (e) {
    report(e)
  }
}

async function setColor(tag: MuseTag, color: string) {
  colorMenuId.value = null
  try {
    await store.setTagColor(tag.id, color)
  }
  catch (e) {
    report(e)
  }
}

function startMerge(tag: MuseTag) {
  mergeFromId.value = tag.id
  mergeToId.value = ''
  renamingId.value = null
}

async function commitMerge() {
  const fromId = mergeFromId.value
  const toId = mergeToId.value
  if (!fromId || !toId || fromId === toId)
    return
  try {
    await store.mergeTag(fromId, toId)
    mergeFromId.value = null
    mergeToId.value = ''
    toast(t('muse.manage.toast.tagMerged'))
  }
  catch (e) {
    report(e)
  }
}

async function confirmDeleteTag() {
  const id = confirmDeleteTagId.value
  confirmDeleteTagId.value = null
  if (!id)
    return
  try {
    await store.removeTagById(id)
    toast(t('muse.manage.toast.tagDeleted'))
  }
  catch (e) {
    report(e)
  }
}

function requestDeleteTag(id: string) {
  // 等菜单关闭后再开确认框，避免同一次点击落到遮罩上立刻取消
  window.setTimeout(() => {
    confirmDeleteTagId.value = id
  }, 0)
}

async function createProject() {
  const name = projectNameDraft.value.trim()
  if (!name)
    return
  try {
    await store.addProject(name)
    projectNameDraft.value = ''
    toast(t('muse.manage.toast.projectCreated'))
  }
  catch (e) {
    report(e)
  }
}

async function startProjectRename(project: MuseProject) {
  projectRenameId.value = project.id
  projectRenameDraft.value = project.name
  await nextTick()
  projectRenameInputEl.value?.focus()
  projectRenameInputEl.value?.select()
}

function cancelProjectRename() {
  projectRenameId.value = null
}

async function commitProjectRename(project: MuseProject) {
  if (projectRenameId.value !== project.id)
    return
  const name = projectRenameDraft.value.trim()
  projectRenameId.value = null
  if (!name || name === project.name)
    return
  try {
    await store.patchProject(project.id, { name })
    toast(t('muse.manage.toast.projectRenamed'))
  }
  catch (e) {
    report(e)
  }
}

async function setProjectStatus(project: MuseProject, status: MuseProject['status']) {
  try {
    await store.patchProject(project.id, { status })
    toast(t('muse.manage.toast.projectUpdated'))
  }
  catch (e) {
    report(e)
  }
}

async function setProjectColor(project: MuseProject, color: string) {
  colorMenuId.value = null
  try {
    await store.patchProject(project.id, { color })
  }
  catch (e) {
    report(e)
  }
}

async function confirmDeleteProject() {
  const id = confirmDeleteProjectId.value
  confirmDeleteProjectId.value = null
  if (!id)
    return
  try {
    await store.removeProject(id)
    toast(t('muse.manage.toast.projectDeleted'))
  }
  catch (e) {
    report(e)
  }
}

function requestDeleteProject(id: string) {
  window.setTimeout(() => {
    confirmDeleteProjectId.value = id
  }, 0)
}

async function doBackup() {
  try {
    const path = await save({
      defaultPath: `muse_manual_${Date.now()}.db`,
      filters: [{ name: 'SQLite', extensions: ['db'] }],
    })
    const dest = await store.createBackup(path ?? null)
    backups.value = await store.fetchBackups()
    toast(t('muse.manage.toast.backedUp', { path: dest }))
  }
  catch (e) {
    report(e)
  }
}

async function pickAndRestore() {
  try {
    const path = await open({
      multiple: false,
      filters: [{ name: 'SQLite', extensions: ['db'] }],
    })
    if (!path || Array.isArray(path))
      return
    confirmRestorePath.value = path
  }
  catch (e) {
    report(e)
  }
}

async function restoreFromList(path: string) {
  confirmRestorePath.value = path
}

async function confirmRestore() {
  const path = confirmRestorePath.value
  confirmRestorePath.value = null
  if (!path)
    return
  try {
    await store.applyRestore(path)
    toast(t('muse.manage.toast.restoreQueued'))
    await store.reboot()
  }
  catch (e) {
    report(e)
  }
}
</script>

<template>
  <div class="min-h-0 overflow-y-auto">
    <p v-if="loading" class="text-muted-foreground text-xs">
      {{ t('muse.list.loading') }}
    </p>

    <!-- Tags：色点点击弹出调色板 -->
    <div v-else-if="section === 'tags'" class="flex flex-col gap-3">
      <p class="text-muted-foreground text-xs">
        {{ t('muse.manage.tags.hint') }}
      </p>
      <div
        v-if="!allTags.length"
        class="text-muted-foreground py-8 text-center text-sm"
      >
        {{ t('muse.nav.noTags') }}
      </div>
      <div v-else class="border-border bg-card overflow-hidden rounded-xl border">
        <div
          v-for="tag in allTags"
          :key="tag.id"
          class="border-border group border-b px-4 py-3 last:border-b-0"
        >
          <div class="flex items-center gap-3">
            <DropdownMenu
              :open="colorMenuId === tag.id"
              @update:open="onColorMenuOpenChange(tag.id, $event)"
            >
              <DropdownMenuTrigger as-child>
                <button
                  type="button"
                  class="size-3.5 shrink-0 rounded-full outline-none transition-shadow hover:ring-2 hover:ring-ring/40 focus-visible:ring-2 focus-visible:ring-ring/50"
                  :style="{ background: tag.color }"
                  :title="t('muse.manage.tags.pickColor')"
                  :aria-label="t('muse.manage.tags.pickColor')"
                />
              </DropdownMenuTrigger>
              <DropdownMenuContent class="w-auto p-2" align="start" :side-offset="8">
                <div class="grid grid-cols-5 gap-1.5">
                  <button
                    v-for="color in TAG_PALETTE"
                    :key="color"
                    type="button"
                    class="size-6 rounded-full border-2 transition-transform hover:scale-110"
                    :class="tag.color === color ? 'border-foreground' : 'border-transparent'"
                    :style="{ background: color }"
                    :title="color"
                    @click="setColor(tag, color)"
                  />
                </div>
              </DropdownMenuContent>
            </DropdownMenu>

            <input
              v-if="renamingId === tag.id"
              ref="renameInputEl"
              v-model="renameDraft"
              class="border-input bg-background h-7 min-w-0 flex-1 rounded-md border px-2 text-sm"
              @blur="commitRename(tag)"
              @keydown.enter.prevent="commitRename(tag)"
              @keydown.esc.prevent="cancelRename"
            >
            <button
              v-else
              type="button"
              class="hover:text-foreground min-w-0 flex-1 truncate text-left text-sm font-medium"
              :title="t('muse.manage.tags.renameHint')"
              @click="startRename(tag)"
            >
              {{ tag.name }}
            </button>
            <span class="text-muted-foreground shrink-0 text-[11px] tabular-nums">
              {{ tag.noteCount }}
            </span>
            <DropdownMenu>
              <DropdownMenuTrigger as-child>
                <button
                  type="button"
                  class="text-muted-foreground/50 hover:bg-muted hover:text-foreground -mr-1 flex size-7 shrink-0 items-center justify-center rounded-md transition-colors"
                  :aria-label="t('muse.manage.more')"
                >
                  <MoreHorizontal class="size-4" />
                </button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end" class="w-36">
                <DropdownMenuItem @select="startMerge(tag)">
                  <Merge class="size-3.5" />
                  {{ t('muse.manage.tags.merge') }}
                </DropdownMenuItem>
                <DropdownMenuSeparator />
                <DropdownMenuItem variant="destructive" @select="requestDeleteTag(tag.id)">
                  <Trash2 class="size-3.5" />
                  {{ t('muse.manage.delete') }}
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>

          <div
            v-if="mergeFromId === tag.id"
            class="bg-muted/40 mt-2.5 flex items-center gap-2 rounded-md px-2 py-1.5"
          >
            <Merge class="text-muted-foreground size-3.5 shrink-0" />
            <span class="text-muted-foreground shrink-0 text-xs">{{ t('muse.manage.tags.mergeInto') }}</span>
            <select
              v-model="mergeToId"
              class="border-input bg-background h-7 min-w-0 flex-1 rounded-md border px-2 text-xs"
            >
              <option value="" disabled>
                {{ t('muse.manage.tags.pickTarget') }}
              </option>
              <option
                v-for="other in allTags.filter(item => item.id !== tag.id)"
                :key="other.id"
                :value="other.id"
              >
                {{ other.name }}
              </option>
            </select>
            <button
              type="button"
              class="bg-primary text-primary-foreground shrink-0 rounded-md px-2 py-1 text-xs disabled:opacity-40"
              :disabled="!mergeToId"
              @click="commitMerge"
            >
              {{ t('muse.manage.tags.confirmMerge') }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Projects -->
    <div v-else-if="section === 'projects'" class="flex flex-col gap-3">
      <p class="text-muted-foreground text-xs">
        {{ t('muse.manage.projects.hint') }}
      </p>
      <div class="flex gap-2">
        <input
          v-model="projectNameDraft"
          class="border-input bg-background h-8 flex-1 rounded-md border px-2 text-sm"
          :placeholder="t('muse.manage.projects.newPlaceholder')"
          @keydown.enter="createProject"
        >
        <button
          type="button"
          class="bg-primary text-primary-foreground rounded-md px-3 text-xs font-medium disabled:opacity-40"
          :disabled="!projectNameDraft.trim()"
          @click="createProject"
        >
          {{ t('muse.manage.projects.create') }}
        </button>
      </div>
      <div
        v-if="!allProjects.length"
        class="text-muted-foreground py-8 text-center text-sm"
      >
        {{ t('muse.nav.noProjects') }}
      </div>
      <div v-else class="border-border bg-card overflow-hidden rounded-xl border">
        <div
          v-for="project in allProjects"
          :key="project.id"
          class="border-border group border-b px-4 py-3 last:border-b-0"
        >
          <div class="flex items-center gap-3">
            <DropdownMenu
              :open="colorMenuId === project.id"
              @update:open="onColorMenuOpenChange(project.id, $event)"
            >
              <DropdownMenuTrigger as-child>
                <button
                  type="button"
                  class="size-3.5 shrink-0 rounded-full outline-none transition-shadow hover:ring-2 hover:ring-ring/40 focus-visible:ring-2 focus-visible:ring-ring/50"
                  :style="{ background: project.color || '#9299a3' }"
                  :title="t('muse.manage.tags.pickColor')"
                  :aria-label="t('muse.manage.tags.pickColor')"
                />
              </DropdownMenuTrigger>
              <DropdownMenuContent class="w-auto p-2" align="start" :side-offset="8">
                <div class="grid grid-cols-5 gap-1.5">
                  <button
                    v-for="color in TAG_PALETTE"
                    :key="color"
                    type="button"
                    class="size-6 rounded-full border-2 transition-transform hover:scale-110"
                    :class="project.color === color ? 'border-foreground' : 'border-transparent'"
                    :style="{ background: color }"
                    @click="setProjectColor(project, color)"
                  />
                </div>
              </DropdownMenuContent>
            </DropdownMenu>
            <div class="min-w-0 flex-1">
              <input
                v-if="projectRenameId === project.id"
                ref="projectRenameInputEl"
                v-model="projectRenameDraft"
                class="border-input bg-background h-7 w-full rounded-md border px-2 text-sm"
                @blur="commitProjectRename(project)"
                @keydown.enter.prevent="commitProjectRename(project)"
                @keydown.esc.prevent="cancelProjectRename"
              >
              <button
                v-else
                type="button"
                class="hover:text-foreground block w-full truncate text-left text-sm font-medium"
                :title="t('muse.manage.tags.renameHint')"
                @click="startProjectRename(project)"
              >
                {{ project.name }}
              </button>
              <p class="text-muted-foreground text-[11px] tabular-nums">
                {{ t('muse.manage.projects.stats', { notes: project.noteCount, done: project.doneCount }) }}
              </p>
            </div>
            <select
              class="text-muted-foreground hover:bg-muted hover:text-foreground h-7 shrink-0 cursor-pointer rounded-md border-0 bg-transparent px-1.5 text-xs outline-none focus-visible:ring-2 focus-visible:ring-ring/40"
              :value="project.status"
              :aria-label="t('muse.manage.projects.statusLabel')"
              @change="setProjectStatus(project, ($event.target as HTMLSelectElement).value as MuseProject['status'])"
            >
              <option value="active">
                {{ t('muse.manage.projects.status.active') }}
              </option>
              <option value="paused">
                {{ t('muse.manage.projects.status.paused') }}
              </option>
              <option value="done">
                {{ t('muse.manage.projects.status.done') }}
              </option>
              <option value="archived">
                {{ t('muse.manage.projects.status.archived') }}
              </option>
            </select>
            <DropdownMenu>
              <DropdownMenuTrigger as-child>
                <button
                  type="button"
                  class="text-muted-foreground/50 hover:bg-muted hover:text-foreground -mr-1 flex size-7 shrink-0 items-center justify-center rounded-md transition-colors"
                  :aria-label="t('muse.manage.more')"
                >
                  <MoreHorizontal class="size-4" />
                </button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end" class="w-36">
                <DropdownMenuItem
                  variant="destructive"
                  :disabled="project.noteCount > 0"
                  :title="project.noteCount > 0 ? t('muse.manage.projects.deleteBlocked') : undefined"
                  @select="requestDeleteProject(project.id)"
                >
                  <Trash2 class="size-3.5" />
                  {{ t('muse.manage.delete') }}
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        </div>
      </div>
    </div>

    <!-- Backup -->
    <div v-else class="flex flex-col gap-3">
      <p class="text-muted-foreground text-xs">
        {{ t('muse.manage.backup.hint') }}
      </p>
      <div class="flex flex-wrap gap-2">
        <button
          type="button"
          class="bg-primary text-primary-foreground rounded-md px-3 py-1.5 text-xs font-medium"
          @click="doBackup"
        >
          {{ t('muse.manage.backup.manual') }}
        </button>
        <button
          type="button"
          class="border-input hover:bg-accent rounded-md border px-3 py-1.5 text-xs"
          @click="pickAndRestore"
        >
          {{ t('muse.manage.backup.restoreFile') }}
        </button>
      </div>
      <p class="text-muted-foreground text-[11px]">
        {{ t('muse.manage.backup.autoHint') }}
      </p>
      <div
        v-if="!backups.length"
        class="text-muted-foreground py-6 text-center text-sm"
      >
        {{ t('muse.manage.backup.empty') }}
      </div>
      <div
        v-for="item in backups"
        :key="item.path"
        class="border-border bg-card flex items-center gap-2 rounded-xl border px-3 py-2"
      >
        <HardDrive class="text-muted-foreground size-3.5 shrink-0" />
        <div class="min-w-0 flex-1">
          <p class="truncate text-sm font-medium">
            {{ item.fileName }}
          </p>
          <p class="text-muted-foreground text-[11px]">
            {{ formatSize(item.size) }} · {{ relativeTime(item.createdAt, locale) }}
          </p>
        </div>
        <button
          type="button"
          class="text-primary text-xs"
          @click="restoreFromList(item.path)"
        >
          {{ t('muse.manage.backup.restore') }}
        </button>
      </div>
    </div>
  </div>

  <ConfirmDialog
    v-if="section === 'tags'"
    :open="!!confirmDeleteTagId"
    :title="t('muse.manage.tags.deleteTitle')"
    :description="t('muse.manage.tags.confirmDelete')"
    :confirm-label="t('muse.manage.delete')"
    :cancel-label="t('muse.detail.cancel')"
    @update:open="(v) => { if (!v) confirmDeleteTagId = null }"
    @confirm="confirmDeleteTag"
  />
  <ConfirmDialog
    v-if="section === 'projects'"
    :open="!!confirmDeleteProjectId"
    :title="t('muse.manage.projects.deleteTitle')"
    :description="t('muse.manage.projects.confirmDelete')"
    :confirm-label="t('muse.manage.delete')"
    :cancel-label="t('muse.detail.cancel')"
    @update:open="(v) => { if (!v) confirmDeleteProjectId = null }"
    @confirm="confirmDeleteProject"
  />
  <ConfirmDialog
    v-if="section === 'backup'"
    :open="!!confirmRestorePath"
    :title="t('muse.manage.backup.restore')"
    :description="t('muse.manage.backup.confirmRestore')"
    :confirm-label="t('muse.manage.backup.restoreAndRestart')"
    :cancel-label="t('muse.detail.cancel')"
    :destructive="true"
    @update:open="(v) => { if (!v) confirmRestorePath = null }"
    @confirm="confirmRestore"
  />
</template>
