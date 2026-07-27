/* Generated from commands/handler_registry.rs; do not edit. */

import { invoke } from '@tauri-apps/api/core'
import type { SyncAllAssetsInput } from '@/types/generated/sync/SyncAllAssetsInput'
import type { SyncAssetInfo } from '@/types/generated/sync/SyncAssetInfo'
import type { SyncAssetOperationInput } from '@/types/generated/sync/SyncAssetOperationInput'
import type { SyncFolderInfo } from '@/types/generated/sync/SyncFolderInfo'
import type { SyncOperationResult } from '@/types/generated/sync/SyncOperationResult'
import type { SyncStatusInfo } from '@/types/generated/sync/SyncStatusInfo'
import type { WebDavConfigDetails } from '@/types/generated/sync/WebDavConfigDetails'
import type { WebDavConfigInput } from '@/types/generated/sync/WebDavConfigInput'
import type { WebDavTestResult } from '@/types/generated/sync/WebDavTestResult'

export type AddSyncFolderInput = { name: string; localPath: string; remotePath: string; description?: string }
export type UpdateSyncFolderInput = { id: string; name?: string; enabled?: boolean; localPath?: string; remotePath?: string; description?: string }

export const syncPush = (force?: boolean): Promise<SyncOperationResult> => invoke('sync_push', { force })
export const syncPull = (force?: boolean): Promise<SyncOperationResult> => invoke('sync_pull', { force })
export const listSyncAssets = (): Promise<SyncAssetInfo[]> => invoke('list_sync_assets')
export const syncPushAsset = (payload: SyncAssetOperationInput): Promise<SyncOperationResult> => invoke('sync_push_asset', { payload })
export const syncPullAsset = (payload: SyncAssetOperationInput): Promise<SyncOperationResult> => invoke('sync_pull_asset', { payload })
export const syncAsset = (payload: SyncAssetOperationInput): Promise<SyncOperationResult> => invoke('sync_asset', { payload })
export const syncAllAssets = (payload: SyncAllAssetsInput = {}): Promise<SyncOperationResult> => invoke('sync_all_assets', { payload })
export const syncPushFolder = (id: string, force?: boolean): Promise<SyncOperationResult> => invoke('sync_push_folder', { id, force })
export const syncPullFolder = (id: string, force?: boolean): Promise<SyncOperationResult> => invoke('sync_pull_folder', { id, force })
export const syncStatus = (): Promise<SyncStatusInfo> => invoke('sync_status')
export const listSyncFolders = (): Promise<SyncFolderInfo[]> => invoke('list_sync_folders')
export const addSyncFolder = (input: AddSyncFolderInput): Promise<SyncFolderInfo> => invoke('add_sync_folder', input)
export const updateSyncFolder = (input: UpdateSyncFolderInput): Promise<SyncFolderInfo> => invoke('update_sync_folder', input)
export const deleteSyncFolder = (id: string): Promise<SyncOperationResult> => invoke('delete_sync_folder', { id })
export const setWebdavConfig = (payload: WebDavConfigInput): Promise<WebDavConfigDetails> => invoke('set_webdav_config', { payload })
export const testWebdavConfig = (payload: WebDavConfigInput): Promise<WebDavTestResult> => invoke('test_webdav_config', { payload })
export const clearWebdavConfig = (): Promise<void> => invoke('clear_webdav_config')
