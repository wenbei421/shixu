import { invoke } from '@tauri-apps/api/core'

export interface DbStatus {
  ok: boolean
  fragmentCount: number
}

export function getDbStatus() {
  return invoke<DbStatus>('get_db_status')
}
