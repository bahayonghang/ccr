/**
 * API 响应类型定义
 * 用于修复 client.ts 中的 any 类型问题
 */

import type { Agent, SlashCommand } from './index'

// ============ MCP Preset Types ============

/** MCP 预设响应 */
export interface McpPreset {
  id: string
  name: string
  description: string
  command?: string
  args: string[]
  tags: string[]
  homepage?: string
  docs?: string
  requires_api_key: boolean
  api_key_env?: string
}

/** 安装预设请求 */
export interface InstallPresetRequest {
  preset_id: string
  platforms: string[]
  env?: Record<string, string>
}

/** 安装结果 */
export interface InstallResult {
  platform: string
  success: boolean
  message?: string
}

// ============ MCP Sync Types ============

/** MCP 服务器信息（同步用） */
export interface McpServerInfo {
  name: string
  command?: string
  args: string[]
  env: Record<string, string>
}

/** 同步请求 */
export interface McpSyncRequest {
  platforms: string[]
}

/** 同步结果 */
export interface McpSyncResult {
  platform: string
  success: boolean
  message?: string
}

/** 同步响应 */
export interface McpSyncResponse {
  message: string
  results: McpSyncResult[]
}

/** 批量同步响应 */
export interface McpSyncAllResponse {
  message: string
  servers: Record<string, McpSyncResult[]>
}

// ============ Sync Folder Types (Multi-folder v2.5+) ============

/** 同步文件夹配置 */
export interface SyncFolder {
  name: string
  description: string
  local_path: string
  remote_path: string
  enabled: boolean
  auto_sync?: boolean
  exclude_patterns?: string[]
}

/** 添加同步文件夹请求 */
export interface AddSyncFolderRequest {
  name: string
  local_path: string
  remote_path?: string
  description?: string
}

/** 同步文件夹列表响应 */
export interface SyncFolderListResponse {
  success: boolean
  folders: SyncFolder[]
  total: number
  enabled_count: number
  disabled_count: number
}

/** 同步文件夹操作响应 */
export interface SyncFolderOperationResponse {
  success: boolean
  message: string
}

/** 同步文件夹同步响应 */
export interface SyncFolderSyncResponse {
  success: boolean
  output: string
  error: string
  duration_ms: number
}

// ============ Platform Types ============

/** 平台列表项 */
export interface PlatformListItem {
  name: string
  enabled: boolean
  current_profile?: string
  description?: string
  last_used?: string
  is_current: boolean
}

/** 当前平台响应 */
export interface CurrentPlatformResponse {
  name: string
  enabled: boolean
  current_profile?: string
  description?: string
  last_used?: string
}

/** 切换平台请求 */
export interface SwitchPlatformRequest {
  platform_name: string
}

/** 平台详情响应 */
export interface PlatformDetailResponse {
  name: string
  enabled: boolean
  current_profile?: string
  description?: string
  last_used?: string
  is_current: boolean
}

/** 更新平台请求 */
export interface UpdatePlatformRequest {
  enabled?: boolean
  description?: string
  current_profile?: string
}

/** 平台 Profile 响应 */
export interface PlatformProfileResponse {
  platform_name: string
  current_profile?: string
}

/** 设置平台 Profile 请求 */
export interface SetPlatformProfileRequest {
  profile_name: string
}

// ============ Agent Folder Types ============

/** Agent 文件夹响应 */
export interface AgentFolder {
  name: string
  path: string
}

// ============ Config Detail Types ============

/** 单个配置详情（用于 getConfig） */
export interface ConfigDetail {
  name: string
  description?: string
  base_url: string
  auth_token: string
  model?: string
  small_fast_model?: string
  provider?: string
  provider_type?: string
  account?: string
  tags?: string[]
  enabled?: boolean
  usage_count?: number
}

/** 更新配置请求数据 */
export interface ConfigUpdateData {
  name?: string
  description?: string
  base_url?: string
  auth_token?: string
  model?: string
  small_fast_model?: string
  provider?: string
  provider_type?: string
  account?: string
  tags?: string[]
  enabled?: boolean
}

// ============ Platform Stub Types ============

/** 通用 Agents 响应（用于未实现的平台） */
export interface PlatformAgentsResponse {
  agents: Agent[]
  folders: string[]
}

/** 通用 SlashCommands 响应（用于未实现的平台） */
export interface PlatformSlashCommandsResponse {
  commands: SlashCommand[]
  folders: string[]
}

/** 通用 Agent 请求（用于未实现的平台） */
export interface PlatformAgentRequest {
  name: string
  model?: string
  tools?: string[]
  system_prompt?: string
  disabled?: boolean
}

/** 通用 SlashCommand 请求（用于未实现的平台） */
export interface PlatformSlashCommandRequest {
  name: string
  description?: string
  command?: string
  args?: string[]
  disabled?: boolean
}

/** 通用 Plugin 请求（用于未实现的平台） */
export interface PlatformPluginRequest {
  id: string
  name?: string
  version?: string
  enabled?: boolean
  config?: Record<string, unknown>
}

/** 通用 MCP Server 请求（用于未实现的平台） */
export interface PlatformMcpServerRequest {
  name?: string
  command?: string
  args?: string[]
  env?: Record<string, string>
  url?: string
}

// ============ Error Handling Types ============

/** 类型守卫：检查是否为 Error 对象 */
export function isError(value: unknown): value is Error {
  return value instanceof Error
}

/** 类型守卫：检查是否有 message 属性 */
export function hasMessage(value: unknown): value is { message: string } {
  return (
    typeof value === 'object' &&
    value !== null &&
    'message' in value &&
    typeof (value as { message: unknown }).message === 'string'
  )
}

/** 从 unknown 错误中提取错误消息 */
export function getErrorMessage(error: unknown, fallback: string = '未知错误'): string {
  if (isError(error)) {
    return error.message
  }
  if (hasMessage(error)) {
    return error.message
  }
  if (typeof error === 'string') {
    return error
  }
  return fallback
}
