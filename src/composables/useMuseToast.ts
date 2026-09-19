import { readonly, ref } from 'vue'

export interface MuseToast {
  id: number
  message: string
}

/** Toast 停留时长（UI 设计规范 4.5：1.8s） */
const TOAST_DURATION = 1800

const toasts = ref<MuseToast[]>([])
let seq = 0

/** 底部居中的轻提示，用于保存、状态切换、归档等即时反馈 */
export function useMuseToast() {
  function toast(message: string) {
    const id = ++seq
    toasts.value = [...toasts.value, { id, message }]
    setTimeout(() => {
      toasts.value = toasts.value.filter(item => item.id !== id)
    }, TOAST_DURATION)
  }

  return { toasts: readonly(toasts), toast }
}
