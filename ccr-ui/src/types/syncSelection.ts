export interface SyncOperationFailure {
  folder: string
  message: string
}

export interface SyncOperationResult {
  success?: boolean
  message?: string
  durationMs?: number
  duration_ms?: number
  total?: number
  successCount?: number
  success_count?: number
  failed?: SyncOperationFailure[]
  output?: string
  data?: {
    output?: string
  }
}

export type SyncAssetKind = 'directory' | 'file'
export type SyncAssetOperation = 'push' | 'pull' | 'sync'

export interface SyncAssetInfo {
  id: string
  group: 'ccr' | 'claude' | 'codex' | string
  name: string
  description: string
  kind: SyncAssetKind
  sensitive: boolean
  localPath: string
  local_path?: string
  resolvedLocalPath: string
  resolved_local_path?: string
  remotePath: string
  remote_path?: string
  localExists: boolean
  local_exists?: boolean
  remoteExists?: boolean | null
  remote_exists?: boolean | null
  canonicalName?: string | null
  canonical_name?: string | null
}

export interface SyncAssetGroup {
  key: string
  title: string
  description: string
  assets: SyncAssetInfo[]
}

/**
 * 与后端 SyncStatusInfo（commands/sync.rs）保持字段同名 snake_case
 */
export interface SyncStatusView {
  configured?: boolean
  enabled?: boolean
  webdav_url?: string
  username?: string
  remote_path?: string
  auto_sync?: boolean
  has_password?: boolean
  remote_accessible?: boolean | null
  remote_exists?: boolean | null
}
