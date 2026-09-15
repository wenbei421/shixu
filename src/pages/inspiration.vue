<script setup lang="ts">
import { ArrowLeft, Lightbulb, Pencil, Plus, Search, Tag, Trash2, X } from '@lucide/vue'
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import InspirationEditor from '@/components/inspiration/InspirationEditor.vue'
import InspirationPreview from '@/components/inspiration/InspirationPreview.vue'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  UNCLASSIFIED_CATEGORY_ID,
  createCategory,
  createInspiration,
  createTag,
  deleteCategory,
  deleteInspiration,
  deleteTag,
  getInspiration,
  listCategories,
  listInspirations,
  listTags,
  updateInspiration,
  type InspirationCategory,
  type InspirationDetail,
  type InspirationSummary,
  type InspirationTag,
} from '@/lib/inspiration'
import { cn } from '@/lib/utils'

const { t } = useI18n()

const categories = ref<InspirationCategory[]>([])
const tags = ref<InspirationTag[]>([])
const items = ref<InspirationSummary[]>([])
const selectedCategoryId = ref<string | 'all'>('all')
const keyword = ref('')
const selectedTagIds = ref<string[]>([])
const loading = ref(false)
const error = ref('')

const panelOpen = ref(false)
const sheetMode = ref<'create' | 'view' | 'edit'>('create')
const detail = ref<InspirationDetail | null>(null)
const formTitle = ref('')
const formHtml = ref('')
const formText = ref('')
const formCategoryId = ref(UNCLASSIFIED_CATEGORY_ID)
const formTagIds = ref<string[]>([])
const newTagInput = ref('')
const newCategoryName = ref('')
const tagManagerOpen = ref(false)
const saving = ref(false)
/** 对齐 RDPMS：编辑器一旦挂载，edit↔view 用 v-show，避免 TipTap BubbleMenu 卸载竞态 */
const editorMounted = ref(false)

const isEditing = computed(() => sheetMode.value === 'create' || sheetMode.value === 'edit')
const editorKey = computed(() => `insp-${detail.value?.id ?? 'draft'}`)
const panelTitle = computed(() => {
  if (sheetMode.value === 'create')
    return t('inspiration.create')
  if (sheetMode.value === 'edit')
    return t('inspiration.edit')
  return t('inspiration.detail')
})

function closePanel() {
  panelOpen.value = false
  detail.value = null
  // 延后卸编辑器，让当前 tick 的更新先结束
  void nextTick(() => {
    editorMounted.value = false
    formHtml.value = ''
    formText.value = ''
  })
}

async function refreshCategories() {
  categories.value = await listCategories()
}

async function refreshTags() {
  tags.value = await listTags()
}

async function refreshList() {
  loading.value = true
  error.value = ''
  try {
    items.value = await listInspirations({
      categoryId: selectedCategoryId.value === 'all' ? null : selectedCategoryId.value,
      tagIds: selectedTagIds.value.length ? selectedTagIds.value : undefined,
      keyword: keyword.value.trim() || undefined,
    })
  }
  catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  }
  finally {
    loading.value = false
  }
}

async function refreshAll() {
  await Promise.all([refreshCategories(), refreshTags(), refreshList()])
}

watch([selectedCategoryId, selectedTagIds], () => {
  void refreshList()
})

let keywordTimer: ReturnType<typeof setTimeout> | undefined
watch(keyword, () => {
  clearTimeout(keywordTimer)
  keywordTimer = setTimeout(() => void refreshList(), 280)
})

onMounted(() => {
  void refreshAll()
})

function openCreate() {
  sheetMode.value = 'create'
  detail.value = null
  formTitle.value = ''
  formHtml.value = ''
  formText.value = ''
  formCategoryId.value = selectedCategoryId.value === 'all'
    ? UNCLASSIFIED_CATEGORY_ID
    : selectedCategoryId.value
  formTagIds.value = [...selectedTagIds.value]
  editorMounted.value = true
  panelOpen.value = true
}

async function openView(id: string) {
  try {
    detail.value = await getInspiration(id)
    sheetMode.value = 'view'
    formHtml.value = detail.value.bodyHtml
    formText.value = detail.value.bodyText
    formTitle.value = detail.value.title
    panelOpen.value = true
  }
  catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  }
}

function startEdit() {
  if (!detail.value)
    return
  sheetMode.value = 'edit'
  formTitle.value = detail.value.title
  formHtml.value = detail.value.bodyHtml
  formText.value = detail.value.bodyText
  formCategoryId.value = detail.value.categoryId ?? UNCLASSIFIED_CATEGORY_ID
  formTagIds.value = [...detail.value.tagIds]
  editorMounted.value = true
}

async function save() {
  saving.value = true
  error.value = ''
  try {
    const payload = {
      title: formTitle.value,
      bodyHtml: formHtml.value,
      bodyText: formText.value,
      categoryId: formCategoryId.value,
      tagIds: formTagIds.value,
    }
    if (sheetMode.value === 'create') {
      detail.value = await createInspiration(payload)
    }
    else if (detail.value) {
      detail.value = await updateInspiration(detail.value.id, payload)
    }
    // 先切到预览（编辑器仅 v-show 隐藏，不销毁）
    sheetMode.value = 'view'
    if (detail.value) {
      formHtml.value = detail.value.bodyHtml
      formText.value = detail.value.bodyText
      formTitle.value = detail.value.title
    }
    await refreshAll()
  }
  catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  }
  finally {
    saving.value = false
  }
}

async function removeCurrent() {
  if (!detail.value)
    return
  if (!window.confirm(t('inspiration.confirmDelete')))
    return
  await deleteInspiration(detail.value.id)
  closePanel()
  await refreshAll()
}

async function addCategory() {
  const name = newCategoryName.value.trim()
  if (!name)
    return
  await createCategory(name)
  newCategoryName.value = ''
  await refreshCategories()
}

async function removeCategory(id: string) {
  if (id === UNCLASSIFIED_CATEGORY_ID)
    return
  if (!window.confirm(t('inspiration.confirmDeleteCategory')))
    return
  await deleteCategory(id)
  if (selectedCategoryId.value === id)
    selectedCategoryId.value = 'all'
  await refreshAll()
}

function toggleFilterTag(id: string) {
  const set = new Set(selectedTagIds.value)
  if (set.has(id))
    set.delete(id)
  else set.add(id)
  selectedTagIds.value = [...set]
}

function toggleFormTag(id: string) {
  const set = new Set(formTagIds.value)
  if (set.has(id))
    set.delete(id)
  else set.add(id)
  formTagIds.value = [...set]
}

async function addFormTag() {
  const name = newTagInput.value.trim()
  if (!name)
    return
  const tag = await createTag(name)
  await refreshTags()
  if (!formTagIds.value.includes(tag.id))
    formTagIds.value = [...formTagIds.value, tag.id]
  newTagInput.value = ''
}

async function removeTag(id: string) {
  if (!window.confirm(t('inspiration.confirmDeleteTag')))
    return
  await deleteTag(id)
  selectedTagIds.value = selectedTagIds.value.filter(x => x !== id)
  formTagIds.value = formTagIds.value.filter(x => x !== id)
  await refreshAll()
}
</script>

<template>
  <div class="flex h-full min-h-0 gap-4">
    <!-- Categories -->
    <aside class="border-border flex w-52 shrink-0 flex-col gap-2 border-r pr-3">
      <div class="text-muted-foreground text-xs font-medium tracking-wide uppercase">
        {{ t('inspiration.categories') }}
      </div>
      <button
        type="button"
        :class="cn(
          'hover:bg-accent flex w-full items-center justify-between rounded-md px-2 py-1.5 text-left text-sm',
          selectedCategoryId === 'all' && 'bg-accent font-medium',
        )"
        @click="selectedCategoryId = 'all'"
      >
        <span>{{ t('inspiration.all') }}</span>
      </button>
      <button
        v-for="cat in categories"
        :key="cat.id"
        type="button"
        :class="cn(
          'hover:bg-accent group flex w-full items-center justify-between rounded-md px-2 py-1.5 text-left text-sm',
          selectedCategoryId === cat.id && 'bg-accent font-medium',
        )"
        @click="selectedCategoryId = cat.id"
      >
        <span class="truncate">{{ cat.name }}</span>
        <span class="text-muted-foreground flex items-center gap-1 text-xs">
          {{ cat.inspirationCount }}
          <button
            v-if="cat.id !== UNCLASSIFIED_CATEGORY_ID"
            type="button"
            class="hover:text-destructive opacity-0 group-hover:opacity-100"
            :title="t('inspiration.deleteCategory')"
            @click.stop="removeCategory(cat.id)"
          >
            <Trash2 class="size-3.5" />
          </button>
        </span>
      </button>
      <div class="mt-auto flex flex-col gap-2 pt-2">
        <Input
          v-model="newCategoryName"
          :placeholder="t('inspiration.newCategory')"
          class="h-8"
          @keydown.enter="addCategory"
        />
        <Button size="sm" variant="outline" class="w-full" @click="addCategory">
          <Plus />
          {{ t('inspiration.addCategory') }}
        </Button>
      </div>
    </aside>

    <!-- Main: list or in-place editor panel -->
    <div class="flex min-h-0 min-w-0 flex-1 flex-col gap-3">
      <template v-if="!panelOpen">
        <div class="flex flex-wrap items-center gap-2">
          <div class="relative min-w-[200px] flex-1">
            <Search class="text-muted-foreground absolute top-1/2 left-2 size-4 -translate-y-1/2" />
            <Input
              v-model="keyword"
              class="pl-8"
              :placeholder="t('inspiration.searchPlaceholder')"
            />
          </div>
          <Button variant="outline" size="sm" @click="tagManagerOpen = !tagManagerOpen">
            <Tag />
            {{ t('inspiration.manageTags') }}
          </Button>
          <Button size="sm" @click="openCreate">
            <Plus />
            {{ t('inspiration.create') }}
          </Button>
        </div>

        <div v-if="tagManagerOpen || tags.length" class="flex flex-wrap items-center gap-1.5">
          <span class="text-muted-foreground text-xs">{{ t('inspiration.tags') }}:</span>
          <button
            v-for="tag in tags"
            :key="tag.id"
            type="button"
            @click="toggleFilterTag(tag.id)"
          >
            <Badge
              :variant="selectedTagIds.includes(tag.id) ? 'default' : 'outline'"
              class="cursor-pointer gap-1 font-normal"
            >
              {{ tag.name }}
              <X
                v-if="tagManagerOpen"
                class="size-3"
                @click.stop="removeTag(tag.id)"
              />
            </Badge>
          </button>
        </div>

        <p v-if="error" class="text-destructive text-sm">
          {{ error }}
        </p>

        <div class="min-h-0 flex-1 overflow-y-auto">
          <div v-if="loading" class="text-muted-foreground py-8 text-center text-sm">
            {{ t('inspiration.loading') }}
          </div>
          <div
            v-else-if="!items.length"
            class="text-muted-foreground flex flex-col items-center gap-2 py-16 text-sm"
          >
            <Lightbulb class="size-8 opacity-40" />
            {{ t('inspiration.empty') }}
          </div>
          <ul v-else class="flex flex-col gap-2">
            <li
              v-for="item in items"
              :key="item.id"
            >
              <button
                type="button"
                class="hover:bg-accent/60 border-border w-full rounded-md border px-3 py-2.5 text-left transition-colors"
                @click="openView(item.id)"
              >
                <div class="flex items-start justify-between gap-2">
                  <div class="min-w-0 flex-1">
                    <div class="truncate font-medium">
                      {{ item.title }}
                    </div>
                    <p class="text-muted-foreground mt-0.5 line-clamp-2 text-sm">
                      {{ item.bodyText || '—' }}
                    </p>
                    <div class="mt-1.5 flex flex-wrap items-center gap-1.5">
                      <Badge v-if="item.categoryName" variant="secondary" class="font-normal">
                        {{ item.categoryName }}
                      </Badge>
                      <Badge
                        v-for="name in item.tagNames"
                        :key="name"
                        variant="outline"
                        class="font-normal"
                      >
                        {{ name }}
                      </Badge>
                    </div>
                  </div>
                  <span class="text-muted-foreground shrink-0 text-xs tabular-nums">
                    {{ item.updatedAt.slice(0, 16) }}
                  </span>
                </div>
              </button>
            </li>
          </ul>
        </div>
      </template>

      <!-- In-place compose / detail (same surface as list, no overlay sheet) -->
      <template v-else>
        <div class="flex items-center gap-2">
          <Button variant="ghost" size="sm" @click="closePanel">
            <ArrowLeft />
            {{ t('inspiration.back') }}
          </Button>
          <h2 class="text-base font-semibold">
            {{ panelTitle }}
          </h2>
          <div class="ml-auto flex items-center gap-2">
            <template v-if="isEditing">
              <Button variant="outline" size="sm" :disabled="saving" @click="closePanel">
                {{ t('inspiration.cancel') }}
              </Button>
              <Button size="sm" :disabled="saving" @click="save">
                {{ t('inspiration.save') }}
              </Button>
            </template>
            <template v-else>
              <Button variant="destructive" size="sm" @click="removeCurrent">
                <Trash2 />
                {{ t('inspiration.delete') }}
              </Button>
              <Button size="sm" @click="startEdit">
                <Pencil />
                {{ t('inspiration.edit') }}
              </Button>
            </template>
          </div>
        </div>

        <p v-if="error" class="text-destructive text-sm">
          {{ error }}
        </p>

        <div class="flex min-h-0 flex-1 flex-col gap-3">
          <div v-show="isEditing" class="flex min-h-0 flex-1 flex-col gap-3">
            <div class="grid gap-3 sm:grid-cols-2">
              <div class="flex flex-col gap-1.5 sm:col-span-2">
                <Label>{{ t('inspiration.title') }}</Label>
                <Input v-model="formTitle" :placeholder="t('inspiration.titlePlaceholder')" />
              </div>
              <div class="flex flex-col gap-1.5">
                <Label>{{ t('inspiration.category') }}</Label>
                <select
                  v-model="formCategoryId"
                  class="border-input bg-background h-9 w-full rounded-md border px-2 text-sm"
                >
                  <option
                    v-for="cat in categories"
                    :key="cat.id"
                    :value="cat.id"
                  >
                    {{ cat.name }}
                  </option>
                </select>
              </div>
              <div class="flex flex-col gap-1.5">
                <Label>{{ t('inspiration.tags') }}</Label>
                <div class="flex gap-2">
                  <Input
                    v-model="newTagInput"
                    class="h-9"
                    :placeholder="t('inspiration.newTag')"
                    @keydown.enter.prevent="addFormTag"
                  />
                  <Button size="sm" variant="outline" class="h-9 shrink-0" @click="addFormTag">
                    <Plus />
                  </Button>
                </div>
              </div>
            </div>
            <div v-if="tags.length" class="flex flex-wrap gap-1.5">
              <button
                v-for="tag in tags"
                :key="tag.id"
                type="button"
                @click="toggleFormTag(tag.id)"
              >
                <Badge
                  :variant="formTagIds.includes(tag.id) ? 'default' : 'outline'"
                  class="cursor-pointer font-normal"
                >
                  {{ tag.name }}
                </Badge>
              </button>
            </div>
            <InspirationEditor
              v-if="editorMounted"
              v-show="isEditing"
              v-model="formHtml"
              class="min-h-0 flex-1"
              :editor-key="editorKey"
              @update:text="formText = $event"
            />
          </div>

          <div v-show="!isEditing && detail" class="flex min-h-0 flex-1 flex-col gap-3">
            <div>
              <h3 class="text-lg font-semibold">
                {{ detail?.title }}
              </h3>
              <div class="mt-1 flex flex-wrap gap-1.5">
                <Badge v-if="detail?.categoryName" variant="secondary">
                  {{ detail?.categoryName }}
                </Badge>
                <Badge v-for="name in detail?.tagNames ?? []" :key="name" variant="outline">
                  {{ name }}
                </Badge>
              </div>
            </div>
            <InspirationPreview
              v-if="detail"
              :html="detail.bodyHtml"
              class="min-h-0 flex-1"
              height="100%"
            />
          </div>
        </div>
      </template>
    </div>
  </div>
</template>
