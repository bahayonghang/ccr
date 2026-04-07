/**
 * Tauri API Client for CCR Desktop
 *
 * 通过 Tauri invoke() 调用 Rust 后端命令的完整封装。
 * 所有函数名与原 api/modules/ 导出保持一致，以确保 Store 层无缝切换。
 *
 * 分组顺序：
 *   1. 环境检测 & 工具函数
 *   2. 配置管理 (Config)
 *   3. 同步 (Sync / WebDAV)
 *   4. Claude Code 平台
 *   5. Codex 平台
 *   6. Gemini 平台
 *   7. Qwen 平台
 *   8. Qoder 平台
 *   9. Droid 平台
 *  10. OpenCode 平台
 *  11. 签到 (CheckIn)
 *  12. 统计 (Stats)
 *  13. 系统 (System)
 *  14. 转换器 (Converter)
 *  15. UI 状态 (Favorites / Recent Items)
 *  16. WAF
 *  17. 统一 MCP (Unified MCP)
 *  18. 事件 (Events)
 *  19. 环境管理 (Environment)
 *  20. HTTP-only 桩函数 (无 Tauri 命令对应)
 */

import { invoke } from '@tauri-apps/api/core'
import { isTauriRuntime } from '@/utils/tauriRuntime'

type UnknownRecord = Record<string, unknown>

const isRecord = (value: unknown): value is UnknownRecord => {
  return typeof value === 'object' && value !== null
}

const asRecord = (value: unknown): UnknownRecord => {
  return isRecord(value) ? value : {}
}

const asArray = (value: unknown): unknown[] => {
  return Array.isArray(value) ? value : []
}

const pickArray = (value: unknown, key: string): unknown[] => {
  if (!isRecord(value)) {
    return []
  }
  return asArray(value[key])
}

const pickRecord = (value: unknown, key: string): UnknownRecord => {
  if (!isRecord(value)) {
    return {}
  }
  return asRecord(value[key])
}

// ════════════════════════════════════════════════════════════
// 类型导出（兼容历史 `@/api` 类型导入）
// ════════════════════════════════════════════════════════════

export interface HeatmapData {
  data: Record<string, number>
  max_value: number
  total_tokens: number
  active_days: number
}

export interface BuiltinPrompt {
  id: string
  name: string
  description: string
  category: string
  tags: string[]
  content: string
}

export interface SkillRepository {
  name: string
  url: string
  branch?: string
  description?: string
  updated_at?: string
  is_official?: boolean
  skill_count?: number
  last_synced?: string
}

export interface Skill {
  name: string
  description?: string
  path: string
  instruction: string
  metadata?: {
    author?: string
    version?: string
    license?: string
    category?: string
    tags?: string[]
    updated_at?: string
  }
  is_remote?: boolean
  repository?: string
}

export interface ClaudeSettingsData {
  model?: string
  availableModels?: string[]
  alwaysThinkingEnabled?: boolean
  maxThinkingTokens?: number
  maxOutputTokens?: number
  effortLevel?: string
  skipDangerousModePermissionPrompt?: boolean
  theme?: string
  language?: string
  showTurnDuration?: boolean
  prefersReducedMotion?: boolean
  spinnerTipsEnabled?: boolean
  terminalProgressBarEnabled?: boolean
  showSpinnerTree?: boolean
  includeCoAuthoredBy?: boolean
  autoUpdates?: boolean
  autoUpdatesChannel?: string
  cleanupPeriodDays?: number
  respectGitignore?: boolean
  env?: Record<string, string>
  permissions?: {
    allow?: string[]
    deny?: string[]
    defaultMode?: string
    additionalDirectories?: string[]
  }
  sandbox?: {
    enabled?: boolean
    autoAllowBashIfSandboxed?: boolean
    network?: {
      allowLocalBinding?: boolean
      allowedDomains?: string[]
    }
    excludedCommands?: string[]
  }
  attribution?: {
    commit?: string
    pr?: string
  }
  [key: string]: unknown
}

export interface SkillHubAgentSummary {
  id: string
  display_name: string
  global_skills_dir?: string
  detected: boolean
  installed_count: number
}

export interface SkillHubInstalledSkill {
  name: string
  description?: string
  skill_dir: string
}

export interface SkillHubMarketplaceItem {
  package: string
  owner: string
  repo: string
  skill?: string
  skills_sh_url: string
  description?: string
  author_avatar?: string
  stars?: number
}

export interface SkillHubMarketplaceResponse {
  items: SkillHubMarketplaceItem[]
  total: number
  cached: boolean
}

export interface SyncResult {
  platform: string
  success: boolean
  message?: string
}

export interface OAuthAuthorizeUrlResponse {
  success: boolean
  authorize_url?: string
  extraction_guide?: string[]
  message?: string
}

export interface OAuthAuthorizeUrlRequest {
  provider_id: string
  oauth_type: 'github' | 'linuxdo'
}

export interface DroidPlugin {
  id: string
  data: Record<string, unknown>
}

export interface SyncFolderItem {
  name: string
  enabled: boolean
  localPath: string
  remotePath: string
  description?: string
}

export interface SyncStatusResponse {
  configured?: boolean
  config?: {
    webdav_url?: string
    username?: string
    remote_path?: string
  }
  [key: string]: unknown
}

export interface CommandResultLike {
  success?: boolean
  message?: string
  output?: string
  data?: {
    output?: string
  }
}

export interface VersionInfoResponse {
  current_version?: string
  build_time?: string
  git_commit?: string
  latest_version?: string
  has_update?: boolean
  release_url?: string
  release_notes?: string
  published_at?: string
}

function resolveNameAndConfig(
  arg1: string | object,
  arg2?: unknown
): { name: string; config: UnknownRecord } {
  if (typeof arg1 === 'string') {
    return { name: arg1, config: asRecord(arg2) }
  }

  const request = { ...asRecord(arg1) }
  const name = String(request.name ?? request.id ?? '')
  delete request.name
  delete request.id

  return { name, config: request }
}

function resolveName(arg1: string | object): string {
  if (typeof arg1 === 'string') {
    return arg1
  }

  const request = asRecord(arg1)
  return String(request.name ?? request.id ?? '')
}

// ════════════════════════════════════════════════════════════
// 1. 环境检测 & 工具函数
// ════════════════════════════════════════════════════════════

/** 检查是否在 Tauri 桌面应用环境中运行 */
export const isTauriEnvironment = (): boolean => {
  return isTauriRuntime()
}

/** 获取当前运行环境名称 */
export const getEnvironmentName = (): 'tauri' | 'web' => {
  return isTauriEnvironment() ? 'tauri' : 'web'
}

/** 获取 Tauri 版本信息 */
export const getTauriVersion = async (): Promise<string | null> => {
  if (!isTauriEnvironment()) {
    return null
  }
  try {
    const { getVersion } = await import('@tauri-apps/api/app')
    return await getVersion()
  } catch {
    return null
  }
}

const SKIP_EXIT_CONFIRM_KEY = 'ccr_skip_exit_confirm'

/** 获取是否跳过退出确认（优先 Tauri 命令，失败回退本地存储） */
export const getSkipExitConfirm = async (): Promise<boolean> => {
  try {
    return await invoke('get_skip_exit_confirm')
  } catch {
    return localStorage.getItem(SKIP_EXIT_CONFIRM_KEY) === '1'
  }
}

/** 设置是否跳过退出确认（优先 Tauri 命令，失败回退本地存储） */
export const setSkipExitConfirm = async (skip: boolean): Promise<void> => {
  try {
    await invoke('set_skip_exit_confirm', { skip })
    return
  } catch {
    localStorage.setItem(SKIP_EXIT_CONFIRM_KEY, skip ? '1' : '0')
  }
}

/** 兼容旧用法：TauriAPI.getTauriVersion() */
export const TauriAPI = {
  getTauriVersion,
}

// ════════════════════════════════════════════════════════════
// 2. 配置管理 (Config)
// ════════════════════════════════════════════════════════════

/** 列出所有配置（包装为 { configs: [...] } 格式供前端消费） */
export const listConfigs = async <T = UnknownRecord>(): Promise<T> => {
  const configs = await invoke('list_configs')
  return { configs } as T
}

/** 切换到指定配置 */
export const switchConfig = async <T = UnknownRecord>(name: string): Promise<T> => {
  return invoke('switch_config', { name })
}

/** 添加新配置（兼容 addConfig(name, config) 与 addConfig({name,...})） */
export const addConfig = async <T = UnknownRecord>(
  nameOrData: string | object,
  config?: unknown
): Promise<T> => {
  if (typeof nameOrData === 'string') {
    return invoke('add_config', { name: nameOrData, config })
  }
  const data = asRecord(nameOrData)
  const { name, ...rest } = data
  return invoke('add_config', { name, config: rest })
}

/** 删除指定配置 */
export const deleteConfig = async <T = UnknownRecord>(name: string): Promise<T> => {
  return invoke('delete_config', { name })
}

/** 重命名配置 */
export const renameConfig = async <T = UnknownRecord>(
  oldName: string,
  newName: string
): Promise<T> => {
  return invoke('rename_config', { oldName, newName })
}

/** 复制配置 */
export const duplicateConfig = async <T = UnknownRecord>(
  name: string,
  newName: string
): Promise<T> => {
  return invoke('duplicate_config', { name, newName })
}

/** 验证所有配置 */
export const validateConfigs = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('validate_configs')
}

/** 导入配置 */
export const importConfig = async <T = UnknownRecord>(data: unknown): Promise<T> => {
  return invoke('import_config', { data })
}

/** 导出配置 */
export const exportConfig = async <T = UnknownRecord>(name?: string): Promise<T> => {
  return invoke('export_config', { name })
}

/** 获取历史记录（包装为 { entries: [...] } 格式供前端消费） */
export const getHistory = async <T = UnknownRecord>(limit?: number): Promise<T> => {
  const entries = await invoke('get_history', { limit: limit ?? 100 })
  return { entries } as T
}

/** 清理历史记录 */
export const clearHistory = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('clear_history')
}

// ════════════════════════════════════════════════════════════
// 3. 同步 (Sync / WebDAV)
// ════════════════════════════════════════════════════════════

/** 推送配置到远端 */
export const pushSync = async <T = UnknownRecord>(force?: boolean): Promise<T> => {
  return invoke('sync_push', { force })
}

/** 从远端拉取配置 */
export const pullSync = async <T = UnknownRecord>(force?: boolean): Promise<T> => {
  return invoke('sync_pull', { force })
}

/** 获取同步状态 */
export const getSyncStatus = async <T = SyncStatusResponse>(): Promise<T> => {
  return invoke('sync_status')
}

/** getSyncInfo - 同 getSyncStatus 的别名 */
export const getSyncInfo = getSyncStatus

/** 列出同步文件夹 */
export const listSyncFolders = async <T = SyncFolderItem[] | CommandResultLike>(): Promise<T> => {
  return invoke('list_sync_folders')
}

/** 添加同步文件夹 */
export const addSyncFolder = async <T = UnknownRecord>(
  name: string,
  localPath: string,
  remotePath: string
): Promise<T> => {
  return invoke('add_sync_folder', { name, localPath, remotePath })
}

/** 更新同步文件夹 */
export const updateSyncFolder = async <T = UnknownRecord>(
  id: string,
  name?: string,
  enabled?: boolean
): Promise<T> => {
  return invoke('update_sync_folder', { id, name, enabled })
}

/** 删除同步文件夹 */
export const deleteSyncFolder = async <T = UnknownRecord>(id: string): Promise<T> => {
  return invoke('delete_sync_folder', { id })
}

// ════════════════════════════════════════════════════════════
// 4. Claude Code 平台
// ════════════════════════════════════════════════════════════

// ── Claude Settings ──

/** 获取 Claude Code 全局设置 */
export const getClaudeSettings = async <T = ClaudeSettingsData>(): Promise<T> => {
  return invoke('claude_get_settings')
}

/** 更新 Claude Code 全局设置 */
export const updateClaudeSettings = async <T = UnknownRecord>(settings: unknown): Promise<T> => {
  return invoke('claude_update_settings', { settings })
}

// ── Claude MCP Servers ──

/** 列出 Claude Code MCP 服务器 */
export const listMcpServers = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('claude_list_mcp_servers')
}

/** 添加 Claude Code MCP 服务器 */
export const addMcpServer = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('claude_add_mcp_server', { name, config: resolvedConfig })
}

/** 更新 Claude Code MCP 服务器 */
export const updateMcpServer = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('claude_update_mcp_server', { name, config: resolvedConfig })
}

/** 删除 Claude Code MCP 服务器 */
export const deleteMcpServer = async <T = UnknownRecord>(
  nameOrRequest: string | object
): Promise<T> => {
  const name = resolveName(nameOrRequest)
  return invoke('claude_delete_mcp_server', { name })
}

/** 切换 Claude Code MCP 服务器启用/禁用状态（通过更新 disabled 字段实现） */
export const toggleMcpServer = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  disabled?: boolean
): Promise<T> => {
  if (typeof nameOrRequest === 'string') {
    return invoke('claude_update_mcp_server', {
      name: nameOrRequest,
      config: { disabled: !!disabled },
    })
  }
  const name = resolveName(nameOrRequest)
  const request = asRecord(nameOrRequest)
  const resolvedDisabled =
    typeof disabled === 'boolean'
      ? disabled
      : typeof request.disabled === 'boolean'
        ? request.disabled
        : true
  return invoke('claude_update_mcp_server', { name, config: { disabled: resolvedDisabled } })
}

// ── Claude Agents ──

/** 列出 Claude Code Agents */
export const listAgents = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('claude_list_agents')
}

/** 获取单个 Agent 详情（通过列表后过滤实现） */
export const getAgent = async <T = UnknownRecord>(name: string): Promise<T> => {
  const result = await invoke<unknown>('claude_list_agents')
  const agents = Array.isArray(result) ? result : pickArray(result, 'agents')
  const found = agents.find((item) => {
    if (!isRecord(item)) {
      return false
    }
    return String(item.name ?? '') === name
  })
  return (found ?? null) as T
}

/** 添加 Claude Code Agent */
export const addAgent = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('claude_add_agent', { name, config: resolvedConfig })
}

/** 更新 Claude Code Agent */
export const updateAgent = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('claude_update_agent', { name, config: resolvedConfig })
}

/** 删除 Claude Code Agent */
export const deleteAgent = async <T = UnknownRecord>(
  nameOrRequest: string | object
): Promise<T> => {
  const name = resolveName(nameOrRequest)
  return invoke('claude_delete_agent', { name })
}

/** 切换 Agent 启用/禁用状态 */
export const toggleAgent = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  enabled?: boolean
): Promise<T> => {
  const name = resolveName(nameOrRequest)
  const request = asRecord(nameOrRequest)
  const resolvedEnabled =
    typeof enabled === 'boolean'
      ? enabled
      : typeof request.enabled === 'boolean'
        ? request.enabled
        : true
  return invoke('claude_update_agent', { name, config: { enabled: resolvedEnabled } })
}

// ── Claude Slash Commands ──

/** 列出 Claude Code 斜杠命令 */
export const listSlashCommands = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('claude_list_slash_commands')
}

/** 添加 Claude Code 斜杠命令 */
export const addSlashCommand = async <T = UnknownRecord>(
  name: string,
  config: unknown
): Promise<T> => {
  return invoke('claude_add_slash_command', { name, config })
}

/** 更新 Claude Code 斜杠命令 */
export const updateSlashCommand = async <T = UnknownRecord>(
  name: string,
  config: unknown
): Promise<T> => {
  return invoke('claude_update_slash_command', { name, config })
}

/** 删除 Claude Code 斜杠命令 */
export const deleteSlashCommand = async <T = UnknownRecord>(name: string): Promise<T> => {
  return invoke('claude_delete_slash_command', { name })
}

/** 切换斜杠命令启用/禁用状态 */
export const toggleSlashCommand = async <T = UnknownRecord>(
  name: string,
  enabled: boolean
): Promise<T> => {
  return invoke('claude_update_slash_command', { name, config: { enabled } })
}

// ── Claude Plugins ──

/** 列出 Claude Code 插件 */
export const listPlugins = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('claude_list_plugins')
}

/** 添加 Claude Code 插件 */
export const addPlugin = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('claude_add_plugin', { name, config: resolvedConfig })
}

/** 更新 Claude Code 插件 */
export const updatePlugin = async <T = UnknownRecord>(
  name: string,
  config: unknown
): Promise<T> => {
  return invoke('claude_update_plugin', { name, config })
}

/** 删除 Claude Code 插件 */
export const deletePlugin = async <T = UnknownRecord>(name: string): Promise<T> => {
  return invoke('claude_delete_plugin', { name })
}

/** 切换插件启用/禁用状态 */
export const togglePlugin = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  enabled?: boolean
): Promise<T> => {
  const name = resolveName(nameOrRequest)
  const request = asRecord(nameOrRequest)
  const resolvedEnabled =
    typeof enabled === 'boolean'
      ? enabled
      : typeof request.enabled === 'boolean'
        ? request.enabled
        : true
  return invoke('claude_update_plugin', { name, config: { enabled: resolvedEnabled } })
}

// ── Claude Output Styles ──

/** 获取输出样式列表 */
export const listOutputStyles = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('claude_get_output_styles')
}

/** 获取单个输出样式（别名） */
export const getOutputStyle = listOutputStyles

/** 创建输出样式（通过 update 实现） */
export const createOutputStyle = async <T = UnknownRecord>(styles: unknown): Promise<T> => {
  return invoke('claude_update_output_styles', { styles })
}

/** 更新输出样式 */
export const updateOutputStyle = async <T = UnknownRecord>(
  nameOrStyles: string | object,
  patch?: Record<string, unknown>
): Promise<T> => {
  if (typeof nameOrStyles === 'string') {
    return invoke('claude_update_output_styles', { styles: { [nameOrStyles]: patch } })
  }
  return invoke('claude_update_output_styles', { styles: nameOrStyles })
}

/** 删除输出样式（通过 update 清空实现） */
export const deleteOutputStyle = async <T = UnknownRecord>(_name: string): Promise<T> => {
  return invoke('claude_update_output_styles', { styles: {} })
}

// ── Claude Statusline ──

/** 获取状态栏配置 */
export const getStatusline = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('claude_get_statusline')
}

/** 更新状态栏配置 */
export const updateStatusline = async <T = UnknownRecord>(statusline: unknown): Promise<T> => {
  return invoke('claude_update_statusline', { statusline })
}

// ── Claude Hooks ──

/** 列出 Claude Code Hooks */
export const listHooks = async <T = UnknownRecord>(): Promise<T> => {
  const response = asRecord(await invoke<unknown>('claude_list_hooks'))
  return asRecord(response.hooks) as T
}

/** 更新 Claude Code Hooks（批量更新） */
export const updateHooks = async <T = UnknownRecord>(hooks: unknown): Promise<T> => {
  const response = asRecord(await invoke<unknown>('claude_update_hooks', { hooks }))
  return asRecord(response.hooks) as T
}

// ── Claude Budgets ──

/** 获取预算配置 */
export const getBudgetStatus = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('claude_get_budgets')
}

/** 设置预算 */
export const setBudget = async <T = UnknownRecord>(budgets: unknown): Promise<T> => {
  return invoke('claude_update_budgets', { budgets })
}

/** 重置预算 */
export const resetBudget = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('claude_update_budgets', { budgets: {} })
}

// ── Claude Prompts ──

/** 列出提示词 */
export const listPrompts = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('claude_list_prompts')
}

/** 更新提示词 */
export const updatePrompts = async <T = UnknownRecord>(prompts: unknown): Promise<T> => {
  return invoke('claude_update_prompts', { prompts })
}

// ── Claude Profiles ──

/** 列出所有 Claude Profiles */
export const listClaudeProfiles = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('claude_list_profiles')
}

/** 获取单个 Claude Profile */
export const getClaudeProfile = async <T = UnknownRecord>(name: string): Promise<T> => {
  return invoke('claude_get_profile', { name })
}

/** 创建 Claude Profile */
export const addClaudeProfile = async <T = UnknownRecord>(request: unknown): Promise<T> => {
  return invoke('claude_add_profile', { request })
}

/** 更新 Claude Profile */
export const updateClaudeProfile = async <T = UnknownRecord>(
  name: string,
  request: unknown
): Promise<T> => {
  return invoke('claude_update_profile', { name, request })
}

/** 删除 Claude Profile */
export const deleteClaudeProfile = async <T = UnknownRecord>(name: string): Promise<T> => {
  return invoke('claude_delete_profile', { name })
}

/** 应用 Claude Profile */
export const applyClaudeProfile = async <T = UnknownRecord>(name: string): Promise<T> => {
  return invoke('claude_apply_profile', { name })
}

// ════════════════════════════════════════════════════════════
// 5. Codex 平台
// ════════════════════════════════════════════════════════════

/** 列出 Codex Profiles */
export const listCodexProfiles = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('codex_list_profiles')
}

/** 获取 Codex 配置 */
export const getCodexConfig = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('codex_get_settings')
}

/** 更新 Codex 配置 */
export const updateCodexConfig = async <T = UnknownRecord>(settings: unknown): Promise<T> => {
  return invoke('codex_update_settings', { settings })
}

/** 列出 Codex MCP 服务器 */
export const listCodexMcpServers = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('codex_list_mcp_servers')
}

/** 添加 Codex MCP 服务器 */
export const addCodexMcpServer = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('codex_add_mcp_server', { name, config: resolvedConfig })
}

/** 更新 Codex MCP 服务器 */
export const updateCodexMcpServer = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('codex_update_mcp_server', { name, config: resolvedConfig })
}

/** 删除 Codex MCP 服务器 */
export const deleteCodexMcpServer = async <T = UnknownRecord>(
  nameOrRequest: string | object
): Promise<T> => {
  const name = resolveName(nameOrRequest)
  return invoke('codex_delete_mcp_server', { name })
}

/** 列出 Codex Agents */
export const listCodexAgents = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('codex_list_agents')
}

/** 添加 Codex Agent */
export const addCodexAgent = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('codex_add_agent', { name, config: resolvedConfig })
}

/** 更新 Codex Agent */
export const updateCodexAgent = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('codex_update_agent', { name, config: resolvedConfig })
}

/** 删除 Codex Agent */
export const deleteCodexAgent = async <T = UnknownRecord>(
  nameOrRequest: string | object
): Promise<T> => {
  const name = resolveName(nameOrRequest)
  return invoke('codex_delete_agent', { name })
}

/** 切换 Codex Agent 启用/禁用状态 */
export const toggleCodexAgent = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  enabled?: boolean
): Promise<T> => {
  const name = resolveName(nameOrRequest)
  const request = asRecord(nameOrRequest)
  const resolvedEnabled =
    typeof enabled === 'boolean'
      ? enabled
      : typeof request.enabled === 'boolean'
        ? request.enabled
        : true
  return invoke('codex_update_agent', { name, config: { enabled: resolvedEnabled } })
}

// ── Codex Models ──

/** 列出 Codex 可选模型（内置 + 自定义） */
export const listCodexModels = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('codex_list_models')
}

/** 保存 Codex 自定义模型 */
export const addCodexCustomModel = async <T = UnknownRecord>(model: string): Promise<T> => {
  return invoke('codex_add_custom_model', { model })
}

// ── Codex Profile 管理（CCR profiles.toml） ──

/** 添加 Codex Profile */
export const addCodexProfile = async <T = UnknownRecord>(
  profileOrName: string | object,
  config?: unknown
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(profileOrName, config)
  return invoke('codex_add_profile', { name, config: resolvedConfig })
}

/** 更新 Codex Profile */
export const updateCodexProfile = async <T = UnknownRecord>(
  profileOrName: string | object,
  config?: unknown
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(profileOrName, config)
  return invoke('codex_update_profile', { name, config: resolvedConfig })
}

/** 删除 Codex Profile */
export const deleteCodexProfile = async <T = UnknownRecord>(
  nameOrRequest: string | object
): Promise<T> => {
  const name = resolveName(nameOrRequest)
  return invoke('codex_delete_profile', { name })
}

/** 获取 Codex Profile 详情 */
export const getCodexProfile = async <T = UnknownRecord>(name: string): Promise<T> => {
  const profiles = await listCodexProfiles<unknown>()
  const arr = Array.isArray(profiles) ? profiles : pickArray(profiles, 'profiles')
  const found = arr.find((item) => {
    if (!isRecord(item)) {
      return false
    }
    return String(item.name ?? '') === name
  })
  return (found ?? null) as T
}

/** 获取 Codex Profile 的环境变量导出 */
export const getCodexProfileEnv = async <T = UnknownRecord>(name: string): Promise<T> => {
  return invoke('codex_get_profile_env', { name })
}

/** 应用 Codex Profile */
export const applyCodexProfile = async <T = UnknownRecord>(name: string): Promise<T> => {
  return invoke('codex_apply_profile', { name })
}

// ── Codex Sessions ──

/** 列出 Codex Sessions */
export const listCodexSessions = async <T = UnknownRecord>(options?: {
  limit?: number
  query?: string
}): Promise<T> => {
  return invoke('codex_list_sessions', {
    limit: options?.limit,
    query: options?.query,
  })
}

/** 获取 Codex Session 详情 */
export const getCodexSessionDetail = async <T = UnknownRecord>(
  filePath: string,
  messageLimit?: number
): Promise<T> => {
  return invoke('codex_get_session_detail', { filePath, messageLimit })
}

/** 导出 Codex Session Markdown */
export const exportCodexSession = async <T = UnknownRecord>(
  filePath: string,
  maxMessages?: number
): Promise<T> => {
  return invoke('codex_export_session', { filePath, maxMessages })
}

/** 克隆 Codex Session */
export const cloneCodexSession = async <T = UnknownRecord>(filePath: string): Promise<T> => {
  return invoke('codex_clone_session', { filePath })
}

/** 删除 Codex Session */
export const deleteCodexSession = async <T = UnknownRecord>(filePath: string): Promise<T> => {
  return invoke('codex_delete_session', { filePath })
}

// ── Codex Auth 管理 ──

/** 列出 Codex Auth 账号 */
export const listCodexAuthAccounts = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('codex_list_auth_accounts')
}

/** 获取 Codex Auth 当前账号 */
export const getCodexAuthCurrent = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('codex_get_auth_current')
}

/** 保存 Codex Auth */
export const saveCodexAuth = async <T = UnknownRecord>(data: unknown): Promise<T> => {
  return invoke('codex_save_auth', asRecord(data))
}

/** 切换 Codex Auth */
export const switchCodexAuth = async <T = UnknownRecord>(name: string): Promise<T> => {
  return invoke('codex_switch_auth', { name })
}

/** 删除 Codex Auth */
export const deleteCodexAuth = async <T = UnknownRecord>(name: string): Promise<T> => {
  return invoke('codex_delete_auth', { name })
}

/** 检测 Codex 进程 */
export const detectCodexProcess = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('codex_detect_process')
}

export interface CodexDashboardUsageSection {
  total_requests: number
  total_input_tokens: number
  total_output_tokens: number
}

export interface CodexDashboardUsageSummary {
  last_activity_at?: string | null
  freshness: 'fresh' | 'stale' | 'old' | 'empty'
  freshness_description: string
  five_hour: CodexDashboardUsageSection
  seven_day: CodexDashboardUsageSection
  all_time: CodexDashboardUsageSection
  top_model?: {
    model: string
    total_requests: number
    total_input_tokens: number
    total_output_tokens: number
    window_end?: string | null
  } | null
}

export interface CodexDashboardOverview {
  auth: {
    logged_in: boolean
    login_state?: string
    store?: string
    saved_accounts_total: number
    expired_accounts_total: number
    current?: {
      name?: string | null
      account_id?: string
      email?: string
      last_refresh?: string | null
      freshness?: string
      freshness_icon?: string
      freshness_description?: string
      expires_at?: string | null
      is_expired?: boolean
    } | null
  }
  profiles: {
    current_profile?: string | null
    total: number
    enabled_total: number
    disabled_total: number
    current?: UnknownRecord | null
  }
  config: {
    model?: string | null
    model_provider?: string | null
    approval_policy?: string | null
    sandbox_mode?: string | null
    model_reasoning_effort?: string | null
    model_reasoning_summary?: string | null
    web_search?: string | null
    disable_response_storage?: boolean | null
  }
  inventory: {
    mcp_servers_total: number
    agents_total: number
    sessions_total: number
    config_profiles_total: number
  }
}

export interface CodexCommandOptions {
  force?: boolean
}

/** 获取 Codex 仪表盘概览 */
export const getCodexDashboardOverview = async <T = CodexDashboardOverview>(
  options?: CodexCommandOptions,
): Promise<T> => {
  return invoke('codex_get_dashboard_overview', { force: options?.force })
}

/** 获取 Codex 仪表盘用量摘要 */
export const getCodexDashboardUsageSummary = async <T = CodexDashboardUsageSummary>(
  options?: CodexCommandOptions,
): Promise<T> => {
  return invoke('codex_get_dashboard_usage_summary', { force: options?.force })
}

export interface CodexUsageCommandOptions {
  force?: boolean
}

/** 获取 Codex 使用量 */
export const getCodexUsage = async <T = UnknownRecord>(
  options?: CodexUsageCommandOptions
): Promise<T> => {
  return invoke('codex_get_usage', { force: options?.force })
}

/** 获取所有 Codex 账号的配额余额 */
export const getCodexAllQuotas = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('codex_get_all_quotas')
}

/** 获取指定 Codex 账号的配额余额 */
export const getCodexQuota = async <T = UnknownRecord>(account: string): Promise<T> => {
  return invoke('codex_get_quota', { account })
}

/** 列出 Codex 斜杠命令（Codex 不支持） */
export const listCodexSlashCommands = async <T = UnknownRecord>(): Promise<T> => {
  return { commands: [], folders: [] } as T
}

/** 添加 Codex 斜杠命令（Codex 不支持） */
export const addCodexSlashCommand = async <T = UnknownRecord>(
  _name: string,
  _config: unknown
): Promise<T> => {
  return { success: false, message: 'Codex 平台不支持斜杠命令' } as T
}

/** 更新 Codex 斜杠命令（Codex 不支持） */
export const updateCodexSlashCommand = async <T = UnknownRecord>(
  _name: string,
  _config: unknown
): Promise<T> => {
  return { success: false, message: 'Codex 平台不支持斜杠命令' } as T
}

/** 删除 Codex 斜杠命令（Codex 不支持） */
export const deleteCodexSlashCommand = async <T = UnknownRecord>(_name: string): Promise<T> => {
  return { success: false, message: 'Codex 平台不支持斜杠命令' } as T
}

/** 切换 Codex 斜杠命令启用/禁用（Codex 不支持） */
export const toggleCodexSlashCommand = async <T = UnknownRecord>(
  _name: string,
  _enabled: boolean
): Promise<T> => {
  return { success: false, message: 'Codex 平台不支持斜杠命令' } as T
}

/** 列出 Codex 插件（Codex 不支持） */
export const listCodexPlugins = async <T = UnknownRecord>(): Promise<T> => {
  return { plugins: [] } as T
}

/** 添加 Codex 插件（Codex 不支持） */
export const addCodexPlugin = async <T = UnknownRecord>(
  _name: string,
  _config: unknown
): Promise<T> => {
  return { success: false, message: 'Codex 平台不支持插件' } as T
}

/** 更新 Codex 插件（Codex 不支持） */
export const updateCodexPlugin = async <T = UnknownRecord>(
  _pluginOrName: string | object,
  _config?: unknown
): Promise<T> => {
  return { success: false, message: 'Codex 平台不支持插件' } as T
}

/** 删除 Codex 插件（Codex 不支持） */
export const deleteCodexPlugin = async <T = UnknownRecord>(_name: string): Promise<T> => {
  return { success: false, message: 'Codex 平台不支持插件' } as T
}

/** 切换 Codex 插件启用/禁用（Codex 不支持） */
export const toggleCodexPlugin = async <T = UnknownRecord>(
  _name: string,
  _enabled: boolean
): Promise<T> => {
  return { success: false, message: 'Codex 平台不支持插件' } as T
}

// ════════════════════════════════════════════════════════════
// 6. Gemini 平台
// ════════════════════════════════════════════════════════════

/** 获取 Gemini 配置 */
export const getGeminiConfig = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('gemini_get_settings')
}

/** 更新 Gemini 配置 */
export const updateGeminiConfig = async <T = UnknownRecord>(settings: unknown): Promise<T> => {
  return invoke('gemini_update_settings', { settings })
}

/** 列出 Gemini MCP 服务器 */
export const listGeminiMcpServers = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('gemini_list_mcp_servers')
}

/** 添加 Gemini MCP 服务器 */
export const addGeminiMcpServer = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('gemini_add_mcp_server', { name, config: resolvedConfig })
}

/** 更新 Gemini MCP 服务器 */
export const updateGeminiMcpServer = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('gemini_update_mcp_server', { name, config: resolvedConfig })
}

/** 删除 Gemini MCP 服务器 */
export const deleteGeminiMcpServer = async <T = UnknownRecord>(
  nameOrRequest: string | object
): Promise<T> => {
  const name = resolveName(nameOrRequest)
  return invoke('gemini_delete_mcp_server', { name })
}

/** 列出 Gemini 斜杠命令 */
export const listGeminiSlashCommands = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('gemini_list_slash_commands')
}

/** 添加 Gemini 斜杠命令 */
export const addGeminiSlashCommand = async <T = UnknownRecord>(
  name: string,
  config: unknown
): Promise<T> => {
  return invoke('gemini_add_slash_command', { name, config })
}

/** 更新 Gemini 斜杠命令 */
export const updateGeminiSlashCommand = async <T = UnknownRecord>(
  name: string,
  config: unknown
): Promise<T> => {
  return invoke('gemini_update_slash_command', { name, config })
}

/** 删除 Gemini 斜杠命令 */
export const deleteGeminiSlashCommand = async <T = UnknownRecord>(name: string): Promise<T> => {
  return invoke('gemini_delete_slash_command', { name })
}

/** 切换 Gemini 斜杠命令启用/禁用 */
export const toggleGeminiSlashCommand = async <T = UnknownRecord>(
  name: string,
  enabled: boolean
): Promise<T> => {
  return invoke('gemini_update_slash_command', { name, config: { enabled } })
}

/** 列出 Gemini Extensions */
export const listGeminiExtensions = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('gemini_list_extensions')
}

// ── Gemini 平台限制 — 安全默认值 ──

/** 列出 Gemini Agents（暂不支持） */
export const listGeminiAgents = async <T = UnknownRecord>(): Promise<T> => {
  return { agents: [] } as T
}

/** 添加 Gemini Agent（暂不支持） */
export const addGeminiAgent = async <T = UnknownRecord>(
  _nameOrRequest: string | object,
  _config?: unknown
): Promise<T> => {
  return { success: false, message: 'Gemini 平台暂不支持 Agents' } as T
}

/** 更新 Gemini Agent（暂不支持） */
export const updateGeminiAgent = async <T = UnknownRecord>(
  _nameOrRequest: string | object,
  _config?: unknown
): Promise<T> => {
  return { success: false, message: 'Gemini 平台暂不支持 Agents' } as T
}

/** 删除 Gemini Agent（暂不支持） */
export const deleteGeminiAgent = async <T = UnknownRecord>(
  _nameOrRequest: string | object
): Promise<T> => {
  return { success: false, message: 'Gemini 平台暂不支持 Agents' } as T
}

/** 切换 Gemini Agent（暂不支持） */
export const toggleGeminiAgent = async <T = UnknownRecord>(
  _nameOrRequest: string | object,
  _enabled?: boolean
): Promise<T> => {
  return { success: false, message: 'Gemini 平台暂不支持 Agents' } as T
}

/** 列出 Gemini 插件（暂不支持） */
export const listGeminiPlugins = async <T = UnknownRecord>(): Promise<T> => {
  return { plugins: [] } as T
}

/** 添加 Gemini 插件（暂不支持） */
export const addGeminiPlugin = async <T = UnknownRecord>(
  _nameOrRequest: string | object,
  _config?: unknown
): Promise<T> => {
  return { success: false, message: 'Gemini 平台暂不支持 Plugins' } as T
}

/** 更新 Gemini 插件（暂不支持） */
export const updateGeminiPlugin = async <T = UnknownRecord>(
  _nameOrRequest: string | object,
  _config?: unknown
): Promise<T> => {
  return { success: false, message: 'Gemini 平台暂不支持 Plugins' } as T
}

/** 删除 Gemini 插件（暂不支持） */
export const deleteGeminiPlugin = async <T = UnknownRecord>(
  _nameOrRequest: string | object
): Promise<T> => {
  return { success: false, message: 'Gemini 平台暂不支持 Plugins' } as T
}

/** 切换 Gemini 插件（暂不支持） */
export const toggleGeminiPlugin = async <T = UnknownRecord>(
  _nameOrRequest: string | object,
  _enabled?: boolean
): Promise<T> => {
  return { success: false, message: 'Gemini 平台暂不支持 Plugins' } as T
}

// ════════════════════════════════════════════════════════════
// 7. Qwen 平台
// ════════════════════════════════════════════════════════════

/** 获取 Qwen 配置 */
export const getQwenConfig = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('qwen_get_settings')
}

/** 更新 Qwen 配置 */
export const updateQwenConfig = async <T = UnknownRecord>(settings: unknown): Promise<T> => {
  return invoke('qwen_update_settings', { settings })
}

/** 列出 Qwen MCP 服务器 */
export const listQwenMcpServers = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('qwen_list_mcp_servers')
}

/** 添加 Qwen MCP 服务器 */
export const addQwenMcpServer = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('qwen_add_mcp_server', { name, config: resolvedConfig })
}

/** 更新 Qwen MCP 服务器 */
export const updateQwenMcpServer = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('qwen_update_mcp_server', { name, config: resolvedConfig })
}

/** 删除 Qwen MCP 服务器 */
export const deleteQwenMcpServer = async <T = UnknownRecord>(
  nameOrRequest: string | object
): Promise<T> => {
  const name = resolveName(nameOrRequest)
  return invoke('qwen_delete_mcp_server', { name })
}

/** 列出 Qwen 斜杠命令 */
export const listQwenSlashCommands = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('qwen_list_slash_commands')
}

/** 添加 Qwen 斜杠命令 */
export const addQwenSlashCommand = async <T = UnknownRecord>(
  name: string,
  config: unknown
): Promise<T> => {
  return invoke('qwen_add_slash_command', { name, config })
}

/** 更新 Qwen 斜杠命令 */
export const updateQwenSlashCommand = async <T = UnknownRecord>(
  name: string,
  config: unknown
): Promise<T> => {
  return invoke('qwen_update_slash_command', { name, config })
}

/** 删除 Qwen 斜杠命令 */
export const deleteQwenSlashCommand = async <T = UnknownRecord>(name: string): Promise<T> => {
  return invoke('qwen_delete_slash_command', { name })
}

/** 切换 Qwen 斜杠命令启用/禁用 */
export const toggleQwenSlashCommand = async <T = UnknownRecord>(
  name: string,
  enabled: boolean
): Promise<T> => {
  return invoke('qwen_update_slash_command', { name, config: { enabled } })
}

// ── Qwen 平台限制 — 安全默认值 ──

/** 列出 Qwen Agents（暂不支持） */
export const listQwenAgents = async <T = UnknownRecord>(): Promise<T> => {
  return { agents: [] } as T
}

/** 添加 Qwen Agent（暂不支持） */
export const addQwenAgent = async <T = UnknownRecord>(
  _nameOrRequest: string | object,
  _config?: unknown
): Promise<T> => {
  return { success: false, message: 'Qwen 平台暂不支持 Agents' } as T
}

/** 更新 Qwen Agent（暂不支持） */
export const updateQwenAgent = async <T = UnknownRecord>(
  _nameOrRequest: string | object,
  _config?: unknown
): Promise<T> => {
  return { success: false, message: 'Qwen 平台暂不支持 Agents' } as T
}

/** 删除 Qwen Agent（暂不支持） */
export const deleteQwenAgent = async <T = UnknownRecord>(
  _nameOrRequest: string | object
): Promise<T> => {
  return { success: false, message: 'Qwen 平台暂不支持 Agents' } as T
}

/** 切换 Qwen Agent（暂不支持） */
export const toggleQwenAgent = async <T = UnknownRecord>(
  _nameOrRequest: string | object,
  _enabled?: boolean
): Promise<T> => {
  return { success: false, message: 'Qwen 平台暂不支持 Agents' } as T
}

/** 列出 Qwen 插件（暂不支持） */
export const listQwenPlugins = async <T = UnknownRecord>(): Promise<T> => {
  return { plugins: [] } as T
}

/** 添加 Qwen 插件（暂不支持） */
export const addQwenPlugin = async <T = UnknownRecord>(
  _nameOrRequest: string | object,
  _config?: unknown
): Promise<T> => {
  return { success: false, message: 'Qwen 平台暂不支持 Plugins' } as T
}

/** 更新 Qwen 插件（暂不支持） */
export const updateQwenPlugin = async <T = UnknownRecord>(
  _nameOrRequest: string | object,
  _config?: unknown
): Promise<T> => {
  return { success: false, message: 'Qwen 平台暂不支持 Plugins' } as T
}

/** 删除 Qwen 插件（暂不支持） */
export const deleteQwenPlugin = async <T = UnknownRecord>(
  _nameOrRequest: string | object
): Promise<T> => {
  return { success: false, message: 'Qwen 平台暂不支持 Plugins' } as T
}

/** 切换 Qwen 插件（暂不支持） */
export const toggleQwenPlugin = async <T = UnknownRecord>(
  _nameOrRequest: string | object,
  _enabled?: boolean
): Promise<T> => {
  return { success: false, message: 'Qwen 平台暂不支持 Plugins' } as T
}

// ════════════════════════════════════════════════════════════
// 8. Qoder 平台
// ════════════════════════════════════════════════════════════

/** 获取 Qoder 设置 */
export const getQoderConfig = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('qoder_get_settings')
}

/** 更新 Qoder 设置 */
export const updateQoderConfig = async <T = UnknownRecord>(settings: unknown): Promise<T> => {
  return invoke('qoder_update_settings', { settings })
}

/** 列出 Qoder MCP 服务器 */
export const listQoderMcpServers = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('qoder_list_mcp_servers')
}

/** 添加 Qoder MCP 服务器 */
export const addQoderMcpServer = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('qoder_add_mcp_server', { name, config: resolvedConfig })
}

/** 更新 Qoder MCP 服务器 */
export const updateQoderMcpServer = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('qoder_update_mcp_server', { name, config: resolvedConfig })
}

/** 删除 Qoder MCP 服务器 */
export const deleteQoderMcpServer = async <T = UnknownRecord>(
  nameOrRequest: string | object
): Promise<T> => {
  const name = resolveName(nameOrRequest)
  return invoke('qoder_delete_mcp_server', { name })
}

/** 列出 Qoder Commands */
export const listQoderCommands = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('qoder_list_commands')
}

/** 添加 Qoder Command */
export const addQoderCommand = async <T = UnknownRecord>(
  name: string,
  config: unknown
): Promise<T> => {
  return invoke('qoder_add_command', { name, config })
}

/** 更新 Qoder Command */
export const updateQoderCommand = async <T = UnknownRecord>(
  name: string,
  config: unknown
): Promise<T> => {
  return invoke('qoder_update_command', { name, config })
}

/** 删除 Qoder Command */
export const deleteQoderCommand = async <T = UnknownRecord>(name: string): Promise<T> => {
  return invoke('qoder_delete_command', { name })
}

/** 切换 Qoder Command 启用状态（Qoder Commands 无显式 enabled 状态，保持兼容 no-op） */
export const toggleQoderCommand = async <T = UnknownRecord>(
  _name: string,
  _enabled: boolean
): Promise<T> => {
  return { success: true, message: 'Qoder Commands 始终启用' } as T
}

/** 兼容旧命名：Qoder Slash Commands */
export const listQoderSlashCommands = listQoderCommands
export const addQoderSlashCommand = addQoderCommand
export const updateQoderSlashCommand = updateQoderCommand
export const deleteQoderSlashCommand = deleteQoderCommand
export const toggleQoderSlashCommand = toggleQoderCommand

/** 列出 Qoder Subagents */
export const listQoderAgents = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('qoder_list_agents')
}

/** 添加 Qoder Subagent */
export const addQoderAgent = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('qoder_add_agent', { name, config: resolvedConfig })
}

/** 更新 Qoder Subagent */
export const updateQoderAgent = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('qoder_update_agent', { name, config: resolvedConfig })
}

/** 删除 Qoder Subagent */
export const deleteQoderAgent = async <T = UnknownRecord>(
  nameOrRequest: string | object
): Promise<T> => {
  const name = resolveName(nameOrRequest)
  return invoke('qoder_delete_agent', { name })
}

/** 切换 Qoder Subagent（Qoder 不支持 enabled 状态，返回兼容错误） */
export const toggleQoderAgent = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  enabled?: boolean
): Promise<T> => {
  const name = resolveName(nameOrRequest)
  return invoke('qoder_toggle_agent', { name, enabled: enabled ?? false })
}

/** 列出 Qoder Hooks */
export const listQoderHooks = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('qoder_list_hooks')
}

/** 添加 Qoder Hook */
export const addQoderHook = async <T = UnknownRecord>(config: unknown): Promise<T> => {
  return invoke('qoder_add_hook', { config })
}

/** 更新 Qoder Hook */
export const updateQoderHook = async <T = UnknownRecord>(
  index: number,
  config: unknown
): Promise<T> => {
  return invoke('qoder_update_hook', { index, config })
}

/** 删除 Qoder Hook */
export const deleteQoderHook = async <T = UnknownRecord>(index: number): Promise<T> => {
  return invoke('qoder_delete_hook', { index })
}

// ── Qoder 未实现能力 — 安全默认值 ──

/** 列出 Qoder 插件（暂不支持） */
export const listQoderPlugins = async <T = UnknownRecord>(): Promise<T> => {
  return { plugins: [] } as T
}

/** 添加 Qoder 插件（暂不支持） */
export const addQoderPlugin = async <T = UnknownRecord>(
  _nameOrRequest: string | object,
  _config?: unknown
): Promise<T> => {
  return { success: false, message: 'Qoder 平台暂不支持 Plugins' } as T
}

/** 更新 Qoder 插件（暂不支持） */
export const updateQoderPlugin = async <T = UnknownRecord>(
  _nameOrRequest: string | object,
  _config?: unknown
): Promise<T> => {
  return { success: false, message: 'Qoder 平台暂不支持 Plugins' } as T
}

/** 删除 Qoder 插件（暂不支持） */
export const deleteQoderPlugin = async <T = UnknownRecord>(
  _nameOrRequest: string | object
): Promise<T> => {
  return { success: false, message: 'Qoder 平台暂不支持 Plugins' } as T
}

/** 切换 Qoder 插件（暂不支持） */
export const toggleQoderPlugin = async <T = UnknownRecord>(
  _nameOrRequest: string | object,
  _enabled?: boolean
): Promise<T> => {
  return { success: false, message: 'Qoder 平台暂不支持 Plugins' } as T
}

// ════════════════════════════════════════════════════════════
// 9. Droid 平台
// ════════════════════════════════════════════════════════════

/** 获取 Droid 设置 */
export const getDroidSettings = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('droid_get_settings')
}

/** 更新 Droid 设置 */
export const updateDroidSettings = async <T = UnknownRecord>(settings: unknown): Promise<T> => {
  return invoke('droid_update_settings', { settings })
}

/** 列出 Droid MCP 服务器 */
export const listDroidMcpServers = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('droid_list_mcp_servers')
}

/** 添加 Droid MCP 服务器 */
export const addDroidMcpServer = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('droid_add_mcp_server', { name, config: resolvedConfig })
}

/** 更新 Droid MCP 服务器 */
export const updateDroidMcpServer = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('droid_update_mcp_server', { name, config: resolvedConfig })
}

/** 删除 Droid MCP 服务器 */
export const deleteDroidMcpServer = async <T = UnknownRecord>(
  nameOrRequest: string | object
): Promise<T> => {
  const name = resolveName(nameOrRequest)
  return invoke('droid_delete_mcp_server', { name })
}

/** 列出 Droid Agents */
export const listDroidAgents = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('droid_list_agents')
}

/** 获取 Droid Agent 详情 */
export const getDroidAgent = async <T = UnknownRecord>(name: string): Promise<T> => {
  const agents = asRecord(await invoke<unknown>('droid_list_agents'))
  const target = asRecord(agents[name])
  if (Object.keys(target).length > 0) {
    return { name, ...target } as T
  }
  return null as T
}

/** 添加 Droid Agent */
export const addDroidAgent = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('droid_add_agent', { name, config: resolvedConfig })
}

/** 更新 Droid Agent */
export const updateDroidAgent = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('droid_update_agent', { name, config: resolvedConfig })
}

/** 删除 Droid Agent */
export const deleteDroidAgent = async <T = UnknownRecord>(
  nameOrRequest: string | object
): Promise<T> => {
  const name = resolveName(nameOrRequest)
  return invoke('droid_delete_agent', { name })
}

/** 列出 Droid 插件 */
export const listDroidPlugins = async (): Promise<DroidPlugin[]> => {
  const result = await invoke<unknown>('droid_list_plugins')
  if (Array.isArray(result)) {
    return result as DroidPlugin[]
  }
  const plugins = pickArray(result, 'plugins')
  if (plugins.length > 0) {
    return plugins as DroidPlugin[]
  }
  return []
}

/** 添加 Droid 插件 */
export const addDroidPlugin = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('droid_add_plugin', { name, config: resolvedConfig })
}

/** 更新 Droid 插件 */
export const updateDroidPlugin = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('droid_update_plugin', { name, config: resolvedConfig })
}

/** 删除 Droid 插件 */
export const deleteDroidPlugin = async <T = UnknownRecord>(
  nameOrRequest: string | object
): Promise<T> => {
  const name = resolveName(nameOrRequest)
  return invoke('droid_delete_plugin', { name })
}

/** 列出 Droid 斜杠命令 */
export const listDroidSlashCommands = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('droid_list_slash_commands')
}

/** 添加 Droid 斜杠命令 */
export const addDroidSlashCommand = async <T = UnknownRecord>(
  name: string,
  config: unknown
): Promise<T> => {
  return invoke('droid_add_slash_command', { name, config })
}

/** 更新 Droid 斜杠命令 */
export const updateDroidSlashCommand = async <T = UnknownRecord>(
  name: string,
  config: unknown
): Promise<T> => {
  return invoke('droid_update_slash_command', { name, config })
}

/** 删除 Droid 斜杠命令 */
export const deleteDroidSlashCommand = async <T = UnknownRecord>(name: string): Promise<T> => {
  return invoke('droid_delete_slash_command', { name })
}

/** 列出 Droid 模型 */
export const listDroidModels = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('droid_list_models')
}

/** 添加 Droid 模型 */
export const addDroidModel = async <T = UnknownRecord>(
  model: Record<string, unknown>
): Promise<T> => {
  const settings = await getDroidSettings<UnknownRecord>()
  const models = [...pickArray(settings, 'customModels')]

  if (models.some((item) => isRecord(item) && item.model === model.model)) {
    throw new Error(`模型 '${model?.model || ''}' 已存在`)
  }

  models.push(model)
  await updateDroidSettings({ customModels: models })
  return model as T
}

/** 更新 Droid 模型 */
export const updateDroidModel = async <T = UnknownRecord>(
  modelId: string,
  model: Record<string, unknown>
): Promise<T> => {
  const settings = await getDroidSettings<UnknownRecord>()
  const models = [...pickArray(settings, 'customModels')]
  const index = models.findIndex((item) => isRecord(item) && item.model === modelId)

  if (index === -1) {
    throw new Error(`模型 '${modelId}' 不存在`)
  }

  models[index] = { ...asRecord(models[index]), ...model }
  await updateDroidSettings({ customModels: models })
  return models[index] as T
}

/** 删除 Droid 模型 */
export const deleteDroidModel = async (modelId: string): Promise<string> => {
  const settings = await getDroidSettings<UnknownRecord>()
  const models = [...pickArray(settings, 'customModels')]
  const nextModels = models.filter((item) => !isRecord(item) || item.model !== modelId)

  if (nextModels.length === models.length) {
    throw new Error(`模型 '${modelId}' 不存在`)
  }

  await updateDroidSettings({ customModels: nextModels })
  return modelId
}

const normalizeDroidProfiles = (profiles: unknown): Record<string, unknown> => {
  if (Array.isArray(profiles)) {
    return profiles.reduce((acc: Record<string, unknown>, profile) => {
      if (isRecord(profile) && profile.name) {
        const profileName = String(profile.name)
        const profileData = { ...profile }
        delete profileData.name
        acc[profileName] = profileData
      }
      return acc
    }, {})
  }

  if (profiles && typeof profiles === 'object') {
    return { ...profiles }
  }

  return {}
}

/** 列出 Droid Profiles */
export const listDroidProfiles = async (): Promise<unknown[]> => {
  const settings = await getDroidSettings<UnknownRecord>()
  const profiles = normalizeDroidProfiles(settings?.profiles)

  return Object.entries(profiles).map(([name, config]) => ({
    name,
    ...(config as Record<string, unknown>),
  }))
}

/** 添加 Droid Profile */
export const addDroidProfile = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  const settings = await getDroidSettings<UnknownRecord>()
  const profiles = normalizeDroidProfiles(settings?.profiles)

  if (profiles[name]) {
    throw new Error(`Profile '${name}' 已存在`)
  }

  profiles[name] = resolvedConfig
  await updateDroidSettings({ profiles })
  return { name, ...resolvedConfig } as T
}

/** 更新 Droid Profile */
export const updateDroidProfile = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  const settings = await getDroidSettings<UnknownRecord>()
  const profiles = normalizeDroidProfiles(settings?.profiles)

  if (!profiles[name]) {
    throw new Error(`Profile '${name}' 不存在`)
  }

  profiles[name] = { ...asRecord(profiles[name]), ...resolvedConfig }
  await updateDroidSettings({ profiles })
  return { name, ...asRecord(profiles[name]) } as T
}

/** 删除 Droid Profile */
export const deleteDroidProfile = async (nameOrRequest: string | object): Promise<string> => {
  const name = resolveName(nameOrRequest)
  const settings = await getDroidSettings<UnknownRecord>()
  const profiles = normalizeDroidProfiles(settings?.profiles)

  if (!profiles[name]) {
    throw new Error(`Profile '${name}' 不存在`)
  }

  delete profiles[name]
  await updateDroidSettings({ profiles })
  return name
}

/** 切换 Droid Profile */
export const switchDroidProfile = async <T = UnknownRecord>(name: string): Promise<T> => {
  const settings = await getDroidSettings<UnknownRecord>()
  const profiles = normalizeDroidProfiles(settings?.profiles)

  if (!profiles[name]) {
    throw new Error(`Profile '${name}' 不存在`)
  }

  Object.keys(profiles).forEach((profileName) => {
    profiles[profileName] = {
      ...asRecord(profiles[profileName]),
      enabled: profileName === name,
    }
  })

  await updateDroidSettings({ profiles, currentProfile: name })
  return { name, ...asRecord(profiles[name]) } as T
}

// ════════════════════════════════════════════════════════════
// 10. OpenCode 平台
// ════════════════════════════════════════════════════════════

/** 获取 OpenCode 配置 */
export const getOpenCodeConfig = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('opencode_get_settings')
}

/** 更新 OpenCode 配置 */
export const updateOpenCodeConfig = async <T = UnknownRecord>(settings: unknown): Promise<T> => {
  return invoke('opencode_update_settings', { settings })
}

/** 获取 OpenCode 快捷键 */
export const getOpenCodeKeybindings = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('opencode_get_keybindings')
}

/** 更新 OpenCode 快捷键 */
export const updateOpenCodeKeybindings = async <T = UnknownRecord>(
  keybindings: unknown
): Promise<T> => {
  return invoke('opencode_update_keybindings', { keybindings })
}

/** 列出 OpenCode 主题 */
export const listOpenCodeThemes = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('opencode_list_themes')
}

// ── OpenCode 组合实现（通过 get/update settings） ──

/** 列出 OpenCode Providers */
export const listOpenCodeProviders = async <T = UnknownRecord>(): Promise<T> => {
  const settings = await getOpenCodeConfig<UnknownRecord>()
  const providers = pickRecord(settings, 'providers')
  return {
    providers: Object.entries(providers).map(([id, config]) => ({ id, ...asRecord(config) })),
  } as T
}

/** 添加 OpenCode Provider */
export const addOpenCodeProvider = async <T = UnknownRecord>(
  providerOrName: string | object,
  config?: unknown
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(providerOrName, config)
  const settings = await getOpenCodeConfig<UnknownRecord>()
  const providers = pickRecord(settings, 'providers')
  providers[name] = resolvedConfig
  await updateOpenCodeConfig({ providers })
  return { id: name, ...resolvedConfig } as T
}

/** 更新 OpenCode Provider */
export const updateOpenCodeProvider = async <T = UnknownRecord>(
  providerOrName: string | object,
  config?: unknown
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(providerOrName, config)
  const settings = await getOpenCodeConfig<UnknownRecord>()
  const providers = pickRecord(settings, 'providers')
  providers[name] = { ...asRecord(providers[name]), ...resolvedConfig }
  await updateOpenCodeConfig({ providers })
  return { id: name, ...asRecord(providers[name]) } as T
}

/** 删除 OpenCode Provider */
export const deleteOpenCodeProvider = async <T = UnknownRecord>(name: string): Promise<T> => {
  const settings = await getOpenCodeConfig<UnknownRecord>()
  const providers = pickRecord(settings, 'providers')
  delete providers[name]
  await updateOpenCodeConfig({ providers })
  return name as T
}

/** 列出 OpenCode MCP 服务器 */
export const listOpenCodeMcpServers = async <T = UnknownRecord>(): Promise<T> => {
  const settings = await getOpenCodeConfig<UnknownRecord>()
  const servers = asRecord(settings.mcpServers ?? settings.mcp_servers)
  return {
    servers: Object.entries(servers).map(([name, config]) => ({ name, ...asRecord(config) })),
  } as T
}

/** 添加 OpenCode MCP 服务器 */
export const addOpenCodeMcpServer = async <T = UnknownRecord>(
  serverOrName: string | object,
  config?: unknown
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(serverOrName, config)
  const settings = await getOpenCodeConfig<UnknownRecord>()
  const servers = asRecord(settings.mcpServers ?? settings.mcp_servers)
  servers[name] = resolvedConfig
  await updateOpenCodeConfig({ mcpServers: servers })
  return { name, ...resolvedConfig } as T
}

/** 更新 OpenCode MCP 服务器 */
export const updateOpenCodeMcpServer = async <T = UnknownRecord>(
  serverOrName: string | object,
  config?: unknown
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(serverOrName, config)
  const settings = await getOpenCodeConfig<UnknownRecord>()
  const servers = asRecord(settings.mcpServers ?? settings.mcp_servers)
  servers[name] = { ...asRecord(servers[name]), ...resolvedConfig }
  await updateOpenCodeConfig({ mcpServers: servers })
  return { name, ...asRecord(servers[name]) } as T
}

/** 删除 OpenCode MCP 服务器 */
export const deleteOpenCodeMcpServer = async <T = UnknownRecord>(name: string): Promise<T> => {
  const settings = await getOpenCodeConfig<UnknownRecord>()
  const servers = asRecord(settings.mcpServers ?? settings.mcp_servers)
  delete servers[name]
  await updateOpenCodeConfig({ mcpServers: servers })
  return name as T
}

/** 列出 OpenCode 插件 */
export const listOpenCodePlugins = async <T = UnknownRecord>(): Promise<T> => {
  const settings = await getOpenCodeConfig<UnknownRecord>()
  const plugins = pickRecord(settings, 'plugins')
  return {
    plugins: Object.entries(plugins).map(([name, config]) => ({ name, ...asRecord(config) })),
  } as T
}

/** 添加 OpenCode 插件 */
export const addOpenCodePlugin = async <T = UnknownRecord>(
  pluginOrName: string | object,
  config?: unknown
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(pluginOrName, config)
  const settings = await getOpenCodeConfig<UnknownRecord>()
  const plugins = pickRecord(settings, 'plugins')
  plugins[name] = resolvedConfig
  await updateOpenCodeConfig({ plugins })
  return { name, ...resolvedConfig } as T
}

/** 删除 OpenCode 插件 */
export const deleteOpenCodePlugin = async <T = UnknownRecord>(name: string): Promise<T> => {
  const settings = await getOpenCodeConfig<UnknownRecord>()
  const plugins = pickRecord(settings, 'plugins')
  delete plugins[name]
  await updateOpenCodeConfig({ plugins })
  return name as T
}

// ════════════════════════════════════════════════════════════
// 11. 签到 (CheckIn)
// ════════════════════════════════════════════════════════════

/** 列出签到 Provider */
export const listCheckinProviders = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('list_providers')
}

/** 获取签到 Provider 详情 */
export const getCheckinProvider = async <T = UnknownRecord>(id: string): Promise<T> => {
  const result = await invoke<unknown>('list_providers')
  const providers = Array.isArray(result) ? result : pickArray(result, 'providers')
  const found = providers.find((item) => isRecord(item) && String(item.id ?? '') === id)
  return (found ?? null) as T
}

/** 创建签到 Provider */
export const createCheckinProvider = async <T = UnknownRecord>(data: unknown): Promise<T> => {
  return invoke('add_provider', { data })
}

/** 更新签到 Provider */
export const updateCheckinProvider = async <T = UnknownRecord>(
  id: string,
  data: unknown
): Promise<T> => {
  return invoke('update_provider', { id, data })
}

/** 删除签到 Provider */
export const deleteCheckinProvider = async <T = UnknownRecord>(id: string): Promise<T> => {
  return invoke('delete_provider', { id })
}

/** 测试签到连接 */
export const testCheckinConnection = async <T = UnknownRecord>(id: string): Promise<T> => {
  return invoke('test_provider_connection', { id })
}

/** 列出签到账号 */
export const listCheckinAccounts = async <T = UnknownRecord>(providerId?: string): Promise<T> => {
  return invoke('list_accounts', { providerId })
}

/** 获取签到账号详情 */
export const getCheckinAccount = async <T = UnknownRecord>(id: string): Promise<T> => {
  const result = await invoke<unknown>('list_accounts', { providerId: null })
  const accounts = Array.isArray(result) ? result : pickArray(result, 'accounts')
  const found = accounts.find((item) => isRecord(item) && String(item.id ?? '') === id)
  return (found ?? null) as T
}

/** 获取签到账号仪表盘（完整 dashboard 数据：account + streak + calendar + trend） */
export const getCheckinAccountDashboard = async <T = UnknownRecord>(
  id: string,
  query?: { year?: number; month?: number; days?: number }
): Promise<T> => {
  return invoke('get_account_dashboard', {
    accountId: id,
    year: query?.year ?? null,
    month: query?.month ?? null,
    days: query?.days ?? null,
  })
}

/** 创建签到账号 */
export const createCheckinAccount = async <T = UnknownRecord>(data: unknown): Promise<T> => {
  return invoke('add_account', { data })
}

/** 更新签到账号 */
export const updateCheckinAccount = async <T = UnknownRecord>(
  id: string,
  data: unknown
): Promise<T> => {
  return invoke('update_account', { id, data })
}

/** 删除签到账号 */
export const deleteCheckinAccount = async <T = UnknownRecord>(id: string): Promise<T> => {
  return invoke('delete_account', { id })
}

/** 批量删除签到账号 */
export const batchDeleteAccounts = async <T = UnknownRecord>(ids: string[]): Promise<T> => {
  return invoke('batch_delete_accounts', { ids })
}

/** 执行签到 */
export const executeCheckin = async <T = UnknownRecord>(accountId: string): Promise<T> => {
  return invoke('execute_checkin', { accountId })
}

/** 签到（executeCheckin 的别名） */
export const checkinAccount = executeCheckin

/** 批量签到 */
export const batchCheckin = async <T = UnknownRecord>(accountIds: string[]): Promise<T> => {
  return invoke('batch_checkin', { accountIds })
}

export const startCheckinJob = async <T = UnknownRecord>(accountIds: string[]): Promise<T> => {
  return invoke('start_checkin_job', { accountIds })
}

export const getCheckinJobStatus = async <T = UnknownRecord>(jobId: string): Promise<T> => {
  return invoke('get_checkin_job_status', { jobId })
}

/** 查询签到余额 */
export const queryCheckinBalance = async <T = UnknownRecord>(accountId: string): Promise<T> => {
  return invoke('get_balance', { accountId })
}

/** 获取余额历史 */
export const getCheckinBalanceHistory = async <T = UnknownRecord>(
  accountId: string,
  days?: number
): Promise<T> => {
  return invoke('get_balance_history', { accountId, days })
}

/** 获取余额统计 */
export const getBalanceStats = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('get_balance_stats')
}

/** 列出签到记录 */
export const listCheckinRecords = async <T = UnknownRecord>(
  params?: number | { page?: number; page_size?: number; account_id?: string }
): Promise<T> => {
  if (typeof params === 'number') {
    return invoke('get_checkin_records', { accountId: null, limit: params })
  }

  const page = params?.page ?? 1
  const pageSize = params?.page_size ?? 20
  const accountId = params?.account_id ?? null
  const limit = pageSize

  return invoke('get_checkin_records', { accountId, limit, page })
}

/** 获取指定账号签到记录 */
export const getAccountCheckinRecords = async <T = UnknownRecord>(
  accountId: string,
  limit?: number
): Promise<T> => {
  return invoke('get_checkin_records', { accountId, limit })
}

/** 导出签到记录 */
export const exportCheckinRecords = async <T = UnknownRecord>(options: unknown): Promise<T> => {
  return invoke('export_checkin_data', { options })
}

/** 获取今日签到统计 */
export const getTodayCheckinStats = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('export_checkin_stats')
}

/** 执行 CDK 充值 */
export const executeCdkRecharge = async <T = UnknownRecord>(
  accountId: string,
  cdkCode: string
): Promise<T> => {
  return invoke('execute_cdk_recharge', { accountId, cdkCode })
}

/** 获取 CDK 历史 */
export const getCdkHistory = async <T = UnknownRecord>(accountId?: string): Promise<T> => {
  return invoke('get_cdk_history', { accountId })
}

/** 列出 WAF Cookies */
export const listWafCookies = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('list_waf_cookies')
}

/** 添加 WAF Cookie */
export const addWafCookie = async <T = UnknownRecord>(
  providerId: string,
  cookie: string
): Promise<T> => {
  return invoke('add_waf_cookie', { providerId, cookie })
}

/** 删除 WAF Cookie */
export const deleteWafCookie = async <T = UnknownRecord>(id: string): Promise<T> => {
  return invoke('delete_waf_cookie', { id })
}

// ── CheckIn 扩展 ──

/** 获取签到账号 Cookies */
export const getCheckinAccountCookies = async <T = UnknownRecord>(
  accountId: string
): Promise<T> => {
  return invoke('get_checkin_account_cookies', { accountId })
}

/** 导出签到配置 */
export const exportCheckinConfig = async <T = UnknownRecord>(
  options?: Record<string, unknown>
): Promise<T> => {
  return invoke('export_checkin_config', { options: options ?? null })
}

/** 预览签到导入 */
export const previewCheckinImport = async <T = UnknownRecord>(data: unknown): Promise<T> => {
  return invoke('preview_checkin_import', { data })
}

/** 导入签到配置 */
export const importCheckinConfig = async <T = UnknownRecord>(
  data: unknown,
  options?: unknown
): Promise<T> => {
  return invoke('import_checkin_config', { data, options: options ?? null })
}

/** 列出内置 Provider */
export const listBuiltinProviders = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('list_builtin_providers')
}

/** 添加内置 Provider */
export const addBuiltinProvider = async <T = UnknownRecord>(providerId: string): Promise<T> => {
  return invoke('add_builtin_provider', { providerId })
}

/** 获取 OAuth 授权链接（仅 HTTP 后端支持） */
export const getOAuthAuthorizeUrl = async (
  _request: OAuthAuthorizeUrlRequest
): Promise<OAuthAuthorizeUrlResponse> => {
  return {
    success: false,
    message: '[Tauri] getOAuthAuthorizeUrl: 仅 HTTP 后端支持',
  }
}

// ════════════════════════════════════════════════════════════
// 12. 统计 (Stats)
// ════════════════════════════════════════════════════════════

/** 获取费用概览 */
export const getCostOverview = async <T = UnknownRecord>(period?: string): Promise<T> => {
  return invoke('get_cost_overview', { period })
}

/** 获取热力图数据 */
export const getHeatmapData = async <T = UnknownRecord>(
  platform?: string,
  days?: number
): Promise<T> => {
  return invoke('get_heatmap_data', { platform, days })
}

/** V2: 获取使用量汇总 */
export const getUsageSummaryV2 = async <T = UnknownRecord>(
  platform?: string,
  startDate?: string,
  endDate?: string
): Promise<T> => {
  return invoke('get_usage_summary_v2', { platform, startDate, endDate })
}

/** V2: 获取每日趋势 */
export const getUsageTrendsV2 = async <T = UnknownRecord>(
  platform?: string,
  startDate?: string,
  endDate?: string
): Promise<T> => {
  return invoke('get_usage_trends_v2', { platform, startDate, endDate })
}

/** V2: 获取模型统计 */
export const getUsageByModelV2 = async <T = UnknownRecord>(
  platform?: string,
  startDate?: string,
  endDate?: string
): Promise<T> => {
  return invoke('get_usage_by_model_v2', { platform, startDate, endDate })
}

/** V2: 获取项目统计 */
export const getUsageByProjectV2 = async <T = UnknownRecord>(
  platform?: string,
  startDate?: string,
  endDate?: string
): Promise<T> => {
  return invoke('get_usage_by_project_v2', { platform, startDate, endDate })
}

/** V2: 获取热力图（兼容映射到现有命令） */
export const getUsageHeatmapV2 = async <T = UnknownRecord>(
  platform?: string,
  days?: number
): Promise<T> => {
  return invoke('get_usage_heatmap_v2', { platform, days })
}

/** V2: 获取日志 */
export interface UsageLogsQuery {
  platform?: string
  model?: string
  start_date?: string
  end_date?: string
  page?: number
  page_size?: number
  cursor?: string
  include_total?: boolean
  mode?: 'cursor' | 'offset'
}

export const getUsageLogsV2 = async <T = UnknownRecord>(
  platformOrQuery?: string | UsageLogsQuery,
  page?: number,
  pageSize?: number,
  model?: string,
  cursor?: string,
  includeTotal?: boolean,
  mode?: 'cursor' | 'offset'
): Promise<T> => {
  const query: UsageLogsQuery =
    typeof platformOrQuery === 'object'
      ? platformOrQuery
      : {
          platform: platformOrQuery,
          page,
          page_size: pageSize,
          model,
          cursor,
          include_total: includeTotal,
          mode,
        }
  return invoke('get_usage_logs_v2', { query })
}

/** V2: 获取仪表盘聚合 */
export const getUsageDashboardV2 = async <T = UnknownRecord>(
  platform?: string,
  startDate?: string,
  endDate?: string,
  heatmapDays?: number,
  includeHeatmap?: boolean
): Promise<T> => {
  return invoke('get_usage_dashboard_v2', {
    platform,
    startDate,
    endDate,
    heatmapDays,
    includeHeatmap,
  })
}

/** V2: 启动 usage 后台导入任务 */
export const startUsageImportJobV2 = async <T = UnknownRecord>(
  platform?: string,
  recentDays?: number,
  resetSources?: boolean
): Promise<T> => {
  return invoke('start_usage_import_job_v2', { platform, recentDays, resetSources })
}

/** V2: 查询 usage 后台导入任务状态 */
export const getUsageImportJobStatusV2 = async <T = UnknownRecord>(jobId: string): Promise<T> => {
  return invoke('get_usage_import_job_status_v2', { jobId })
}

/** V2: 导入单平台 usage */
export const importUsageV2 = async <T = UnknownRecord>(platform: string): Promise<T> => {
  return invoke('import_usage_v2', { platform })
}

/** V2: 导入全部 usage */
export const importAllUsageV2 = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('import_all_usage_v2')
}

/** V2: 首页工作区概览 */
export const getHomeUsageOverviewV2 = async <T = UnknownRecord>(days?: number): Promise<T> => {
  return invoke('get_home_usage_overview_v2', { days })
}

/** 获取会话统计 */
export const getSessionStats = async <T = UnknownRecord>(platform?: string): Promise<T> => {
  return invoke('get_session_stats', { platform })
}

// ── Stats 扩展 ──

/** 获取费用趋势 */
export const getCostTrend = async <T = UnknownRecord>(period?: string): Promise<T> => {
  return invoke('get_cost_trend', { period })
}

/** 按模型统计费用 */
export const getCostByModel = async <T = UnknownRecord>(_period?: string): Promise<T> => {
  return invoke('get_cost_by_model')
}

/** 按项目统计费用 */
export const getCostByProject = async <T = UnknownRecord>(_period?: string): Promise<T> => {
  return invoke('get_cost_by_project')
}

/** 获取提供商使用量 */
export const getProviderUsage = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('get_provider_usage')
}

/** 获取 Top Sessions */
export const getTopSessions = async <T = UnknownRecord>(limit?: number): Promise<T> => {
  return invoke('get_top_sessions', { limit })
}

/** 获取统计摘要 */
export const getStatsSummary = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('get_stats_summary')
}

/** 设置定价 */
export const setPricing = async <T = UnknownRecord>(data: unknown): Promise<T> => {
  const source = asRecord(data)
  const model = String(source.model ?? source.name ?? '')
  return invoke('set_pricing', { model, pricing: data })
}

/** 获取定价列表 */
export const getPricingList = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('get_pricing_list')
}

/** 移除定价 */
export const removePricing = async <T = UnknownRecord>(model: string): Promise<T> => {
  return invoke('remove_pricing', { model })
}

/** 重置定价 */
export const resetPricing = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('reset_pricing')
}

/** 获取每日统计 */
export const getDailyStats = async <T = UnknownRecord>(days?: number): Promise<T> => {
  return invoke('get_daily_stats', { days })
}

// ════════════════════════════════════════════════════════════
// 13. 系统 (System)
// ════════════════════════════════════════════════════════════

/** 获取系统信息 */
export const getSystemInfo = async <T = VersionInfoResponse>(): Promise<T> => {
  return invoke('get_system_info')
}

/** 检查版本更新 */
export const checkVersion = async <T = VersionInfoResponse>(): Promise<T> => {
  return invoke('check_version')
}

/** 健康检查 */
export const healthCheck = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('health_check')
}

/** 获取版本号（通过 check_version 实现） */
export const getVersion = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('check_version')
}

/** 检查更新（别名） */
export const checkUpdate = checkVersion

/** 执行 CCR 更新 */
export const updateCCR = async <T = UnknownRecord>(_branch?: string): Promise<T> => {
  return invoke('update_ccr')
}

export interface CliVersionsCommandOptions {
  mode?: 'fast' | 'full'
  timeoutMs?: number
  parallelism?: number
  // 兼容历史调用参数
  timeout?: number
}

export interface CliVersionCommandOptions {
  tool: string
  timeoutMs?: number
  force?: boolean
}

/** 获取 CLI 版本 */
export const getCliVersions = async <T = UnknownRecord>(
  options?: CliVersionsCommandOptions
): Promise<T> => {
  const normalizedOptions = options
    ? {
        mode: options.mode,
        timeoutMs: options.timeoutMs ?? options.timeout,
        parallelism: options.parallelism,
      }
    : undefined

  const raw = await invoke<unknown>('get_cli_versions', { options: normalizedOptions })
  if (!isRecord(raw)) {
    return raw as T
  }

  const entries = Array.isArray(raw.entries)
    ? raw.entries
    : Array.isArray(raw.versions)
      ? raw.versions
      : Object.entries(pickRecord(raw, 'versions')).map(([platform, value]) => {
          const text = String(value ?? '')
          if (!text || text === 'not found') {
            return {
              platform,
              installed: false,
              status: 'not_installed',
            }
          }
          return {
            platform,
            installed: true,
            version: text,
            status: 'ok',
          }
        })

  return {
    ...raw,
    versions: entries,
  } as T
}

/** 获取单个 CLI 版本 */
export const getCliVersion = async <T = UnknownRecord>(
  options: CliVersionCommandOptions
): Promise<T> => {
  const normalizedOptions = {
    tool: options.tool,
    timeoutMs: options.timeoutMs,
    force: options.force,
  }

  return invoke('get_cli_version', { options: normalizedOptions })
}

// ════════════════════════════════════════════════════════════
// 14. 转换器 (Converter)
// ════════════════════════════════════════════════════════════

/** 转换配置格式 */
export const convertConfig = async <T = UnknownRecord>(request: unknown): Promise<T> => {
  return invoke('convert_config', { request })
}

// ════════════════════════════════════════════════════════════
// 15. UI 状态 (Favorites / Recent Items)
// ════════════════════════════════════════════════════════════

/** 获取收藏列表 */
export const getFavorites = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('get_favorites')
}

/** 添加收藏 */
export const addFavorite = async <T = UnknownRecord>(
  command: string,
  args: string[],
  displayName: string | undefined,
  module: string
): Promise<T> => {
  return invoke('add_favorite', { command, args, displayName, module })
}

/** 移除收藏 */
export const removeFavorite = async <T = UnknownRecord>(id: string): Promise<T> => {
  return invoke('remove_favorite', { id })
}

/** 获取最近项目 */
export const getRecentItems = async <T = UnknownRecord>(limit?: number): Promise<T> => {
  return invoke('get_recent_items', { limit })
}

/** 添加最近项目 */
export const addRecentItem = async <T = UnknownRecord>(
  command: string,
  args: string[],
  success: boolean,
  durationMs: number
): Promise<T> => {
  return invoke('add_recent_item', { command, args, success, durationMs })
}

/** 清空最近项目 */
export const clearRecentItems = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('clear_recent_items')
}

// ════════════════════════════════════════════════════════════
// 16. WAF
// ════════════════════════════════════════════════════════════

/** 打开 WAF 登录窗口 */
export const openWafLogin = async <T = UnknownRecord>(
  loginUrl: string,
  providerId: string
): Promise<T> => {
  return invoke('open_waf_login', { loginUrl, providerId })
}

/** 获取 WAF Cookie 状态 */
export const getWafCookieStatus = async <T = UnknownRecord>(providerId: string): Promise<T> => {
  return invoke('get_waf_cookie_status', { providerId })
}

// ════════════════════════════════════════════════════════════
// 17. 统一 MCP (Unified MCP)
// ════════════════════════════════════════════════════════════

/** 列出所有平台的 MCP 服务器（统一视图） */
export const listUnifiedMcp = async <T = UnknownRecord>(
  platforms?: string[] | string
): Promise<T> => {
  const normalized = typeof platforms === 'string' ? [platforms] : platforms
  return invoke('unified_list_mcp_servers', { platforms: normalized })
}

/** 添加统一 MCP 服务器 */
export const addUnifiedMcp = async <T = UnknownRecord>(request: unknown): Promise<T> => {
  return invoke('unified_add_mcp_server', { request })
}

/** 更新统一 MCP 服务器（通过删除+添加实现） */
export const updateUnifiedMcp = async <T = UnknownRecord>(
  platformOrRequest: string | object,
  name?: string,
  request?: unknown
): Promise<T> => {
  const requestRecord = asRecord(request)
  const mergedRequest =
    typeof platformOrRequest === 'string'
      ? { ...requestRecord, platform: platformOrRequest, name }
      : asRecord(platformOrRequest)

  try {
    await invoke('unified_delete_mcp_server', {
      platform: mergedRequest.platform,
      name: mergedRequest.name,
    })
  } catch {
    // 删除失败则忽略（可能是新增）
  }

  return invoke('unified_add_mcp_server', { request: mergedRequest })
}

/** 删除统一 MCP 服务器 */
export const deleteUnifiedMcp = async <T = UnknownRecord>(
  platform: string,
  name: string
): Promise<T> => {
  return invoke('unified_delete_mcp_server', { platform, name })
}

/** 切换统一 MCP 服务器启用/禁用 */
export const toggleUnifiedMcp = async <T = UnknownRecord>(
  platform: string,
  name: string,
  disabled?: boolean
): Promise<T> => {
  if (platform === 'claude') {
    return invoke('claude_update_mcp_server', { name, config: { disabled: disabled ?? true } })
  }
  throw new Error(`[Tauri] toggleUnifiedMcp: 平台 ${platform} 不支持启用/禁用切换`)
}

// ════════════════════════════════════════════════════════════
// 18. 事件 (Events)
// ════════════════════════════════════════════════════════════

/** 获取最近事件 */
export const getRecentEvents = async <T = UnknownRecord>(count?: number): Promise<T> => {
  return invoke('get_recent_events', { count })
}

/** 获取运行时性能指标 */
export const getRuntimeMetrics = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('get_runtime_metrics')
}

// ════════════════════════════════════════════════════════════
// 19. 环境管理 (Environment)
// ════════════════════════════════════════════════════════════

/** 列出所有执行环境 */
export const listEnvironments = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('list_environments')
}

/** 获取当前活跃环境 */
export const getCurrentEnvironment = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('get_current_environment')
}

/** 切换活跃环境 */
export const switchEnvironment = async <T = UnknownRecord>(envId: string): Promise<T> => {
  return invoke('switch_environment', { envId })
}

/** 刷新环境列表 */
export const refreshEnvironments = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('refresh_environments')
}

/** 通过环境列出平台 */
export const envListPlatforms = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('env_list_platforms')
}

/** 通过环境检测 CLI */
export const envDetectCli = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('env_detect_cli')
}

// ── SSH 命令 ──

export interface SshHostConfig {
  id?: string
  name?: string
  host: string
  port?: number
  user?: string
  identity_file?: string
  remote_home?: string
}

export interface SshConnectionState {
  env_id: string
  connected: boolean
  has_password: boolean
  last_checked_at?: string | null
  last_error?: string | null
}

export interface SshFingerprintProbeResult {
  host: string
  port: number
  key_type: string
  fingerprint: string
  status: 'new' | 'matched' | 'mismatch'
  stored_fingerprint?: string | null
}

export const sshListHosts = async (): Promise<SshHostConfig[]> => {
  return invoke('ssh_list_hosts')
}

export const sshAddHost = async (host: SshHostConfig): Promise<SshHostConfig> => {
  return invoke('ssh_add_host', { host })
}

export const sshConnect = async (envId: string, password?: string): Promise<SshConnectionState> => {
  return invoke('ssh_connect', { envId, password })
}

export const sshReconnect = async (
  envId: string,
  password?: string
): Promise<SshConnectionState> => {
  return invoke('ssh_reconnect', { envId, password })
}

export const sshDisconnect = async (): Promise<SshConnectionState> => {
  return invoke('ssh_disconnect')
}

export const sshGetConnectionState = async (
  envId?: string
): Promise<SshConnectionState | SshConnectionState[]> => {
  return invoke('ssh_get_connection_state', { envId })
}

export const sshProbeHostFingerprint = async (
  envId?: string,
  host?: string,
  port?: number
): Promise<SshFingerprintProbeResult> => {
  return invoke('ssh_probe_host_fingerprint', { request: { env_id: envId, host, port } })
}

export const sshConfirmHostFingerprint = async (
  host: string,
  keyType: string,
  fingerprint: string,
  port?: number
): Promise<void> => {
  return invoke('ssh_confirm_host_fingerprint', {
    request: { host, key_type: keyType, fingerprint, port },
  })
}

export const sshReadConfig = async (
  envId: string,
  platform: string,
  path: string
): Promise<string> => {
  return invoke('ssh_read_config', { envId, platform, path })
}

export const sshWriteConfig = async (
  envId: string,
  platform: string,
  path: string,
  content: string,
  enableBackup = true
): Promise<void> => {
  return invoke('ssh_write_config', { envId, platform, path, content, enableBackup })
}

export const sshDetectCli = async <T = UnknownRecord>(envId: string): Promise<T> => {
  return invoke('ssh_detect_cli', { envId })
}

export interface SshConnectResult {
  success: boolean
  latency_ms: number
  error?: string | null
}

export const sshTestConnection = async (envId: string): Promise<SshConnectResult> => {
  return invoke('ssh_test_connection', { envId })
}

export interface SshKeyInfo {
  path: string
  key_type: string
  has_passphrase: boolean
  fingerprint?: string | null
}

export const sshListKeys = async (): Promise<SshKeyInfo[]> => {
  return invoke('ssh_list_keys')
}

// ════════════════════════════════════════════════════════════
// 20. HTTP-only 通用桩函数
// ════════════════════════════════════════════════════════════

/** 执行 CCR 命令 */
export const executeCommand = async (
  commandOrPayload: string | { command: string; args?: string[] },
  args?: string[]
): Promise<unknown> => {
  const command = typeof commandOrPayload === 'string' ? commandOrPayload : commandOrPayload.command
  const resolvedArgs = typeof commandOrPayload === 'string' ? args : commandOrPayload.args
  return invoke('execute_ccr_command', { command, args: resolvedArgs })
}

/** 列出可用命令 */
export const listCommands = async <T = UnknownRecord>(_client?: string): Promise<T> => {
  return invoke('list_ccr_commands')
}

/** 获取命令帮助 */
export const getCommandHelp = async <T = UnknownRecord>(command: string): Promise<T> => {
  return invoke('get_ccr_command_help', { command })
}

/** 启用配置（通过 switchConfig 实现） */
export const enableConfig = async <T = UnknownRecord>(name: string): Promise<T> => {
  return switchConfig(name)
}

/** 禁用配置（通过 update_config 设置 enabled=false） */
export const disableConfig = async <T = UnknownRecord>(name: string): Promise<T> => {
  return invoke('update_config', { name, data: { enabled: false } })
}

/** 获取单个配置详情（通过列表后过滤实现） */
export const getConfig = async <T = UnknownRecord>(name: string): Promise<T> => {
  const result = await invoke<unknown>('list_configs')
  const configs = Array.isArray(result) ? result : pickArray(result, 'configs')
  const found = configs.find((item) => isRecord(item) && String(item.name ?? '') === name)
  return (found ?? null) as T
}

/** 更新配置 */
export const updateConfig = async <T = UnknownRecord>(
  name: string,
  config: unknown
): Promise<T> => {
  return invoke('update_config', { name, data: config })
}

/** 清理备份 */
export const cleanBackups = async <T = UnknownRecord>(_days?: number): Promise<T> => {
  return invoke('clean_backups')
}

// ── MCP 预设 / 同步 / 内置提示词 ──

/** 列出 MCP 预设 */
export const listMcpPresets = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('list_mcp_presets')
}

/** 获取 MCP 预设详情 */
export const getMcpPreset = async <T = UnknownRecord>(id: string): Promise<T> => {
  return invoke('get_mcp_preset', { id })
}

/** 安装 MCP 预设 */
export const installMcpPreset = async <T = UnknownRecord>(
  presetIdOrRequest: string | object,
  platforms?: string[],
  env?: Record<string, string>
): Promise<T> => {
  const request = asRecord(presetIdOrRequest)
  const presetId =
    typeof presetIdOrRequest === 'string'
      ? presetIdOrRequest
      : String(request.preset_id ?? request.id ?? '')
  const envValue = env ?? request.env
  const envVars = isRecord(envValue)
    ? Object.entries(envValue).reduce<Record<string, string>>((acc, [key, value]) => {
        if (typeof value === 'string') {
          acc[key] = value
        }
        return acc
      }, {})
    : undefined
  return invoke('install_mcp_preset', { presetId, platforms, envVars })
}

/** 安装单个 MCP 预设 */
export const installMcpPresetSingle = async <T = UnknownRecord>(
  presetIdOrRequest: string | object,
  platform?: string,
  env?: Record<string, string>
): Promise<T> => {
  const request = asRecord(presetIdOrRequest)
  const presetId =
    typeof presetIdOrRequest === 'string'
      ? presetIdOrRequest
      : String(request.preset_id ?? request.id ?? '')
  const resolvedPlatform =
    platform ?? (typeof request.platform === 'string' ? request.platform : undefined)
  const envValue = env ?? request.env
  const envVars = isRecord(envValue)
    ? Object.entries(envValue).reduce<Record<string, string>>((acc, [key, value]) => {
        if (typeof value === 'string') {
          acc[key] = value
        }
        return acc
      }, {})
    : undefined
  return invoke('install_mcp_preset_single', { platform: resolvedPlatform, presetId, envVars })
}

/** 列出来源 MCP 服务器 */
export const listSourceMcpServers = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('list_source_mcp_servers')
}

/** 同步 MCP 服务器 */
export const syncMcpServer = async <T = UnknownRecord>(
  name: string,
  platforms?: string[]
): Promise<T> => {
  return invoke('sync_mcp_server', { name, platforms })
}

/** 同步所有 MCP 服务器 */
export const syncAllMcpServers = async <T = UnknownRecord>(platforms?: string[]): Promise<T> => {
  return invoke('sync_all_mcp_servers', { platforms })
}

/** 列出内置提示词 */
export const listBuiltinPrompts = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('list_builtin_prompts')
}

/** 获取内置提示词 */
export const getBuiltinPrompt = async <T = UnknownRecord>(id: string): Promise<T> => {
  return invoke('get_builtin_prompt', { id })
}

/** 按分类获取内置提示词 */
export const getBuiltinPromptsByCategory = async <T = UnknownRecord>(
  category: string
): Promise<T> => {
  return invoke('get_builtin_prompts_by_category', { category })
}

// ── Skills ──

/** 列出技能 */
export const listSkills = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('list_skills')
}

/** 添加技能 */
export const addSkill = async <T = UnknownRecord>(data: unknown): Promise<T> => {
  const payload = asRecord(data)
  const name = String(payload.name ?? '')
  const instruction = String(payload.instruction ?? payload.content ?? '')
  return invoke('add_skill', { name, instruction })
}

// ── Skills Domain ──

export const skillsInventory = async <T = UnknownRecord>(query?: unknown): Promise<T> => {
  return invoke('skills_inventory', { query: query ?? null })
}

export const skillsDetail = async <T = UnknownRecord>(skillId: string): Promise<T> => {
  return invoke('skills_detail', { skillId })
}

export const skillsContentGet = async <T = UnknownRecord>(
  skillId: string,
  installationId?: string | null
): Promise<T> => {
  return invoke('skills_content_get', { skillId, installationId: installationId ?? null })
}

export const skillsFilesList = async <T = UnknownRecord>(
  skillId: string,
  installationId?: string | null,
): Promise<T> => {
  return invoke('skills_files_list', { skillId, installationId: installationId ?? null })
}

export const skillsFileGet = async <T = UnknownRecord>(
  skillId: string,
  path: string,
  installationId?: string | null,
): Promise<T> => {
  return invoke('skills_file_get', { skillId, path, installationId: installationId ?? null })
}

export const skillsOnboardingCandidates = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('skills_onboarding_candidates')
}

export const skillsContentSave = async <T = UnknownRecord>(
  skillId: string,
  installationId: string,
  raw: string
): Promise<T> => {
  return invoke('skills_content_save', { skillId, installationId, raw })
}

export const skillsInstall = async <T = UnknownRecord>(request: unknown): Promise<T> => {
  return invoke('skills_install', { request })
}

export const skillsSync = async <T = UnknownRecord>(request: unknown): Promise<T> => {
  return invoke('skills_sync', { request })
}

export const skillsRemoveInstallation = async <T = UnknownRecord>(
  skillId: string,
  installationId: string
): Promise<T> => {
  return invoke('skills_remove_installation', { skillId, installationId })
}

export const skillsRemoveSkill = async <T = UnknownRecord>(skillId: string): Promise<T> => {
  return invoke('skills_remove_skill', { skillId })
}

export const skillsSourcesList = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('skills_sources_list')
}

export const skillsSourceAddGit = async <T = UnknownRecord>(url: string): Promise<T> => {
  return invoke('skills_source_add_git', { url })
}

export const skillsSourceAddLocal = async <T = UnknownRecord>(path: string): Promise<T> => {
  return invoke('skills_source_add_local', { path })
}

export const skillsSourceSync = async <T = UnknownRecord>(sourceId: string): Promise<T> => {
  return invoke('skills_source_sync', { sourceId })
}

export const skillsSourceRemove = async <T = UnknownRecord>(sourceId: string): Promise<T> => {
  return invoke('skills_source_remove', { sourceId })
}

export const skillsMarketplaceList = async <T = UnknownRecord>(
  query?: string | null,
  page = 1,
  pageSize = 20
): Promise<T> => {
  return invoke('skills_marketplace_list', { query: query ?? null, page, pageSize })
}

export const skillsMarketplaceDetail = async <T = UnknownRecord>(packageId: string): Promise<T> => {
  return invoke('skills_marketplace_detail', { packageId })
}

export const skillsNpxStatus = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('skills_npx_status')
}

export const skillsPickFolder = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('skills_pick_folder')
}

// Legacy Skills aliases kept temporarily so remaining components compile during the refactor.
export const deleteSkill = async <T = UnknownRecord>(skillId: string): Promise<T> => {
  return skillsRemoveSkill(skillId)
}

export const getSkillDetail = async <T = UnknownRecord>(skillId: string): Promise<T> => {
  return skillsDetail(skillId)
}

export const updateSkillContent = async <T = UnknownRecord>(
  skillId: string,
  raw: string
): Promise<T> => {
  const detail = asRecord(await skillsDetail(skillId))
  const installations = Array.isArray(detail.installations) ? detail.installations : []
  const installationId = String(asRecord(installations[0]).id ?? '')
  return skillsContentSave(skillId, installationId, raw)
}

export const listSkillRepositories = async <T = UnknownRecord>(): Promise<T> => {
  return skillsSourcesList()
}

export const addSkillRepository = async <T = UnknownRecord>(data: unknown): Promise<T> => {
  const payload = asRecord(data)
  if (typeof payload.url === 'string' && payload.url.trim()) {
    return skillsSourceAddGit(payload.url)
  }
  if (typeof payload.path === 'string' && payload.path.trim()) {
    return skillsSourceAddLocal(payload.path)
  }
  throw new Error('Repository url/path is required')
}

export const removeSkillRepository = async <T = UnknownRecord>(sourceId: string): Promise<T> => {
  return skillsSourceRemove(sourceId)
}

export const scanSkillRepository = async <T = UnknownRecord>(sourceId: string): Promise<T> => {
  return skillsSourceSync(sourceId)
}

export const getSkillHubTrending = async <T = UnknownRecord>(
  page = 1,
  pageSize = 20,
): Promise<T> => {
  return skillsMarketplaceList(null, page, pageSize)
}

export const searchSkillHubMarketplace = async <T = UnknownRecord>(
  query: string,
  page = 1,
  pageSize = 20,
): Promise<T> => {
  return skillsMarketplaceList(query, page, pageSize)
}

export const getSkillHubAgents = async <T = UnknownRecord>(): Promise<T> => {
  const inventory = asRecord(await skillsInventory())
  return (inventory.platforms ?? []) as T
}

export const getSkillHubAgentSkills = async <T = UnknownRecord>(agentName: string): Promise<T> => {
  return skillsInventory({ platform: agentName })
}

export const installSkillHubSkill = async <T = UnknownRecord>(data: unknown): Promise<T> => {
  const payload = asRecord(data)
  const agents = asArray(payload.agents).filter((value): value is string => typeof value === 'string')
  return skillsInstall({
    source_kind: 'marketplace',
    source_ref: String(payload.url ?? payload.package ?? ''),
    source_skill_id: typeof payload.skill === 'string' ? payload.skill : null,
    target_platforms: agents,
    force: Boolean(payload.force),
  })
}

export const removeSkillHubSkill = async <T = UnknownRecord>(skillId: string): Promise<T> => {
  return skillsRemoveSkill(skillId)
}

export const getSkillHubUnified = async <T = UnknownRecord>(platform?: string): Promise<T> => {
  return skillsInventory(platform ? { platform } : null)
}

export const getSkillHubSkillContent = async <T = UnknownRecord>(
  skillId: string,
  installationId?: string | null,
): Promise<T> => {
  return skillsContentGet(skillId, installationId ?? null)
}

export const saveSkillHubSkillContent = async <T = UnknownRecord>(
  skillId: string,
  installationIdOrContent: string,
  maybeContent?: string,
): Promise<T> => {
  if (maybeContent == null) {
    return updateSkillContent(skillId, installationIdOrContent)
  }

  return skillsContentSave(skillId, installationIdOrContent, maybeContent)
}

export const importSkillFromGithub = async <T = UnknownRecord>(
  url: string,
  agents: string[],
  force = false
): Promise<T> => {
  return skillsInstall({
    source_kind: 'github',
    source_ref: url,
    target_platforms: agents,
    force,
  })
}

export const importSkillFromLocal = async <T = UnknownRecord>(
  sourcePath: string,
  agents: string[],
  skillName?: string
): Promise<T> => {
  return skillsInstall({
    source_kind: 'local',
    source_ref: sourcePath,
    source_skill_id: skillName ?? null,
    target_platforms: agents,
    force: false,
  })
}

export const importSkillViaNpx = async <T = UnknownRecord>(
  packageName: string,
  agents: string[],
  global = false
): Promise<T> => {
  return skillsInstall({
    source_kind: 'npx',
    source_ref: packageName,
    target_platforms: agents,
    force: global,
  })
}

export const batchInstallSkills = async <T = UnknownRecord>(
  packages: string[],
  agents: string[],
  force = false
): Promise<T> => {
  const results = await Promise.all(
    packages.map((pkg) =>
      skillsInstall({
        source_kind: 'marketplace',
        source_ref: pkg,
        target_platforms: agents,
        force,
      })
    )
  )
  return {
    total: packages.length,
    success_count: results.filter((item) =>
      asArray(asRecord(item).results).every((row) => Boolean(asRecord(row).ok))
    ).length,
    fail_count: results.filter(
      (item) => !asArray(asRecord(item).results).every((row) => Boolean(asRecord(row).ok))
    ).length,
    results: results.flatMap((item) => asArray(asRecord(item).results)),
  } as T
}

export const checkNpxAvailability = async <T = UnknownRecord>(): Promise<T> => {
  return skillsNpxStatus()
}

export const browseForFolder = async <T = UnknownRecord>(): Promise<T> => {
  return skillsPickFolder()
}

// ── Axios / HTTP 核心（不再需要） ──

/** HTTP API 基础 URL（Tauri 模式返回空字符串） */
export const resolveApiBaseUrl = (): string => {
  return ''
}

/** 后端健康检查（使用 Tauri invoke 实现） */
export const getBackendHealth = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('health_check')
}
