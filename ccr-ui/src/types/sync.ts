// WebDAV sync type definitions

export type { SyncFolderInfo, SyncFolderInfo as SyncFolderItem } from './generated/sync/SyncFolderInfo'
export type { SyncStatusInfo, SyncStatusInfo as SyncStatusResponse } from './generated/sync/SyncStatusInfo'
export type { WebDavConfigDetails } from './generated/sync/WebDavConfigDetails'
export type { WebDavConfigInput } from './generated/sync/WebDavConfigInput'
export type { WebDavTestResult } from './generated/sync/WebDavTestResult'

export interface SyncOperationRequest {
  force?: boolean;
}

export interface SyncInfoResponse {
  feature_name: string;
  description: string;
  supported_services: string[];
  setup_steps: string[];
  security_notes: string[];
}

// ── 账号管理（弹窗 / 测试连接 / 断开）──

export type WebDavProvider = 'nutstore' | 'nextcloud' | 'owncloud' | 'custom'

/** 服务商预设：URL 模板（custom 不在预设中，UI 单独处理） */
export const WEBDAV_PROVIDER_PRESETS: Record<Exclude<WebDavProvider, 'custom'>, string> = {
  nutstore: 'https://dav.jianguoyun.com/dav/',
  nextcloud: 'https://your-host/remote.php/dav/files/USERNAME/',
  owncloud: 'https://your-host/remote.php/dav/files/USERNAME/',
}

/** 由 WebDAV URL 反推 Provider（仅 nutstore 可精确匹配） */
export const detectProvider = (url: string | undefined): WebDavProvider => {
  if (!url) return 'nutstore'
  if (url === WEBDAV_PROVIDER_PRESETS.nutstore) return 'nutstore'
  return 'custom'
}

export interface SyncResult {
  platform: string
  success: boolean
  message?: string
}
