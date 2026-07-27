import type { SyncAssetInfo as GeneratedSyncAssetInfo } from './generated/sync/SyncAssetInfo'
import type { SyncAssetKind as GeneratedSyncAssetKind } from './generated/sync/SyncAssetKind'
import type { SyncAssetOperationInput } from './generated/sync/SyncAssetOperationInput'
import type { SyncEncryptionState as GeneratedSyncEncryptionState } from './generated/sync/SyncEncryptionState'
import type { SyncOperationFailure as GeneratedSyncOperationFailure } from './generated/sync/SyncOperationFailure'
import type { SyncOperationResult as GeneratedSyncOperationResult } from './generated/sync/SyncOperationResult'
import type { SyncStatusInfo } from './generated/sync/SyncStatusInfo'

export type SyncOperationFailure = GeneratedSyncOperationFailure

export type SyncOperationOutputStatus = 'success' | 'partial' | 'failed'

export interface SyncOperationOutputFailure {
  assetId?: string
  assetName: string
  message: string
  reason: string
  localPath?: string
  remotePath?: string
  advice?: string
}

export interface SyncOperationOutput {
  status: SyncOperationOutputStatus
  title: string
  summary: string
  total?: number
  successCount?: number
  failedCount: number
  durationMs?: number
  failures: SyncOperationOutputFailure[]
  suggestions: string[]
  rawLog: string
}

export type SyncOperationResult = GeneratedSyncOperationResult

export type SyncAssetKind = GeneratedSyncAssetKind
export type SyncAssetOperation = 'push' | 'pull' | 'sync'
export type SyncEncryptionState = GeneratedSyncEncryptionState

export type SyncAssetOperationOptions = Omit<SyncAssetOperationInput, 'id'>

export type SyncAssetInfo = GeneratedSyncAssetInfo

export interface SyncAssetGroup {
  key: string
  title: string
  description: string
  assets: SyncAssetInfo[]
}

/**
 * 与后端 SyncStatusInfo（commands/sync.rs）保持字段同名 snake_case
 */
export type SyncStatusView = SyncStatusInfo
