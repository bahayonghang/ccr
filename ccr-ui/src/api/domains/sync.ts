/**
 * Sync Domain —— WebDAV 同步 API
 *
 * 对应后端 commands::sync::* 命令。
 * 真迁移自 tauri.ts 第 3 分组（同步 / WebDAV）。
 */

import { invoke } from '@tauri-apps/api/core'
import type { UnknownRecord } from '../_shared'
import type { CommandResultLike, SyncFolderItem, SyncStatusResponse } from '../tauri'
import type {
  WebDavConfigDetails,
  WebDavConfigInput,
  WebDavTestResult,
} from '@/types/sync'

/** 推送配置到远端 */
export const pushSync = async <T = UnknownRecord>(force?: boolean): Promise<T> => {
  return invoke('sync_push', { force })
}

/** 从远端拉取配置 */
export const pullSync = async <T = UnknownRecord>(force?: boolean): Promise<T> => {
  return invoke('sync_pull', { force })
}

/** 列出固定配置同步资产 */
export const listSyncAssets = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('list_sync_assets')
}

/** 推送单个配置资产 */
export const pushSyncAsset = async <T = UnknownRecord>(
  id: string,
  force?: boolean,
): Promise<T> => {
  return invoke('sync_push_asset', { id, force })
}

/** 拉取单个配置资产 */
export const pullSyncAsset = async <T = UnknownRecord>(
  id: string,
  force?: boolean,
): Promise<T> => {
  return invoke('sync_pull_asset', { id, force })
}

/** 同步单个配置资产（默认上传本地资产；本地缺失且远端存在时拉取补齐） */
export const syncSingleAsset = async <T = UnknownRecord>(
  id: string,
  force?: boolean,
): Promise<T> => {
  return invoke('sync_asset', { id, force })
}

/** 对固定配置资产执行一次全量同步 */
export const syncAllAssets = async <T = UnknownRecord>(force?: boolean): Promise<T> => {
  return invoke('sync_all_assets', { force })
}

/** 推送单个同步文件夹 */
export const pushSyncFolder = async <T = UnknownRecord>(
  id: string,
  force?: boolean,
): Promise<T> => {
  return invoke('sync_push_folder', { id, force })
}

/** 拉取单个同步文件夹 */
export const pullSyncFolder = async <T = UnknownRecord>(
  id: string,
  force?: boolean,
): Promise<T> => {
  return invoke('sync_pull_folder', { id, force })
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
  description?: string,
): Promise<T> => {
  return invoke('add_sync_folder', { name, localPath, remotePath, description })
}

/** 更新同步文件夹 */
export const updateSyncFolder = async <T = UnknownRecord>(
  id: string,
  name?: string,
  enabled?: boolean,
  localPath?: string,
  remotePath?: string,
  description?: string,
): Promise<T> => {
  return invoke('update_sync_folder', { id, name, enabled, localPath, remotePath, description })
}

/** 删除同步文件夹 */
export const deleteSyncFolder = async <T = UnknownRecord>(id: string): Promise<T> => {
  return invoke('delete_sync_folder', { id })
}

/** 保存 WebDAV 账号（即启用） */
export const setWebdavConfig = async (
  payload: WebDavConfigInput,
): Promise<WebDavConfigDetails> => {
  return invoke('set_webdav_config', { payload })
}

/** 测试 WebDAV 连接（不持久化） */
export const testWebdavConfig = async (
  payload: WebDavConfigInput,
): Promise<WebDavTestResult> => {
  return invoke('test_webdav_config', { payload })
}

/** 断开 WebDAV 账号（物理删除 sync.toml） */
export const clearWebdavConfig = async (): Promise<void> => {
  return invoke('clear_webdav_config')
}
