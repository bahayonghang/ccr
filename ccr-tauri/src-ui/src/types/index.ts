// 🎯 TypeScript 类型定义
// 本小姐精心设计的类型系统！与 Tauri 后端完美对接！(￣▽￣)／

/**
 * 配置信息
 * 对应 Tauri 后端的 ConfigInfo 结构
 */
export interface ConfigInfo {
  name: string
  description: string
  base_url: string | null
  auth_token: string | null
  model: string | null
  small_fast_model: string | null
  is_current: boolean
  is_default: boolean
  // === 🆕 分类字段 ===
  provider: string | null
  provider_type: string | null  // "official_relay" | "third_party_model"
  account: string | null
  tags: string[] | null
}

/**
 * 配置列表响应
 * 包含当前配置、默认配置和所有配置列表
 */
export interface ConfigList {
  current_config: string
  default_config: string
  configs: ConfigInfo[]
}

/**
 * 历史记录条目
 * 对应 Tauri 后端的 HistoryEntry 结构
 */
export interface HistoryEntry {
  id: string
  timestamp: string
  operation: string
  from_config: string | null
  to_config: string | null
  actor: string
}

/**
 * 备份信息
 * 对应 Tauri 后端的 BackupInfo 结构
 */
export interface BackupInfo {
  filename: string
  path: string
  created_at: string
  size: number
}

/**
 * 系统信息
 * 对应 Tauri 后端的 SystemInfo 结构
 */
export interface SystemInfo {
  hostname: string
  username: string
  os: string
  config_path: string
  settings_path: string
  backups_path: string
}

/**
 * 创建配置请求参数
 * 传递给 create_config 命令
 */
export interface CreateConfigRequest {
  name: string
  description?: string
  base_url?: string
  auth_token?: string
  model?: string
  small_fast_model?: string
  provider?: string
  provider_type?: string  // "OfficialRelay" | "ThirdPartyModel"
  account?: string
  tags?: string[]
}

/**
 * 更新配置请求参数
 * 传递给 update_config 命令
 */
export interface UpdateConfigRequest {
  old_name: string
  new_name: string
  description?: string
  base_url?: string
  auth_token?: string
  model?: string
  small_fast_model?: string
  provider?: string
  provider_type?: string  // "OfficialRelay" | "ThirdPartyModel"
  account?: string
  tags?: string[]
}
