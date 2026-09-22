<script setup lang="ts">
import type { Attachment, AttachmentOwnerType } from '@/lib/attachments'
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useMuseToast } from '@/composables/useMuseToast'
import {
  listAttachments,
  MAX_ATTACHMENTS_PER_OWNER,
  removeAttachment,
} from '@/lib/attachments'
import AttachmentList from './AttachmentList.vue'
import AttachmentPicker from './AttachmentPicker.vue'
import AttachmentPreview from './AttachmentPreview.vue'

const props = defineProps<{
  ownerType: AttachmentOwnerType
  ownerId: string | null
}>()

const pickerRef = ref<{ handlePaste: (event: ClipboardEvent) => void } | null>(null)

function handlePaste(event: ClipboardEvent) {
  pickerRef.value?.handlePaste(event)
}

defineExpose({ handlePaste })

const { t } = useI18n()
const { toast } = useMuseToast()
const items = ref<Attachment[]>([])
const preview = ref<Attachment | null>(null)
const previewOpen = ref(false)

const remaining = computed(() => Math.max(0, MAX_ATTACHMENTS_PER_OWNER - items.value.length))

async function reload() {
  if (!props.ownerId) {
    items.value = []
    return
  }
  try {
    items.value = await listAttachments(props.ownerType, props.ownerId)
  }
  catch (e) {
    toast(e instanceof Error ? e.message : String(e))
  }
}

watch(() => [props.ownerType, props.ownerId] as const, () => {
  void reload()
}, { immediate: true })

async function onRemove(id: string) {
  try {
    await removeAttachment(id)
    items.value = items.value.filter(item => item.id !== id)
  }
  catch (e) {
    toast(e instanceof Error ? e.message : String(e))
  }
}
</script>

<template>
  <section class="space-y-2">
    <div class="flex items-center justify-between gap-2">
      <p class="text-muted-foreground text-[10.5px] font-semibold tracking-widest uppercase">
        {{ t('attachments.title') }}
      </p>
      <AttachmentPicker
        v-if="ownerId"
        ref="pickerRef"
        mode="owner"
        :owner-type="ownerType"
        :owner-id="ownerId"
        :remaining="remaining"
        @added="reload"
        @error="toast"
      />
    </div>
    <p v-if="!items.length" class="text-muted-foreground text-xs">
      {{ t('attachments.empty') }}
    </p>
    <AttachmentList
      :saved="items"
      @remove-saved="onRemove"
      @preview="(item) => { preview = item; previewOpen = true }"
      @error="toast"
    />
    <AttachmentPreview
      :open="previewOpen"
      :attachment="preview"
      @close="previewOpen = false"
    />
  </section>
</template>
