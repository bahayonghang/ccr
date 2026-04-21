/**
 * Sync Domain —— WebDAV 同步 API
 *
 * 对应后端 commands::sync::* 命令。
 * 真迁移自 tauri.ts 第 3 分组（同步 / WebDAV）。
 */

import { invoke } from '@tauri-apps/api/core'
import type { UnknownRecord } from '../_shared'
import type { CommandResultLike, SyncFolderItem, SyncStatusResponse } from '../tauri'

/** 推送配置到远端 */
export const pushSync = async <T = UnknownRecord>(force?: boolean): Promise<T> => {
  return invoke('sync_push', { force })
}

/** 从远端拉取配置 */
export const pullSync = async <T = UnknownRecord>(force?: boolean): Promise<T> => {
  return invoke('sync_pull', { force })
}

/** 获取同步状态 */
export const getSyncStatus = async <T = SyncStatusResponse>(): Promise<T> => {
  return invoke('sync_status')
}

/** getSyncInfo —— getSyncStatus 别名，保留以兼容历史调用 */
export const getSyncInfo = getSyncStatus

/** 列出同步文件夹 */
export const listSyncFolders = async <
  T = SyncFolderItem[] | CommandResultLike,
>(): Promise<T> => {
  return invoke('list_sync_folders')
}

/** 添加同步文件夹 */
export const addSyncFolder = async <T = UnknownRecord>(
  name: string,
  localPath: string,
  remotePath: string,
): Promise<T> => {
  return invoke('add_sync_folder', { name, localPath, remotePath })
}

/** 更新同步文件夹 */
export const updateSyncFolder = async <T = UnknownRecord>(
  id: string,
  name?: string,
  enabled?: boolean,
): Promise<T> => {
  return invoke('update_sync_folder', { id, name, enabled })
}

/** 删除同步文件夹 */
export const deleteSyncFolder = async <T = UnknownRecord>(id: string): Promise<T> => {
  return invoke('delete_sync_folder', { id })
}
