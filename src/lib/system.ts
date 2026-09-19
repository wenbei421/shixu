import { invoke } from '@tauri-apps/api/core'

/** 清空当前环境的业务数据。库文件保留，不会重新写入种子数据。 */
export function resetSystem() {
  return invoke<void>('reset_system')
}
