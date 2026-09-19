import { invoke } from '@tauri-apps/api/core'

export interface DbStatus {
  ok: boolean
  fragmentCount: number
  dataDir: string
  dbPath: string
}

export function getDbStatus() {
  return invoke<DbStatus>('get_db_status')
}

/** 用系统资源管理器打开应用数据目录 */
export function revealDataDir() {
  return invoke<void>('reveal_data_dir')
}
