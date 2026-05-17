// WebDAV sync type definitions

export interface SyncConfigDetails {
  enabled: boolean;
  webdav_url: string;
  username: string;
  remote_path: string;
  auto_sync: boolean;
  remote_file_exists?: boolean | null;
}

export interface SyncStatusResponse {
  success: boolean;
  output: string;
  configured: boolean;
  config?: SyncConfigDetails | null;
}

export interface SyncOperationRequest {
  force?: boolean;
}

export interface SyncOperationResponse {
  success: boolean;
  output: string;
  error: string;
  duration_ms: number;
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

/** set/test 命令入参（前端 camelCase，与后端 #[serde(rename_all="camelCase")] 对齐） */
export interface WebDavConfigInput {
  webdavUrl: string
  username: string
  password: string
  remotePath?: string
  autoSync?: boolean
}

/** set 命令返回值 */
export interface WebDavConfigDetails {
  enabled: boolean
  webdavUrl: string
  username: string
  remotePath: string
  autoSync: boolean
  hasPassword: boolean
}

/** test 命令返回值 */
export interface WebDavTestResult {
  ok: boolean
  message: string
}
