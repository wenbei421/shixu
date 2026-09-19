import { invoke } from '@tauri-apps/api/core'

export interface DeepseekProgress {
  taskId: string
  type: string
  text?: string
  result?: string | null
  generating?: boolean
  stableCount?: number
  error?: string | null
  message?: string | null
}

export interface DeepseekTaskResponse {
  success: boolean
  result?: string | null
  error?: string | null
  message?: string | null
  elapsedMs: number
}

export function openDeepseekWindow() {
  return invoke<void>('open_deepseek_window')
}

export function executeDeepseekTask(prompt: string, timeoutMs = 180_000) {
  return invoke<DeepseekTaskResponse>('execute_deepseek_task', {
    request: { prompt, timeoutMs },
  })
}
