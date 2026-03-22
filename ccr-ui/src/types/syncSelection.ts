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

export interface SyncStatusConfigView {
  webdav_url?: string
  username?: string
  remote_path?: string
}

export interface SyncStatusView {
  configured?: boolean
  config?: SyncStatusConfigView
}
