export interface SyncSelectableItem {
  key: string
  name: string
  description: string
  localPath: string
  remotePath: string
  selected: boolean
  icon?: string
  required?: boolean
}

export interface CustomSyncFolderForm {
  name: string
  localPath: string
  remotePath: string
  description: string
}

export interface SyncManagedFolder {
  name: string
  enabled: boolean
  description?: string
  localPath: string
  remotePath: string
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
