<script setup lang="ts">
withDefaults(defineProps<{
  open: boolean
  title: string
  description: string
  confirmLabel?: string
  cancelLabel?: string
  destructive?: boolean
}>(), {
  confirmLabel: '确定',
  cancelLabel: '取消',
  destructive: true,
})

const emit = defineEmits<{
  'update:open': [open: boolean]
  confirm: []
  cancel: []
}>()

function onCancel() {
  emit('update:open', false)
  emit('cancel')
}

function onConfirm() {
  emit('update:open', false)
  emit('confirm')
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="fixed inset-0 z-100 flex items-center justify-center p-4"
      role="dialog"
      aria-modal="true"
      :aria-label="title"
    >
      <button
        type="button"
        class="bg-black/50 absolute inset-0 cursor-default border-0"
        aria-label="close"
        @click="onCancel"
      />
      <div class="bg-background border-border relative z-10 w-full max-w-sm rounded-md border p-4 shadow-lg">
        <h3 class="text-foreground text-base font-semibold">
          {{ title }}
        </h3>
        <p class="text-muted-foreground mt-2 text-sm leading-relaxed">
          {{ description }}
        </p>
        <div class="mt-4 flex justify-end gap-2">
          <button
            type="button"
            class="border-input bg-background hover:bg-accent hover:text-accent-foreground inline-flex h-8 items-center rounded-md border px-3 text-sm"
            @click="onCancel"
          >
            {{ cancelLabel }}
          </button>
          <button
            type="button"
            class="inline-flex h-8 items-center rounded-md px-3 text-sm text-white"
            :class="destructive ? 'bg-destructive hover:bg-destructive/90' : 'bg-primary hover:bg-primary/90'"
            @click="onConfirm"
          >
            {{ confirmLabel }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
