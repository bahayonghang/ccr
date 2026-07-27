/**
 * Sync Domain —— WebDAV 同步 API
 *
 * 对应后端 commands::sync::* 命令。
 * 真迁移自 tauri.ts 第 3 分组（同步 / WebDAV）。
 */

import {
  addSyncFolder as addTypedSyncFolder,
  clearWebdavConfig as clearTypedWebdavConfig,
  deleteSyncFolder as deleteTypedSyncFolder,
  listSyncAssets as listTypedSyncAssets,
  listSyncFolders as listTypedSyncFolders,
  setWebdavConfig as setTypedWebdavConfig,
  syncAllAssets as syncAllTypedAssets,
  syncAsset as syncTypedAsset,
  syncPull,
  syncPullAsset,
  syncPullFolder,
  syncPush,
  syncPushAsset,
  syncPushFolder,
  syncStatus,
  testWebdavConfig as testTypedWebdavConfig,
  updateSyncFolder as updateTypedSyncFolder,
} from '../generated/sync'
import type { WebDavConfigInput } from '@/types/generated/sync/WebDavConfigInput'
import type { SyncAssetOperationOptions } from '@/types/syncSelection'

/** 推送配置到远端 */
export const pushSync = syncPush

/** 从远端拉取配置 */
export const pullSync = syncPull

/** 列出固定配置同步资产 */
export const listSyncAssets = listTypedSyncAssets

/** 推送单个配置资产 */
export const pushSyncAsset = async (
  id: string,
  options: SyncAssetOperationOptions = {}
 ) => {
  return syncPushAsset({ id, ...options })
}

/** 拉取单个配置资产 */
export const pullSyncAsset = async (
  id: string,
  options: SyncAssetOperationOptions = {}
) => {
  return syncPullAsset({ id, ...options })
}

/** 同步单个配置资产（默认上传本地资产；本地缺失且远端存在时拉取补齐） */
export const syncSingleAsset = async (
  id: string,
  options: SyncAssetOperationOptions = {}
) => {
  return syncTypedAsset({ id, ...options })
}

/** 对固定配置资产执行一次全量同步 */
export const syncAllAssets = async (
  options: SyncAssetOperationOptions = {}
) => {
  return syncAllTypedAssets(options)
}

/** 推送单个同步文件夹 */
export const pushSyncFolder = syncPushFolder

/** 拉取单个同步文件夹 */
export const pullSyncFolder = syncPullFolder

/** 获取同步状态 */
export const getSyncStatus = syncStatus

/** getSyncInfo —— getSyncStatus 别名，保留以兼容历史调用 */
export const getSyncInfo = getSyncStatus

/** 列出同步文件夹 */
export const listSyncFolders = listTypedSyncFolders

/** 添加同步文件夹 */
export const addSyncFolder = async (
  name: string,
  localPath: string,
  remotePath: string,
  description?: string
) => {
  return addTypedSyncFolder({ name, localPath, remotePath, description })
}

/** 更新同步文件夹 */
export const updateSyncFolder = async (
  id: string,
  name?: string,
  enabled?: boolean,
  localPath?: string,
  remotePath?: string,
  description?: string
) => {
  return updateTypedSyncFolder({ id, name, enabled, localPath, remotePath, description })
}

/** 删除同步文件夹 */
export const deleteSyncFolder = deleteTypedSyncFolder

/** 保存 WebDAV 账号（即启用） */
export const setWebdavConfig = (payload: WebDavConfigInput) => setTypedWebdavConfig(payload)

/** 测试 WebDAV 连接（不持久化） */
export const testWebdavConfig = (payload: WebDavConfigInput) => testTypedWebdavConfig(payload)

/** 断开 WebDAV 账号（清空 canonical 配置并删除 migration-only sync.toml） */
export const clearWebdavConfig = clearTypedWebdavConfig
