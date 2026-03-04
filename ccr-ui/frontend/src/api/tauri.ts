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
 *   8. iFlow 平台
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

function resolveNameAndConfig(
  arg1: string | Record<string, any>,
  arg2?: Record<string, any>,
): { name: string; config: Record<string, any> } {
  if (typeof arg1 === 'string') {
    return { name: arg1, config: arg2 ?? {} }
  }

  const request = { ...arg1 }
  const name = String(request.name ?? request.id ?? '')
  delete request.name
  delete request.id

  return { name, config: request }
}

function resolveName(
  arg1: string | Record<string, any>,
): string {
  if (typeof arg1 === 'string') {
    return arg1
  }

  return String(arg1.name ?? arg1.id ?? '')
}


// ════════════════════════════════════════════════════════════
// 1. 环境检测 & 工具函数
// ════════════════════════════════════════════════════════════

/** 检查是否在 Tauri 桌面应用环境中运行 */
export const isTauriEnvironment = (): boolean => {
  return '__TAURI__' in window
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
export const listConfigs = async (): Promise<any> => {
  const configs = await invoke('list_configs')
  return { configs }
}

/** 切换到指定配置 */
export const switchConfig = async (name: string): Promise<any> => {
  return invoke('switch_config', { name })
}

/** 添加新配置（兼容 addConfig(name, config) 与 addConfig({name,...})） */
export const addConfig = async (nameOrData: string | Record<string, any>, config?: any): Promise<any> => {
  if (typeof nameOrData === 'string') {
    return invoke('add_config', { name: nameOrData, config })
  }
  const data = nameOrData || {}
  const { name, ...rest } = data
  return invoke('add_config', { name, config: rest })
}

/** 删除指定配置 */
export const deleteConfig = async (name: string): Promise<any> => {
  return invoke('delete_config', { name })
}

/** 重命名配置 */
export const renameConfig = async (oldName: string, newName: string): Promise<any> => {
  return invoke('rename_config', { oldName, newName })
}

/** 复制配置 */
export const duplicateConfig = async (name: string, newName: string): Promise<any> => {
  return invoke('duplicate_config', { name, newName })
}

/** 验证所有配置 */
export const validateConfigs = async (): Promise<any> => {
  return invoke('validate_configs')
}

/** 导入配置 */
export const importConfig = async (data: any): Promise<any> => {
  return invoke('import_config', { data })
}

/** 导出配置 */
export const exportConfig = async (name?: string): Promise<any> => {
  return invoke('export_config', { name })
}

/** 获取历史记录（包装为 { entries: [...] } 格式供前端消费） */
export const getHistory = async (limit?: number): Promise<any> => {
  const entries = await invoke('get_history', { limit: limit ?? 100 })
  return { entries }
}

/** 清理历史记录 */
export const clearHistory = async (): Promise<any> => {
  return invoke('clear_history')
}

// ════════════════════════════════════════════════════════════
// 3. 同步 (Sync / WebDAV)
// ════════════════════════════════════════════════════════════

/** 推送配置到远端 */
export const pushSync = async (force?: boolean): Promise<any> => {
  return invoke('sync_push', { force })
}

/** 从远端拉取配置 */
export const pullSync = async (force?: boolean): Promise<any> => {
  return invoke('sync_pull', { force })
}

/** 获取同步状态 */
export const getSyncStatus = async (): Promise<any> => {
  return invoke('sync_status')
}

/** getSyncInfo - 同 getSyncStatus 的别名 */
export const getSyncInfo = getSyncStatus

/** 列出同步文件夹 */
export const listSyncFolders = async (): Promise<any> => {
  return invoke('list_sync_folders')
}

/** 添加同步文件夹 */
export const addSyncFolder = async (
  name: string,
  localPath: string,
  remotePath: string,
): Promise<any> => {
  return invoke('add_sync_folder', { name, localPath, remotePath })
}

/** 更新同步文件夹 */
export const updateSyncFolder = async (
  id: string,
  name?: string,
  enabled?: boolean,
): Promise<any> => {
  return invoke('update_sync_folder', { id, name, enabled })
}

/** 删除同步文件夹 */
export const deleteSyncFolder = async (id: string): Promise<any> => {
  return invoke('delete_sync_folder', { id })
}

// ════════════════════════════════════════════════════════════
// 4. Claude Code 平台
// ════════════════════════════════════════════════════════════

// ── Claude Settings ──

/** 获取 Claude Code 全局设置 */
export const getClaudeSettings = async (): Promise<any> => {
  return invoke('claude_get_settings')
}

/** 更新 Claude Code 全局设置 */
export const updateClaudeSettings = async (settings: any): Promise<any> => {
  return invoke('claude_update_settings', { settings })
}

// ── Claude MCP Servers ──

/** 列出 Claude Code MCP 服务器 */
export const listMcpServers = async (): Promise<any> => {
  return invoke('claude_list_mcp_servers')
}

/** 添加 Claude Code MCP 服务器 */
export const addMcpServer = async (
  nameOrRequest: string | Record<string, any>,
  config?: any,
): Promise<any> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('claude_add_mcp_server', { name, config: resolvedConfig })
}

/** 更新 Claude Code MCP 服务器 */
export const updateMcpServer = async (
  nameOrRequest: string | Record<string, any>,
  config?: any,
): Promise<any> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('claude_update_mcp_server', { name, config: resolvedConfig })
}

/** 删除 Claude Code MCP 服务器 */
export const deleteMcpServer = async (nameOrRequest: string | Record<string, any>): Promise<any> => {
  const name = resolveName(nameOrRequest)
  return invoke('claude_delete_mcp_server', { name })
}

/** 切换 Claude Code MCP 服务器启用/禁用状态（通过更新 disabled 字段实现） */
export const toggleMcpServer = async (
  nameOrRequest: string | Record<string, any>,
  disabled?: boolean,
): Promise<any> => {
  if (typeof nameOrRequest === 'string') {
    return invoke('claude_update_mcp_server', { name: nameOrRequest, config: { disabled: !!disabled } })
  }
  const name = resolveName(nameOrRequest)
  const resolvedDisabled =
    typeof disabled === 'boolean'
      ? disabled
      : typeof nameOrRequest.disabled === 'boolean'
        ? nameOrRequest.disabled
        : true
  return invoke('claude_update_mcp_server', { name, config: { disabled: resolvedDisabled } })
}

// ── Claude Agents ──

/** 列出 Claude Code Agents */
export const listAgents = async (): Promise<any> => {
  return invoke('claude_list_agents')
}

/** 获取单个 Agent 详情（通过列表后过滤实现） */
export const getAgent = async (name: string): Promise<any> => {
  const result: any = await invoke('claude_list_agents')
  const agents = result?.agents || result || []
  return agents.find((a: any) => a.name === name) ?? null
}

/** 添加 Claude Code Agent */
export const addAgent = async (
  nameOrRequest: string | Record<string, any>,
  config?: any,
): Promise<any> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('claude_add_agent', { name, config: resolvedConfig })
}

/** 更新 Claude Code Agent */
export const updateAgent = async (
  nameOrRequest: string | Record<string, any>,
  config?: any,
): Promise<any> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('claude_update_agent', { name, config: resolvedConfig })
}

/** 删除 Claude Code Agent */
export const deleteAgent = async (nameOrRequest: string | Record<string, any>): Promise<any> => {
  const name = resolveName(nameOrRequest)
  return invoke('claude_delete_agent', { name })
}

/** 切换 Agent 启用/禁用状态 */
export const toggleAgent = async (
  nameOrRequest: string | Record<string, any>,
  enabled?: boolean,
): Promise<any> => {
  const name = resolveName(nameOrRequest)
  const resolvedEnabled =
    typeof enabled === 'boolean'
      ? enabled
      : typeof nameOrRequest === 'object' && typeof nameOrRequest.enabled === 'boolean'
        ? nameOrRequest.enabled
        : true
  return invoke('claude_update_agent', { name, config: { enabled: resolvedEnabled } })
}

// ── Claude Slash Commands ──

/** 列出 Claude Code 斜杠命令 */
export const listSlashCommands = async (): Promise<any> => {
  return invoke('claude_list_slash_commands')
}

/** 添加 Claude Code 斜杠命令 */
export const addSlashCommand = async (name: string, config: any): Promise<any> => {
  return invoke('claude_add_slash_command', { name, config })
}

/** 更新 Claude Code 斜杠命令 */
export const updateSlashCommand = async (name: string, config: any): Promise<any> => {
  return invoke('claude_update_slash_command', { name, config })
}

/** 删除 Claude Code 斜杠命令 */
export const deleteSlashCommand = async (name: string): Promise<any> => {
  return invoke('claude_delete_slash_command', { name })
}

/** 切换斜杠命令启用/禁用状态 */
export const toggleSlashCommand = async (name: string, enabled: boolean): Promise<any> => {
  return invoke('claude_update_slash_command', { name, config: { enabled } })
}

// ── Claude Plugins ──

/** 列出 Claude Code 插件 */
export const listPlugins = async (): Promise<any> => {
  return invoke('claude_list_plugins')
}

/** 添加 Claude Code 插件 */
export const addPlugin = async (
  nameOrRequest: string | Record<string, any>,
  config?: any,
): Promise<any> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('claude_add_plugin', { name, config: resolvedConfig })
}

/** 更新 Claude Code 插件 */
export const updatePlugin = async (name: string, config: any): Promise<any> => {
  return invoke('claude_update_plugin', { name, config })
}

/** 删除 Claude Code 插件 */
export const deletePlugin = async (name: string): Promise<any> => {
  return invoke('claude_delete_plugin', { name })
}

/** 切换插件启用/禁用状态 */
export const togglePlugin = async (
  nameOrRequest: string | Record<string, any>,
  enabled?: boolean,
): Promise<any> => {
  const name = resolveName(nameOrRequest)
  const resolvedEnabled =
    typeof enabled === 'boolean'
      ? enabled
      : typeof nameOrRequest === 'object' && typeof nameOrRequest.enabled === 'boolean'
        ? nameOrRequest.enabled
        : true
  return invoke('claude_update_plugin', { name, config: { enabled: resolvedEnabled } })
}

// ── Claude Output Styles ──

/** 获取输出样式列表 */
export const listOutputStyles = async (): Promise<any> => {
  return invoke('claude_get_output_styles')
}

/** 获取单个输出样式（别名） */
export const getOutputStyle = listOutputStyles

/** 创建输出样式（通过 update 实现） */
export const createOutputStyle = async (styles: any): Promise<any> => {
  return invoke('claude_update_output_styles', { styles })
}

/** 更新输出样式 */
export const updateOutputStyle = async (
  nameOrStyles: string | Record<string, any>,
  patch?: Record<string, any>,
): Promise<any> => {
  if (typeof nameOrStyles === 'string') {
    return invoke('claude_update_output_styles', { styles: { [nameOrStyles]: patch } })
  }
  return invoke('claude_update_output_styles', { styles: nameOrStyles })
}

/** 删除输出样式（通过 update 清空实现） */
export const deleteOutputStyle = async (_name: string): Promise<any> => {
  return invoke('claude_update_output_styles', { styles: {} })
}

// ── Claude Statusline ──

/** 获取状态栏配置 */
export const getStatusline = async (): Promise<any> => {
  return invoke('claude_get_statusline')
}

/** 更新状态栏配置 */
export const updateStatusline = async (statusline: any): Promise<any> => {
  return invoke('claude_update_statusline', { statusline })
}

// ── Claude Hooks ──

/** 列出 Claude Code Hooks */
export const listHooks = async (): Promise<any> => {
  return invoke('claude_list_hooks')
}

/** 更新 Claude Code Hooks（批量更新） */
export const updateHooks = async (hooks: any): Promise<any> => {
  return invoke('claude_update_hooks', { hooks })
}

/** 添加单个 Hook（通过读取后合并实现） */
export const addHook = async (
  nameOrConfig: string | Record<string, any>,
  config?: any,
): Promise<any> => {
  const payload = typeof nameOrConfig === 'string' ? { [nameOrConfig]: config } : nameOrConfig
  const current: any = await invoke('claude_list_hooks')
  const merged = { ...current, ...payload }
  return invoke('claude_update_hooks', { hooks: merged })
}

/** 更新单个 Hook（通过读取后合并实现） */
export const updateHook = async (
  nameOrConfig: string | Record<string, any>,
  config?: any,
): Promise<any> => {
  const payload = typeof nameOrConfig === 'string' ? { [nameOrConfig]: config } : nameOrConfig
  const current: any = await invoke('claude_list_hooks')
  const merged = { ...current, ...payload }
  return invoke('claude_update_hooks', { hooks: merged })
}

/** 删除单个 Hook */
export const deleteHook = async (name: string): Promise<any> => {
  const current: any = await invoke('claude_list_hooks')
  if (current && typeof current === 'object') {
    delete current[name]
  }
  return invoke('claude_update_hooks', { hooks: current })
}

/** 切换 Hook 启用/禁用状态 */
export const toggleHook = async (
  nameOrRequest: string | Record<string, any>,
  enabled?: boolean,
): Promise<any> => {
  const name = resolveName(nameOrRequest)
  const resolvedEnabled =
    typeof enabled === 'boolean'
      ? enabled
      : typeof nameOrRequest === 'object' && typeof nameOrRequest.enabled === 'boolean'
        ? nameOrRequest.enabled
        : true
  const current: any = await invoke('claude_list_hooks')
  if (current && current[name]) {
    current[name].enabled = resolvedEnabled
  }
  return invoke('claude_update_hooks', { hooks: current })
}

// ── Claude Budgets ──

/** 获取预算配置 */
export const getBudgetStatus = async (): Promise<any> => {
  return invoke('claude_get_budgets')
}

/** 设置预算 */
export const setBudget = async (budgets: any): Promise<any> => {
  return invoke('claude_update_budgets', { budgets })
}

/** 重置预算 */
export const resetBudget = async (): Promise<any> => {
  return invoke('claude_update_budgets', { budgets: {} })
}

// ── Claude Prompts ──

/** 列出提示词 */
export const listPrompts = async (): Promise<any> => {
  return invoke('claude_list_prompts')
}

/** 更新提示词 */
export const updatePrompts = async (prompts: any): Promise<any> => {
  return invoke('claude_update_prompts', { prompts })
}

// ════════════════════════════════════════════════════════════
// 5. Codex 平台
// ════════════════════════════════════════════════════════════

/** 列出 Codex Profiles */
export const listCodexProfiles = async (): Promise<any> => {
  return invoke('codex_list_profiles')
}

/** 获取 Codex 配置 */
export const getCodexConfig = async (): Promise<any> => {
  return invoke('codex_get_settings')
}

/** 更新 Codex 配置 */
export const updateCodexConfig = async (settings: any): Promise<any> => {
  return invoke('codex_update_settings', { settings })
}

/** 列出 Codex MCP 服务器 */
export const listCodexMcpServers = async (): Promise<any> => {
  return invoke('codex_list_mcp_servers')
}

/** 添加 Codex MCP 服务器 */
export const addCodexMcpServer = async (
  nameOrRequest: string | Record<string, any>,
  config?: any,
): Promise<any> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('codex_add_mcp_server', { name, config: resolvedConfig })
}

/** 更新 Codex MCP 服务器 */
export const updateCodexMcpServer = async (
  nameOrRequest: string | Record<string, any>,
  config?: any,
): Promise<any> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('codex_update_mcp_server', { name, config: resolvedConfig })
}

/** 删除 Codex MCP 服务器 */
export const deleteCodexMcpServer = async (nameOrRequest: string | Record<string, any>): Promise<any> => {
  const name = resolveName(nameOrRequest)
  return invoke('codex_delete_mcp_server', { name })
}

/** 列出 Codex Agents */
export const listCodexAgents = async (): Promise<any> => {
  return invoke('codex_list_agents')
}

/** 添加 Codex Agent */
export const addCodexAgent = async (
  nameOrRequest: string | Record<string, any>,
  config?: any,
): Promise<any> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('codex_add_agent', { name, config: resolvedConfig })
}

/** 更新 Codex Agent */
export const updateCodexAgent = async (
  nameOrRequest: string | Record<string, any>,
  config?: any,
): Promise<any> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('codex_update_agent', { name, config: resolvedConfig })
}

/** 删除 Codex Agent */
export const deleteCodexAgent = async (nameOrRequest: string | Record<string, any>): Promise<any> => {
  const name = resolveName(nameOrRequest)
  return invoke('codex_delete_agent', { name })
}

/** 切换 Codex Agent 启用/禁用状态 */
export const toggleCodexAgent = async (
  nameOrRequest: string | Record<string, any>,
  enabled?: boolean,
): Promise<any> => {
  const name = resolveName(nameOrRequest)
  const resolvedEnabled =
    typeof enabled === 'boolean'
      ? enabled
      : typeof nameOrRequest === 'object' && typeof nameOrRequest.enabled === 'boolean'
        ? nameOrRequest.enabled
        : true
  return invoke('codex_update_agent', { name, config: { enabled: resolvedEnabled } })
}

// ── Codex Profile 组合实现（通过 get/update settings） ──

/** 添加 Codex Profile */
export const addCodexProfile = async (
  profileOrName: string | Record<string, any>,
  config?: any,
): Promise<any> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(profileOrName, config)
  const settings = await getCodexConfig()
  const profiles = settings?.profiles ?? {}
  if (profiles[name]) throw new Error(`Profile '${name}' 已存在`)
  profiles[name] = resolvedConfig
  await updateCodexConfig({ profiles })
  return { name, ...resolvedConfig }
}

/** 更新 Codex Profile */
export const updateCodexProfile = async (
  profileOrName: string | Record<string, any>,
  config?: any,
): Promise<any> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(profileOrName, config)
  const settings = await getCodexConfig()
  const profiles = settings?.profiles ?? {}
  if (!profiles[name]) throw new Error(`Profile '${name}' 不存在`)
  profiles[name] = { ...profiles[name], ...resolvedConfig }
  await updateCodexConfig({ profiles })
  return { name, ...profiles[name] }
}

/** 删除 Codex Profile */
export const deleteCodexProfile = async (nameOrRequest: string | Record<string, any>): Promise<any> => {
  const name = resolveName(nameOrRequest)
  const settings = await getCodexConfig()
  const profiles = settings?.profiles ?? {}
  if (!profiles[name]) throw new Error(`Profile '${name}' 不存在`)
  delete profiles[name]
  await updateCodexConfig({ profiles })
  return name
}

/** 获取 Codex Profile 详情 */
export const getCodexProfile = async (name: string): Promise<any> => {
  const profiles = await listCodexProfiles()
  const arr = Array.isArray(profiles) ? profiles : profiles?.profiles ?? []
  return arr.find((p: any) => p.name === name) ?? null
}

/** 应用 Codex Profile */
export const applyCodexProfile = async (name: string): Promise<any> => {
  const settings = await getCodexConfig()
  const profiles = settings?.profiles ?? {}
  if (!profiles[name]) throw new Error(`Profile '${name}' 不存在`)
  Object.keys(profiles).forEach((k) => {
    profiles[k] = { ...profiles[k], enabled: k === name }
  })
  await updateCodexConfig({ profiles, currentProfile: name })
  return { name, ...profiles[name] }
}

// ── Codex Auth 管理 ──

/** 列出 Codex Auth 账号 */
export const listCodexAuthAccounts = async (): Promise<any> => {
  return invoke('codex_list_auth_accounts')
}

/** 获取 Codex Auth 当前账号 */
export const getCodexAuthCurrent = async (): Promise<any> => {
  return invoke('codex_get_auth_current')
}

/** 保存 Codex Auth */
export const saveCodexAuth = async (data: any): Promise<any> => {
  return invoke('codex_save_auth', data)
}

/** 切换 Codex Auth */
export const switchCodexAuth = async (name: string): Promise<any> => {
  return invoke('codex_switch_auth', { name })
}

/** 删除 Codex Auth */
export const deleteCodexAuth = async (name: string): Promise<any> => {
  return invoke('codex_delete_auth', { name })
}

/** 检测 Codex 进程 */
export const detectCodexProcess = async (): Promise<any> => {
  return invoke('codex_detect_process')
}

/** 获取 Codex 使用量 */
export const getCodexUsage = async (): Promise<any> => {
  return invoke('codex_get_usage')
}

/** 列出 Codex 斜杠命令（Codex 不支持） */
export const listCodexSlashCommands = async (): Promise<any> => {
  return { commands: [], folders: [] }
}

/** 添加 Codex 斜杠命令（Codex 不支持） */
export const addCodexSlashCommand = async (_name: string, _config: any): Promise<any> => {
  return { success: false, message: 'Codex 平台不支持斜杠命令' }
}

/** 更新 Codex 斜杠命令（Codex 不支持） */
export const updateCodexSlashCommand = async (_name: string, _config: any): Promise<any> => {
  return { success: false, message: 'Codex 平台不支持斜杠命令' }
}

/** 删除 Codex 斜杠命令（Codex 不支持） */
export const deleteCodexSlashCommand = async (_name: string): Promise<any> => {
  return { success: false, message: 'Codex 平台不支持斜杠命令' }
}

/** 切换 Codex 斜杠命令启用/禁用（Codex 不支持） */
export const toggleCodexSlashCommand = async (
  _name: string,
  _enabled: boolean,
): Promise<any> => {
  return { success: false, message: 'Codex 平台不支持斜杠命令' }
}

/** 列出 Codex 插件（Codex 不支持） */
export const listCodexPlugins = async (): Promise<any> => {
  return { plugins: [] }
}

/** 添加 Codex 插件（Codex 不支持） */
export const addCodexPlugin = async (_name: string, _config: any): Promise<any> => {
  return { success: false, message: 'Codex 平台不支持插件' }
}

/** 更新 Codex 插件（Codex 不支持） */
export const updateCodexPlugin = async (
  _pluginOrName: string | Record<string, any>,
  _config?: any,
): Promise<any> => {
  return { success: false, message: 'Codex 平台不支持插件' }
}

/** 删除 Codex 插件（Codex 不支持） */
export const deleteCodexPlugin = async (_name: string): Promise<any> => {
  return { success: false, message: 'Codex 平台不支持插件' }
}

/** 切换 Codex 插件启用/禁用（Codex 不支持） */
export const toggleCodexPlugin = async (_name: string, _enabled: boolean): Promise<any> => {
  return { success: false, message: 'Codex 平台不支持插件' }
}

// ════════════════════════════════════════════════════════════
// 6. Gemini 平台
// ════════════════════════════════════════════════════════════

/** 获取 Gemini 配置 */
export const getGeminiConfig = async (): Promise<any> => {
  return invoke('gemini_get_settings')
}

/** 更新 Gemini 配置 */
export const updateGeminiConfig = async (settings: any): Promise<any> => {
  return invoke('gemini_update_settings', { settings })
}

/** 列出 Gemini MCP 服务器 */
export const listGeminiMcpServers = async (): Promise<any> => {
  return invoke('gemini_list_mcp_servers')
}

/** 添加 Gemini MCP 服务器 */
export const addGeminiMcpServer = async (
  nameOrRequest: string | Record<string, any>,
  config?: any,
): Promise<any> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('gemini_add_mcp_server', { name, config: resolvedConfig })
}

/** 更新 Gemini MCP 服务器 */
export const updateGeminiMcpServer = async (
  nameOrRequest: string | Record<string, any>,
  config?: any,
): Promise<any> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('gemini_update_mcp_server', { name, config: resolvedConfig })
}

/** 删除 Gemini MCP 服务器 */
export const deleteGeminiMcpServer = async (nameOrRequest: string | Record<string, any>): Promise<any> => {
  const name = resolveName(nameOrRequest)
  return invoke('gemini_delete_mcp_server', { name })
}

/** 列出 Gemini 斜杠命令 */
export const listGeminiSlashCommands = async (): Promise<any> => {
  return invoke('gemini_list_slash_commands')
}

/** 添加 Gemini 斜杠命令 */
export const addGeminiSlashCommand = async (name: string, config: any): Promise<any> => {
  return invoke('gemini_add_slash_command', { name, config })
}

/** 更新 Gemini 斜杠命令 */
export const updateGeminiSlashCommand = async (name: string, config: any): Promise<any> => {
  return invoke('gemini_update_slash_command', { name, config })
}

/** 删除 Gemini 斜杠命令 */
export const deleteGeminiSlashCommand = async (name: string): Promise<any> => {
  return invoke('gemini_delete_slash_command', { name })
}

/** 切换 Gemini 斜杠命令启用/禁用 */
export const toggleGeminiSlashCommand = async (
  name: string,
  enabled: boolean,
): Promise<any> => {
  return invoke('gemini_update_slash_command', { name, config: { enabled } })
}

/** 列出 Gemini Extensions */
export const listGeminiExtensions = async (): Promise<any> => {
  return invoke('gemini_list_extensions')
}

// ── Gemini 平台限制 — 安全默认值 ──

/** 列出 Gemini Agents（暂不支持） */
export const listGeminiAgents = async (): Promise<any> => {
  return { agents: [] }
}

/** 添加 Gemini Agent（暂不支持） */
export const addGeminiAgent = async (
  _nameOrRequest: string | Record<string, any>,
  _config?: any,
): Promise<any> => {
  return { success: false, message: 'Gemini 平台暂不支持 Agents' }
}

/** 更新 Gemini Agent（暂不支持） */
export const updateGeminiAgent = async (
  _nameOrRequest: string | Record<string, any>,
  _config?: any,
): Promise<any> => {
  return { success: false, message: 'Gemini 平台暂不支持 Agents' }
}

/** 删除 Gemini Agent（暂不支持） */
export const deleteGeminiAgent = async (_nameOrRequest: string | Record<string, any>): Promise<any> => {
  return { success: false, message: 'Gemini 平台暂不支持 Agents' }
}

/** 切换 Gemini Agent（暂不支持） */
export const toggleGeminiAgent = async (
  _nameOrRequest: string | Record<string, any>,
  _enabled?: boolean,
): Promise<any> => {
  return { success: false, message: 'Gemini 平台暂不支持 Agents' }
}

/** 列出 Gemini 插件（暂不支持） */
export const listGeminiPlugins = async (): Promise<any> => {
  return { plugins: [] }
}

/** 添加 Gemini 插件（暂不支持） */
export const addGeminiPlugin = async (
  _nameOrRequest: string | Record<string, any>,
  _config?: any,
): Promise<any> => {
  return { success: false, message: 'Gemini 平台暂不支持 Plugins' }
}

/** 更新 Gemini 插件（暂不支持） */
export const updateGeminiPlugin = async (
  _nameOrRequest: string | Record<string, any>,
  _config?: any,
): Promise<any> => {
  return { success: false, message: 'Gemini 平台暂不支持 Plugins' }
}

/** 删除 Gemini 插件（暂不支持） */
export const deleteGeminiPlugin = async (_nameOrRequest: string | Record<string, any>): Promise<any> => {
  return { success: false, message: 'Gemini 平台暂不支持 Plugins' }
}

/** 切换 Gemini 插件（暂不支持） */
export const toggleGeminiPlugin = async (
  _nameOrRequest: string | Record<string, any>,
  _enabled?: boolean,
): Promise<any> => {
  return { success: false, message: 'Gemini 平台暂不支持 Plugins' }
}

// ════════════════════════════════════════════════════════════
// 7. Qwen 平台
// ════════════════════════════════════════════════════════════

/** 获取 Qwen 配置 */
export const getQwenConfig = async (): Promise<any> => {
  return invoke('qwen_get_settings')
}

/** 更新 Qwen 配置 */
export const updateQwenConfig = async (settings: any): Promise<any> => {
  return invoke('qwen_update_settings', { settings })
}

/** 列出 Qwen MCP 服务器 */
export const listQwenMcpServers = async (): Promise<any> => {
  return invoke('qwen_list_mcp_servers')
}

/** 添加 Qwen MCP 服务器 */
export const addQwenMcpServer = async (
  nameOrRequest: string | Record<string, any>,
  config?: any,
): Promise<any> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('qwen_add_mcp_server', { name, config: resolvedConfig })
}

/** 更新 Qwen MCP 服务器 */
export const updateQwenMcpServer = async (
  nameOrRequest: string | Record<string, any>,
  config?: any,
): Promise<any> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('qwen_update_mcp_server', { name, config: resolvedConfig })
}

/** 删除 Qwen MCP 服务器 */
export const deleteQwenMcpServer = async (nameOrRequest: string | Record<string, any>): Promise<any> => {
  const name = resolveName(nameOrRequest)
  return invoke('qwen_delete_mcp_server', { name })
}

/** 列出 Qwen 斜杠命令 */
export const listQwenSlashCommands = async (): Promise<any> => {
  return invoke('qwen_list_slash_commands')
}

/** 添加 Qwen 斜杠命令 */
export const addQwenSlashCommand = async (name: string, config: any): Promise<any> => {
  return invoke('qwen_add_slash_command', { name, config })
}

/** 更新 Qwen 斜杠命令 */
export const updateQwenSlashCommand = async (name: string, config: any): Promise<any> => {
  return invoke('qwen_update_slash_command', { name, config })
}

/** 删除 Qwen 斜杠命令 */
export const deleteQwenSlashCommand = async (name: string): Promise<any> => {
  return invoke('qwen_delete_slash_command', { name })
}

/** 切换 Qwen 斜杠命令启用/禁用 */
export const toggleQwenSlashCommand = async (
  name: string,
  enabled: boolean,
): Promise<any> => {
  return invoke('qwen_update_slash_command', { name, config: { enabled } })
}

// ── Qwen 平台限制 — 安全默认值 ──

/** 列出 Qwen Agents（暂不支持） */
export const listQwenAgents = async (): Promise<any> => {
  return { agents: [] }
}

/** 添加 Qwen Agent（暂不支持） */
export const addQwenAgent = async (
  _nameOrRequest: string | Record<string, any>,
  _config?: any,
): Promise<any> => {
  return { success: false, message: 'Qwen 平台暂不支持 Agents' }
}

/** 更新 Qwen Agent（暂不支持） */
export const updateQwenAgent = async (
  _nameOrRequest: string | Record<string, any>,
  _config?: any,
): Promise<any> => {
  return { success: false, message: 'Qwen 平台暂不支持 Agents' }
}

/** 删除 Qwen Agent（暂不支持） */
export const deleteQwenAgent = async (_nameOrRequest: string | Record<string, any>): Promise<any> => {
  return { success: false, message: 'Qwen 平台暂不支持 Agents' }
}

/** 切换 Qwen Agent（暂不支持） */
export const toggleQwenAgent = async (
  _nameOrRequest: string | Record<string, any>,
  _enabled?: boolean,
): Promise<any> => {
  return { success: false, message: 'Qwen 平台暂不支持 Agents' }
}

/** 列出 Qwen 插件（暂不支持） */
export const listQwenPlugins = async (): Promise<any> => {
  return { plugins: [] }
}

/** 添加 Qwen 插件（暂不支持） */
export const addQwenPlugin = async (
  _nameOrRequest: string | Record<string, any>,
  _config?: any,
): Promise<any> => {
  return { success: false, message: 'Qwen 平台暂不支持 Plugins' }
}

/** 更新 Qwen 插件（暂不支持） */
export const updateQwenPlugin = async (
  _nameOrRequest: string | Record<string, any>,
  _config?: any,
): Promise<any> => {
  return { success: false, message: 'Qwen 平台暂不支持 Plugins' }
}

/** 删除 Qwen 插件（暂不支持） */
export const deleteQwenPlugin = async (_nameOrRequest: string | Record<string, any>): Promise<any> => {
  return { success: false, message: 'Qwen 平台暂不支持 Plugins' }
}

/** 切换 Qwen 插件（暂不支持） */
export const toggleQwenPlugin = async (
  _nameOrRequest: string | Record<string, any>,
  _enabled?: boolean,
): Promise<any> => {
  return { success: false, message: 'Qwen 平台暂不支持 Plugins' }
}

// ════════════════════════════════════════════════════════════
// 8. iFlow 平台
// ════════════════════════════════════════════════════════════

/** 获取 iFlow 设置 */
export const getIflowConfig = async (): Promise<any> => {
  return invoke('iflow_get_settings')
}

/** 更新 iFlow 设置 */
export const updateIflowConfig = async (settings: any): Promise<any> => {
  return invoke('iflow_update_settings', { settings })
}

/** 列出 iFlow MCP 服务器 */
export const listIflowMcpServers = async (): Promise<any> => {
  return invoke('iflow_list_mcp_servers')
}

/** 添加 iFlow MCP 服务器 */
export const addIflowMcpServer = async (
  nameOrRequest: string | Record<string, any>,
  config?: any,
): Promise<any> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('iflow_add_mcp_server', { name, config: resolvedConfig })
}

/** 更新 iFlow MCP 服务器 */
export const updateIflowMcpServer = async (
  nameOrRequest: string | Record<string, any>,
  config?: any,
): Promise<any> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('iflow_update_mcp_server', { name, config: resolvedConfig })
}

/** 删除 iFlow MCP 服务器 */
export const deleteIflowMcpServer = async (nameOrRequest: string | Record<string, any>): Promise<any> => {
  const name = resolveName(nameOrRequest)
  return invoke('iflow_delete_mcp_server', { name })
}

/** 列出 iFlow 斜杠命令 */
export const listIflowSlashCommands = async (): Promise<any> => {
  return invoke('iflow_list_slash_commands')
}

// ── iFlow 平台限制 — 安全默认值 ──

/** 添加 iFlow 斜杠命令（暂不支持） */
export const addIflowSlashCommand = async (_name: string, _config: any): Promise<any> => {
  return { success: false, message: 'iFlow 平台暂不支持此操作' }
}

/** 更新 iFlow 斜杠命令（暂不支持） */
export const updateIflowSlashCommand = async (_name: string, _config: any): Promise<any> => {
  return { success: false, message: 'iFlow 平台暂不支持此操作' }
}

/** 删除 iFlow 斜杠命令（暂不支持） */
export const deleteIflowSlashCommand = async (_name: string): Promise<any> => {
  return { success: false, message: 'iFlow 平台暂不支持此操作' }
}

/** 切换 iFlow 斜杠命令（暂不支持） */
export const toggleIflowSlashCommand = async (
  _name: string,
  _enabled: boolean,
): Promise<any> => {
  return { success: false, message: 'iFlow 平台暂不支持此操作' }
}

/** 列出 iFlow Agents（暂不支持） */
export const listIflowAgents = async (): Promise<any> => {
  return { agents: [] }
}

/** 添加 iFlow Agent（暂不支持） */
export const addIflowAgent = async (
  _nameOrRequest: string | Record<string, any>,
  _config?: any,
): Promise<any> => {
  return { success: false, message: 'iFlow 平台暂不支持 Agents' }
}

/** 更新 iFlow Agent（暂不支持） */
export const updateIflowAgent = async (
  _nameOrRequest: string | Record<string, any>,
  _config?: any,
): Promise<any> => {
  return { success: false, message: 'iFlow 平台暂不支持 Agents' }
}

/** 删除 iFlow Agent（暂不支持） */
export const deleteIflowAgent = async (_nameOrRequest: string | Record<string, any>): Promise<any> => {
  return { success: false, message: 'iFlow 平台暂不支持 Agents' }
}

/** 切换 iFlow Agent（暂不支持） */
export const toggleIflowAgent = async (
  _nameOrRequest: string | Record<string, any>,
  _enabled?: boolean,
): Promise<any> => {
  return { success: false, message: 'iFlow 平台暂不支持 Agents' }
}

/** 列出 iFlow 插件（暂不支持） */
export const listIflowPlugins = async (): Promise<any> => {
  return { plugins: [] }
}

/** 添加 iFlow 插件（暂不支持） */
export const addIflowPlugin = async (
  _nameOrRequest: string | Record<string, any>,
  _config?: any,
): Promise<any> => {
  return { success: false, message: 'iFlow 平台暂不支持 Plugins' }
}

/** 更新 iFlow 插件（暂不支持） */
export const updateIflowPlugin = async (
  _nameOrRequest: string | Record<string, any>,
  _config?: any,
): Promise<any> => {
  return { success: false, message: 'iFlow 平台暂不支持 Plugins' }
}

/** 删除 iFlow 插件（暂不支持） */
export const deleteIflowPlugin = async (_nameOrRequest: string | Record<string, any>): Promise<any> => {
  return { success: false, message: 'iFlow 平台暂不支持 Plugins' }
}

/** 切换 iFlow 插件（暂不支持） */
export const toggleIflowPlugin = async (
  _nameOrRequest: string | Record<string, any>,
  _enabled?: boolean,
): Promise<any> => {
  return { success: false, message: 'iFlow 平台暂不支持 Plugins' }
}

// ════════════════════════════════════════════════════════════
// 9. Droid 平台
// ════════════════════════════════════════════════════════════

/** 获取 Droid 设置 */
export const getDroidSettings = async (): Promise<any> => {
  return invoke('droid_get_settings')
}

/** 更新 Droid 设置 */
export const updateDroidSettings = async (settings: any): Promise<any> => {
  return invoke('droid_update_settings', { settings })
}

/** 列出 Droid MCP 服务器 */
export const listDroidMcpServers = async (): Promise<any> => {
  return invoke('droid_list_mcp_servers')
}

/** 添加 Droid MCP 服务器 */
export const addDroidMcpServer = async (
  nameOrRequest: string | Record<string, any>,
  config?: any,
): Promise<any> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('droid_add_mcp_server', { name, config: resolvedConfig })
}

/** 更新 Droid MCP 服务器 */
export const updateDroidMcpServer = async (
  nameOrRequest: string | Record<string, any>,
  config?: any,
): Promise<any> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('droid_update_mcp_server', { name, config: resolvedConfig })
}

/** 删除 Droid MCP 服务器 */
export const deleteDroidMcpServer = async (nameOrRequest: string | Record<string, any>): Promise<any> => {
  const name = resolveName(nameOrRequest)
  return invoke('droid_delete_mcp_server', { name })
}

/** 列出 Droid Agents */
export const listDroidAgents = async (): Promise<any> => {
  return invoke('droid_list_agents')
}

/** 获取 Droid Agent 详情 */
export const getDroidAgent = async (name: string): Promise<any> => {
  const agents: any = await invoke('droid_list_agents')
  if (agents && typeof agents === 'object' && agents[name]) {
    return { name, ...agents[name] }
  }
  return null
}

/** 添加 Droid Agent */
export const addDroidAgent = async (
  nameOrRequest: string | Record<string, any>,
  config?: any,
): Promise<any> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('droid_add_agent', { name, config: resolvedConfig })
}

/** 更新 Droid Agent */
export const updateDroidAgent = async (
  nameOrRequest: string | Record<string, any>,
  config?: any,
): Promise<any> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('droid_update_agent', { name, config: resolvedConfig })
}

/** 删除 Droid Agent */
export const deleteDroidAgent = async (nameOrRequest: string | Record<string, any>): Promise<any> => {
  const name = resolveName(nameOrRequest)
  return invoke('droid_delete_agent', { name })
}

/** 列出 Droid 插件 */
export const listDroidPlugins = async (): Promise<DroidPlugin[]> => {
  const result = await invoke<any>('droid_list_plugins')
  if (Array.isArray(result)) {
    return result as DroidPlugin[]
  }
  if (result && typeof result === 'object' && Array.isArray(result.plugins)) {
    return result.plugins as DroidPlugin[]
  }
  return []
}

/** 添加 Droid 插件 */
export const addDroidPlugin = async (
  nameOrRequest: string | Record<string, any>,
  config?: any,
): Promise<any> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('droid_add_plugin', { name, config: resolvedConfig })
}

/** 更新 Droid 插件 */
export const updateDroidPlugin = async (
  nameOrRequest: string | Record<string, any>,
  config?: any,
): Promise<any> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('droid_update_plugin', { name, config: resolvedConfig })
}

/** 删除 Droid 插件 */
export const deleteDroidPlugin = async (nameOrRequest: string | Record<string, any>): Promise<any> => {
  const name = resolveName(nameOrRequest)
  return invoke('droid_delete_plugin', { name })
}

/** 列出 Droid 斜杠命令 */
export const listDroidSlashCommands = async (): Promise<any> => {
  return invoke('droid_list_slash_commands')
}

/** 添加 Droid 斜杠命令 */
export const addDroidSlashCommand = async (name: string, config: any): Promise<any> => {
  return invoke('droid_add_slash_command', { name, config })
}

/** 更新 Droid 斜杠命令 */
export const updateDroidSlashCommand = async (name: string, config: any): Promise<any> => {
  return invoke('droid_update_slash_command', { name, config })
}

/** 删除 Droid 斜杠命令 */
export const deleteDroidSlashCommand = async (name: string): Promise<any> => {
  return invoke('droid_delete_slash_command', { name })
}

/** 列出 Droid 模型 */
export const listDroidModels = async (): Promise<any> => {
  return invoke('droid_list_models')
}

/** 添加 Droid 模型 */
export const addDroidModel = async (model: Record<string, any>): Promise<any> => {
  const settings = await getDroidSettings()
  const models = Array.isArray(settings?.customModels) ? [...settings.customModels] : []

  if (models.some((item: any) => item?.model === model?.model)) {
    throw new Error(`模型 '${model?.model || ''}' 已存在`)
  }

  models.push(model)
  await updateDroidSettings({ customModels: models })
  return model
}

/** 更新 Droid 模型 */
export const updateDroidModel = async (modelId: string, model: Record<string, any>): Promise<any> => {
  const settings = await getDroidSettings()
  const models = Array.isArray(settings?.customModels) ? [...settings.customModels] : []
  const index = models.findIndex((item: any) => item?.model === modelId)

  if (index === -1) {
    throw new Error(`模型 '${modelId}' 不存在`)
  }

  models[index] = { ...models[index], ...model }
  await updateDroidSettings({ customModels: models })
  return models[index]
}

/** 删除 Droid 模型 */
export const deleteDroidModel = async (modelId: string): Promise<string> => {
  const settings = await getDroidSettings()
  const models = Array.isArray(settings?.customModels) ? [...settings.customModels] : []
  const nextModels = models.filter((item: any) => item?.model !== modelId)

  if (nextModels.length === models.length) {
    throw new Error(`模型 '${modelId}' 不存在`)
  }

  await updateDroidSettings({ customModels: nextModels })
  return modelId
}

const normalizeDroidProfiles = (profiles: any): Record<string, any> => {
  if (Array.isArray(profiles)) {
    return profiles.reduce((acc: Record<string, any>, profile: any) => {
      if (profile?.name) {
        acc[profile.name] = { ...profile }
        delete acc[profile.name].name
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
export const listDroidProfiles = async (): Promise<any[]> => {
  const settings = await getDroidSettings()
  const profiles = normalizeDroidProfiles(settings?.profiles)

  return Object.entries(profiles).map(([name, config]) => ({
    name,
    ...(config as Record<string, any>),
  }))
}

/** 添加 Droid Profile */
export const addDroidProfile = async (nameOrRequest: string | Record<string, any>, config?: any): Promise<any> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  const settings = await getDroidSettings()
  const profiles = normalizeDroidProfiles(settings?.profiles)

  if (profiles[name]) {
    throw new Error(`Profile '${name}' 已存在`)
  }

  profiles[name] = resolvedConfig
  await updateDroidSettings({ profiles })
  return { name, ...resolvedConfig }
}

/** 更新 Droid Profile */
export const updateDroidProfile = async (nameOrRequest: string | Record<string, any>, config?: any): Promise<any> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  const settings = await getDroidSettings()
  const profiles = normalizeDroidProfiles(settings?.profiles)

  if (!profiles[name]) {
    throw new Error(`Profile '${name}' 不存在`)
  }

  profiles[name] = { ...profiles[name], ...resolvedConfig }
  await updateDroidSettings({ profiles })
  return { name, ...profiles[name] }
}

/** 删除 Droid Profile */
export const deleteDroidProfile = async (nameOrRequest: string | Record<string, any>): Promise<string> => {
  const name = resolveName(nameOrRequest)
  const settings = await getDroidSettings()
  const profiles = normalizeDroidProfiles(settings?.profiles)

  if (!profiles[name]) {
    throw new Error(`Profile '${name}' 不存在`)
  }

  delete profiles[name]
  await updateDroidSettings({ profiles })
  return name
}

/** 切换 Droid Profile */
export const switchDroidProfile = async (name: string): Promise<any> => {
  const settings = await getDroidSettings()
  const profiles = normalizeDroidProfiles(settings?.profiles)

  if (!profiles[name]) {
    throw new Error(`Profile '${name}' 不存在`)
  }

  Object.keys(profiles).forEach((profileName) => {
    profiles[profileName] = {
      ...profiles[profileName],
      enabled: profileName === name,
    }
  })

  await updateDroidSettings({ profiles, currentProfile: name })
  return { name, ...profiles[name] }
}

// ════════════════════════════════════════════════════════════
// 10. OpenCode 平台
// ════════════════════════════════════════════════════════════

/** 获取 OpenCode 配置 */
export const getOpenCodeConfig = async (): Promise<any> => {
  return invoke('opencode_get_settings')
}

/** 更新 OpenCode 配置 */
export const updateOpenCodeConfig = async (settings: any): Promise<any> => {
  return invoke('opencode_update_settings', { settings })
}

/** 获取 OpenCode 快捷键 */
export const getOpenCodeKeybindings = async (): Promise<any> => {
  return invoke('opencode_get_keybindings')
}

/** 更新 OpenCode 快捷键 */
export const updateOpenCodeKeybindings = async (keybindings: any): Promise<any> => {
  return invoke('opencode_update_keybindings', { keybindings })
}

/** 列出 OpenCode 主题 */
export const listOpenCodeThemes = async (): Promise<any> => {
  return invoke('opencode_list_themes')
}

// ── OpenCode 组合实现（通过 get/update settings） ──

/** 列出 OpenCode Providers */
export const listOpenCodeProviders = async (): Promise<any> => {
  const settings = await getOpenCodeConfig()
  const providers = settings?.providers ?? {}
  return { providers: Object.entries(providers).map(([id, config]) => ({ id, ...(config as Record<string, any>) })) }
}

/** 添加 OpenCode Provider */
export const addOpenCodeProvider = async (
  providerOrName: string | Record<string, any>,
  config?: any,
): Promise<any> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(providerOrName, config)
  const settings = await getOpenCodeConfig()
  const providers = settings?.providers ?? {}
  providers[name] = resolvedConfig
  await updateOpenCodeConfig({ providers })
  return { id: name, ...resolvedConfig }
}

/** 更新 OpenCode Provider */
export const updateOpenCodeProvider = async (
  providerOrName: string | Record<string, any>,
  config?: any,
): Promise<any> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(providerOrName, config)
  const settings = await getOpenCodeConfig()
  const providers = settings?.providers ?? {}
  providers[name] = { ...providers[name], ...resolvedConfig }
  await updateOpenCodeConfig({ providers })
  return { id: name, ...providers[name] }
}

/** 删除 OpenCode Provider */
export const deleteOpenCodeProvider = async (name: string): Promise<any> => {
  const settings = await getOpenCodeConfig()
  const providers = settings?.providers ?? {}
  delete providers[name]
  await updateOpenCodeConfig({ providers })
  return name
}

/** 列出 OpenCode MCP 服务器 */
export const listOpenCodeMcpServers = async (): Promise<any> => {
  const settings = await getOpenCodeConfig()
  const servers = settings?.mcpServers ?? settings?.mcp_servers ?? {}
  return { servers: Object.entries(servers).map(([name, config]) => ({ name, ...(config as Record<string, any>) })) }
}

/** 添加 OpenCode MCP 服务器 */
export const addOpenCodeMcpServer = async (
  serverOrName: string | Record<string, any>,
  config?: any,
): Promise<any> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(serverOrName, config)
  const settings = await getOpenCodeConfig()
  const servers = settings?.mcpServers ?? settings?.mcp_servers ?? {}
  servers[name] = resolvedConfig
  await updateOpenCodeConfig({ mcpServers: servers })
  return { name, ...resolvedConfig }
}

/** 更新 OpenCode MCP 服务器 */
export const updateOpenCodeMcpServer = async (
  serverOrName: string | Record<string, any>,
  config?: any,
): Promise<any> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(serverOrName, config)
  const settings = await getOpenCodeConfig()
  const servers = settings?.mcpServers ?? settings?.mcp_servers ?? {}
  servers[name] = { ...servers[name], ...resolvedConfig }
  await updateOpenCodeConfig({ mcpServers: servers })
  return { name, ...servers[name] }
}

/** 删除 OpenCode MCP 服务器 */
export const deleteOpenCodeMcpServer = async (name: string): Promise<any> => {
  const settings = await getOpenCodeConfig()
  const servers = settings?.mcpServers ?? settings?.mcp_servers ?? {}
  delete servers[name]
  await updateOpenCodeConfig({ mcpServers: servers })
  return name
}

/** 列出 OpenCode 插件 */
export const listOpenCodePlugins = async (): Promise<any> => {
  const settings = await getOpenCodeConfig()
  const plugins = settings?.plugins ?? {}
  return { plugins: Object.entries(plugins).map(([name, config]) => ({ name, ...(config as Record<string, any>) })) }
}

/** 添加 OpenCode 插件 */
export const addOpenCodePlugin = async (
  pluginOrName: string | Record<string, any>,
  config?: any,
): Promise<any> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(pluginOrName, config)
  const settings = await getOpenCodeConfig()
  const plugins = settings?.plugins ?? {}
  plugins[name] = resolvedConfig
  await updateOpenCodeConfig({ plugins })
  return { name, ...resolvedConfig }
}

/** 删除 OpenCode 插件 */
export const deleteOpenCodePlugin = async (name: string): Promise<any> => {
  const settings = await getOpenCodeConfig()
  const plugins = settings?.plugins ?? {}
  delete plugins[name]
  await updateOpenCodeConfig({ plugins })
  return name
}

// ════════════════════════════════════════════════════════════
// 11. 签到 (CheckIn)
// ════════════════════════════════════════════════════════════

/** 列出签到 Provider */
export const listCheckinProviders = async (): Promise<any> => {
  return invoke('list_providers')
}

/** 获取签到 Provider 详情 */
export const getCheckinProvider = async (id: string): Promise<any> => {
  const result: any = await invoke('list_providers')
  const providers = result?.providers || result || []
  return providers.find((p: any) => p.id === id || String(p.id) === id) ?? null
}

/** 创建签到 Provider */
export const createCheckinProvider = async (data: any): Promise<any> => {
  return invoke('add_provider', { data })
}

/** 更新签到 Provider */
export const updateCheckinProvider = async (id: string, data: any): Promise<any> => {
  return invoke('update_provider', { id, data })
}

/** 删除签到 Provider */
export const deleteCheckinProvider = async (id: string): Promise<any> => {
  return invoke('delete_provider', { id })
}

/** 测试签到连接 */
export const testCheckinConnection = async (id: string): Promise<any> => {
  return invoke('test_provider_connection', { id })
}

/** 列出签到账号 */
export const listCheckinAccounts = async (providerId?: string): Promise<any> => {
  return invoke('list_accounts', { providerId })
}

/** 获取签到账号详情 */
export const getCheckinAccount = async (id: string): Promise<any> => {
  const result: any = await invoke('list_accounts', { providerId: null })
  const accounts = result?.accounts || result || []
  return accounts.find((a: any) => a.id === id || String(a.id) === id) ?? null
}

/** 获取签到账号仪表盘（完整 dashboard 数据：account + streak + calendar + trend） */
export const getCheckinAccountDashboard = async (
  id: string,
  query?: { year?: number; month?: number; days?: number },
): Promise<any> => {
  return invoke('get_account_dashboard', {
    accountId: id,
    year: query?.year ?? null,
    month: query?.month ?? null,
    days: query?.days ?? null,
  })
}

/** 创建签到账号 */
export const createCheckinAccount = async (data: any): Promise<any> => {
  return invoke('add_account', { data })
}

/** 更新签到账号 */
export const updateCheckinAccount = async (id: string, data: any): Promise<any> => {
  return invoke('update_account', { id, data })
}

/** 删除签到账号 */
export const deleteCheckinAccount = async (id: string): Promise<any> => {
  return invoke('delete_account', { id })
}

/** 批量删除签到账号 */
export const batchDeleteAccounts = async (ids: string[]): Promise<any> => {
  return invoke('batch_delete_accounts', { ids })
}

/** 执行签到 */
export const executeCheckin = async (accountId: string): Promise<any> => {
  return invoke('execute_checkin', { accountId })
}

/** 签到（executeCheckin 的别名） */
export const checkinAccount = executeCheckin

/** 批量签到 */
export const batchCheckin = async (accountIds: string[]): Promise<any> => {
  return invoke('batch_checkin', { accountIds })
}

/** 查询签到余额 */
export const queryCheckinBalance = async (accountId: string): Promise<any> => {
  return invoke('get_balance', { accountId })
}

/** 获取余额历史 */
export const getCheckinBalanceHistory = async (
  accountId: string,
  days?: number,
): Promise<any> => {
  return invoke('get_balance_history', { accountId, days })
}

/** 获取余额统计 */
export const getBalanceStats = async (): Promise<any> => {
  return invoke('get_balance_stats')
}

/** 列出签到记录 */
export const listCheckinRecords = async (
  params?: number | { page?: number; page_size?: number; account_id?: string },
): Promise<any> => {
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
export const getAccountCheckinRecords = async (
  accountId: string,
  limit?: number,
): Promise<any> => {
  return invoke('get_checkin_records', { accountId, limit })
}

/** 导出签到记录 */
export const exportCheckinRecords = async (options: any): Promise<any> => {
  return invoke('export_checkin_data', { options })
}

/** 获取今日签到统计 */
export const getTodayCheckinStats = async (): Promise<any> => {
  return invoke('export_checkin_stats')
}

/** 执行 CDK 充值 */
export const executeCdkRecharge = async (
  accountId: string,
  cdkCode: string,
): Promise<any> => {
  return invoke('execute_cdk_recharge', { accountId, cdkCode })
}

/** 获取 CDK 历史 */
export const getCdkHistory = async (accountId?: string): Promise<any> => {
  return invoke('get_cdk_history', { accountId })
}

/** 列出 WAF Cookies */
export const listWafCookies = async (): Promise<any> => {
  return invoke('list_waf_cookies')
}

/** 添加 WAF Cookie */
export const addWafCookie = async (providerId: string, cookie: string): Promise<any> => {
  return invoke('add_waf_cookie', { providerId, cookie })
}

/** 删除 WAF Cookie */
export const deleteWafCookie = async (id: string): Promise<any> => {
  return invoke('delete_waf_cookie', { id })
}

// ── CheckIn 扩展 ──

/** 获取签到账号 Cookies */
export const getCheckinAccountCookies = async (accountId: string): Promise<any> => {
  return invoke('get_checkin_account_cookies', { accountId })
}

/** 导出签到配置 */
export const exportCheckinConfig = async (options?: Record<string, any>): Promise<any> => {
  return invoke('export_checkin_config', { options: options ?? null })
}

/** 预览签到导入 */
export const previewCheckinImport = async (data: any): Promise<any> => {
  return invoke('preview_checkin_import', { data })
}

/** 导入签到配置 */
export const importCheckinConfig = async (data: any, options?: any): Promise<any> => {
  return invoke('import_checkin_config', { data, options: options ?? null })
}

/** 列出内置 Provider */
export const listBuiltinProviders = async (): Promise<any> => {
  return invoke('list_builtin_providers')
}

/** 添加内置 Provider */
export const addBuiltinProvider = async (providerId: string): Promise<any> => {
  return invoke('add_builtin_provider', { providerId })
}

/** 获取 OAuth 授权链接（仅 HTTP 后端支持） */
export const getOAuthAuthorizeUrl = async (
  _request: OAuthAuthorizeUrlRequest,
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
export const getCostOverview = async (period?: string): Promise<any> => {
  return invoke('get_cost_overview', { period })
}

/** 获取热力图数据 */
export const getHeatmapData = async (
  platform?: string,
  days?: number,
): Promise<any> => {
  return invoke('get_heatmap_data', { platform, days })
}

/** V2: 获取使用量汇总 */
export const getUsageSummaryV2 = async (
  platform?: string,
  startDate?: string,
  endDate?: string,
): Promise<any> => {
  return invoke('get_usage_summary_v2', { platform, startDate, endDate })
}

/** V2: 获取每日趋势 */
export const getUsageTrendsV2 = async (
  platform?: string,
  startDate?: string,
  endDate?: string,
): Promise<any> => {
  return invoke('get_usage_trends_v2', { platform, startDate, endDate })
}

/** V2: 获取模型统计 */
export const getUsageByModelV2 = async (
  platform?: string,
  startDate?: string,
  endDate?: string,
): Promise<any> => {
  return invoke('get_usage_by_model_v2', { platform, startDate, endDate })
}

/** V2: 获取项目统计 */
export const getUsageByProjectV2 = async (
  platform?: string,
  startDate?: string,
  endDate?: string,
): Promise<any> => {
  return invoke('get_usage_by_project_v2', { platform, startDate, endDate })
}

/** V2: 获取热力图（兼容映射到现有命令） */
export const getUsageHeatmapV2 = async (
  platform?: string,
  days?: number,
): Promise<any> => {
  return getHeatmapData(platform, days)
}

/** V2: 获取日志 */
export const getUsageLogsV2 = async (
  platform?: string,
  page?: number,
  pageSize?: number,
  _model?: string,
  _cursor?: string,
  _includeTotal?: boolean,
): Promise<any> => {
  return invoke('get_usage_logs_v2', { platform, page, pageSize })
}

/** V2: 获取仪表盘聚合 */
export const getUsageDashboardV2 = async (
  platform?: string,
  startDate?: string,
  endDate?: string,
  _heatmapDays?: number,
  _includeHeatmap?: boolean,
): Promise<any> => {
  return invoke('get_usage_dashboard_v2', { platform, startDate, endDate })
}

/** V2: 导入单平台 usage */
export const importUsageV2 = async (platform: string): Promise<any> => {
  return invoke('import_usage_v2', { platform })
}

/** V2: 导入全部 usage */
export const importAllUsageV2 = async (): Promise<any> => {
  return invoke('import_all_usage_v2')
}

/** 获取会话统计 */
export const getSessionStats = async (platform?: string): Promise<any> => {
  return invoke('get_session_stats', { platform })
}

// ── Stats 扩展 ──

/** 获取费用趋势 */
export const getCostTrend = async (period?: string): Promise<any> => {
  return invoke('get_cost_trend', { period })
}

/** 按模型统计费用 */
export const getCostByModel = async (_period?: string): Promise<any> => {
  return invoke('get_cost_by_model')
}

/** 按项目统计费用 */
export const getCostByProject = async (_period?: string): Promise<any> => {
  return invoke('get_cost_by_project')
}

/** 获取提供商使用量 */
export const getProviderUsage = async (): Promise<any> => {
  return invoke('get_provider_usage')
}

/** 获取 Top Sessions */
export const getTopSessions = async (limit?: number): Promise<any> => {
  return invoke('get_top_sessions', { limit })
}

/** 获取统计摘要 */
export const getStatsSummary = async (): Promise<any> => {
  return invoke('get_stats_summary')
}

/** 设置定价 */
export const setPricing = async (data: any): Promise<any> => {
  const model = data?.model ?? data?.name ?? ''
  return invoke('set_pricing', { model, pricing: data })
}

/** 获取定价列表 */
export const getPricingList = async (): Promise<any> => {
  return invoke('get_pricing_list')
}

/** 移除定价 */
export const removePricing = async (model: string): Promise<any> => {
  return invoke('remove_pricing', { model })
}

/** 重置定价 */
export const resetPricing = async (): Promise<any> => {
  return invoke('reset_pricing')
}

/** 获取每日统计 */
export const getDailyStats = async (days?: number): Promise<any> => {
  return invoke('get_daily_stats', { days })
}

// ════════════════════════════════════════════════════════════
// 13. 系统 (System)
// ════════════════════════════════════════════════════════════

/** 获取系统信息 */
export const getSystemInfo = async (): Promise<any> => {
  return invoke('get_system_info')
}

/** 检查版本更新 */
export const checkVersion = async (): Promise<any> => {
  return invoke('check_version')
}

/** 健康检查 */
export const healthCheck = async (): Promise<any> => {
  return invoke('health_check')
}

/** 获取版本号（通过 check_version 实现） */
export const getVersion = async (): Promise<any> => {
  return invoke('check_version')
}

/** 检查更新（别名） */
export const checkUpdate = checkVersion

/** 执行 CCR 更新 */
export const updateCCR = async (_branch?: string): Promise<any> => {
  return invoke('update_ccr')
}

/** 获取 CLI 版本 */
export const getCliVersions = async (_options?: Record<string, any>): Promise<any> => {
  return invoke('get_cli_versions')
}

// ════════════════════════════════════════════════════════════
// 14. 转换器 (Converter)
// ════════════════════════════════════════════════════════════

/** 转换配置格式 */
export const convertConfig = async (request: any): Promise<any> => {
  return invoke('convert_config', { request })
}

// ════════════════════════════════════════════════════════════
// 15. UI 状态 (Favorites / Recent Items)
// ════════════════════════════════════════════════════════════

/** 获取收藏列表 */
export const getFavorites = async (): Promise<any> => {
  return invoke('get_favorites')
}

/** 添加收藏 */
export const addFavorite = async (
  command: string,
  args: string[],
  displayName: string | undefined,
  module: string,
): Promise<any> => {
  return invoke('add_favorite', { command, args, displayName, module })
}

/** 移除收藏 */
export const removeFavorite = async (id: string): Promise<any> => {
  return invoke('remove_favorite', { id })
}

/** 获取最近项目 */
export const getRecentItems = async (limit?: number): Promise<any> => {
  return invoke('get_recent_items', { limit })
}

/** 添加最近项目 */
export const addRecentItem = async (
  command: string,
  args: string[],
  success: boolean,
  durationMs: number,
): Promise<any> => {
  return invoke('add_recent_item', { command, args, success, durationMs })
}

/** 清空最近项目 */
export const clearRecentItems = async (): Promise<any> => {
  return invoke('clear_recent_items')
}

// ════════════════════════════════════════════════════════════
// 16. WAF
// ════════════════════════════════════════════════════════════

/** 打开 WAF 登录窗口 */
export const openWafLogin = async (
  loginUrl: string,
  providerId: number,
): Promise<any> => {
  return invoke('open_waf_login', { loginUrl, providerId })
}

/** 获取 WAF Cookie 状态 */
export const getWafCookieStatus = async (providerId: number): Promise<any> => {
  return invoke('get_waf_cookie_status', { providerId })
}

// ════════════════════════════════════════════════════════════
// 17. 统一 MCP (Unified MCP)
// ════════════════════════════════════════════════════════════

/** 列出所有平台的 MCP 服务器（统一视图） */
export const listUnifiedMcp = async (platforms?: string[] | string): Promise<any> => {
  const normalized = typeof platforms === 'string' ? [platforms] : platforms
  return invoke('unified_list_mcp_servers', { platforms: normalized })
}

/** 添加统一 MCP 服务器 */
export const addUnifiedMcp = async (request: any): Promise<any> => {
  return invoke('unified_add_mcp_server', { request })
}

/** 更新统一 MCP 服务器（通过删除+添加实现） */
export const updateUnifiedMcp = async (
  platformOrRequest: string | Record<string, any>,
  name?: string,
  request?: any,
): Promise<any> => {
  const mergedRequest =
    typeof platformOrRequest === 'string'
      ? { ...(request ?? {}), platform: platformOrRequest, name }
      : platformOrRequest

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
export const deleteUnifiedMcp = async (platform: string, name: string): Promise<any> => {
  return invoke('unified_delete_mcp_server', { platform, name })
}

/** 切换统一 MCP 服务器启用/禁用 */
export const toggleUnifiedMcp = async (
  platform: string,
  name: string,
  disabled?: boolean,
): Promise<any> => {
  if (platform === 'claude') {
    return invoke('claude_update_mcp_server', { name, config: { disabled: disabled ?? true } })
  }
  throw new Error(`[Tauri] toggleUnifiedMcp: 平台 ${platform} 不支持启用/禁用切换`)
}

// ════════════════════════════════════════════════════════════
// 18. 事件 (Events)
// ════════════════════════════════════════════════════════════

/** 获取最近事件 */
export const getRecentEvents = async (count?: number): Promise<any> => {
  return invoke('get_recent_events', { count })
}

// ════════════════════════════════════════════════════════════
// 19. 环境管理 (Environment)
// ════════════════════════════════════════════════════════════

/** 列出所有执行环境 */
export const listEnvironments = async (): Promise<any> => {
  return invoke('list_environments')
}

/** 获取当前活跃环境 */
export const getCurrentEnvironment = async (): Promise<any> => {
  return invoke('get_current_environment')
}

/** 切换活跃环境 */
export const switchEnvironment = async (envId: string): Promise<any> => {
  return invoke('switch_environment', { envId })
}

/** 刷新环境列表 */
export const refreshEnvironments = async (): Promise<any> => {
  return invoke('refresh_environments')
}

/** 通过环境列出平台 */
export const envListPlatforms = async (): Promise<any> => {
  return invoke('env_list_platforms')
}

/** 通过环境检测 CLI */
export const envDetectCli = async (): Promise<any> => {
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

export const sshReconnect = async (envId: string, password?: string): Promise<SshConnectionState> => {
  return invoke('ssh_reconnect', { envId, password })
}

export const sshDisconnect = async (): Promise<SshConnectionState> => {
  return invoke('ssh_disconnect')
}

export const sshGetConnectionState = async (
  envId?: string,
): Promise<SshConnectionState | SshConnectionState[]> => {
  return invoke('ssh_get_connection_state', { envId })
}

export const sshProbeHostFingerprint = async (
  envId?: string,
  host?: string,
  port?: number,
): Promise<SshFingerprintProbeResult> => {
  return invoke('ssh_probe_host_fingerprint', { request: { env_id: envId, host, port } })
}

export const sshConfirmHostFingerprint = async (
  host: string,
  keyType: string,
  fingerprint: string,
  port?: number,
): Promise<void> => {
  return invoke('ssh_confirm_host_fingerprint', {
    request: { host, key_type: keyType, fingerprint, port },
  })
}

export const sshReadConfig = async (
  envId: string,
  platform: string,
  path: string,
): Promise<string> => {
  return invoke('ssh_read_config', { envId, platform, path })
}

export const sshWriteConfig = async (
  envId: string,
  platform: string,
  path: string,
  content: string,
  enableBackup = true,
): Promise<void> => {
  return invoke('ssh_write_config', { envId, platform, path, content, enableBackup })
}

export const sshDetectCli = async (envId: string): Promise<any> => {
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
  args?: string[],
): Promise<any> => {
  const command = typeof commandOrPayload === 'string' ? commandOrPayload : commandOrPayload.command
  const resolvedArgs = typeof commandOrPayload === 'string' ? args : commandOrPayload.args
  return invoke('execute_ccr_command', { command, args: resolvedArgs })
}

/** 列出可用命令 */
export const listCommands = async (_client?: string): Promise<any> => {
  return invoke('list_ccr_commands')
}

/** 获取命令帮助 */
export const getCommandHelp = async (command: string): Promise<any> => {
  return invoke('get_ccr_command_help', { command })
}

/** 启用配置（通过 switchConfig 实现） */
export const enableConfig = async (name: string): Promise<any> => {
  return switchConfig(name)
}

/** 禁用配置（通过 update_config 设置 enabled=false） */
export const disableConfig = async (name: string): Promise<any> => {
  return invoke('update_config', { name, data: { enabled: false } })
}

/** 获取单个配置详情（通过列表后过滤实现） */
export const getConfig = async (name: string): Promise<any> => {
  const result: any = await invoke('list_configs')
  const configs = result?.configs || result || []
  return configs.find((c: any) => c.name === name) ?? null
}

/** 更新配置 */
export const updateConfig = async (name: string, config: any): Promise<any> => {
  return invoke('update_config', { name, data: config })
}

/** 清理备份 */
export const cleanBackups = async (_days?: number): Promise<any> => {
  return invoke('clean_backups')
}

// ── MCP 预设 / 同步 / 内置提示词 ──

/** 列出 MCP 预设 */
export const listMcpPresets = async (): Promise<any> => {
  return invoke('list_mcp_presets')
}

/** 获取 MCP 预设详情 */
export const getMcpPreset = async (id: string): Promise<any> => {
  return invoke('get_mcp_preset', { id })
}

/** 安装 MCP 预设 */
export const installMcpPreset = async (
  presetIdOrRequest: string | Record<string, any>,
  platforms?: string[],
  env?: Record<string, string>,
): Promise<any> => {
  const presetId = typeof presetIdOrRequest === 'string' ? presetIdOrRequest : presetIdOrRequest.preset_id ?? presetIdOrRequest.id ?? ''
  const envVars = env ?? (typeof presetIdOrRequest === 'object' ? presetIdOrRequest.env : undefined)
  return invoke('install_mcp_preset', { presetId, platforms, envVars })
}

/** 安装单个 MCP 预设 */
export const installMcpPresetSingle = async (
  presetIdOrRequest: string | Record<string, any>,
  platform?: string,
  env?: Record<string, string>,
): Promise<any> => {
  const presetId = typeof presetIdOrRequest === 'string' ? presetIdOrRequest : presetIdOrRequest.preset_id ?? presetIdOrRequest.id ?? ''
  const resolvedPlatform = platform ?? (typeof presetIdOrRequest === 'object' ? presetIdOrRequest.platform : undefined)
  const envVars = env ?? (typeof presetIdOrRequest === 'object' ? presetIdOrRequest.env : undefined)
  return invoke('install_mcp_preset_single', { platform: resolvedPlatform, presetId, envVars })
}

/** 列出来源 MCP 服务器 */
export const listSourceMcpServers = async (): Promise<any> => {
  return invoke('list_source_mcp_servers')
}

/** 同步 MCP 服务器 */
export const syncMcpServer = async (name: string, platforms?: string[]): Promise<any> => {
  return invoke('sync_mcp_server', { name, platforms })
}

/** 同步所有 MCP 服务器 */
export const syncAllMcpServers = async (platforms?: string[]): Promise<any> => {
  return invoke('sync_all_mcp_servers', { platforms })
}

/** 列出内置提示词 */
export const listBuiltinPrompts = async (): Promise<any> => {
  return invoke('list_builtin_prompts')
}

/** 获取内置提示词 */
export const getBuiltinPrompt = async (id: string): Promise<any> => {
  return invoke('get_builtin_prompt', { id })
}

/** 按分类获取内置提示词 */
export const getBuiltinPromptsByCategory = async (category: string): Promise<any> => {
  return invoke('get_builtin_prompts_by_category', { category })
}

// ── Skills ──

/** 列出技能 */
export const listSkills = async (): Promise<any> => {
  return invoke('list_skills')
}

/** 添加技能 */
export const addSkill = async (data: any): Promise<any> => {
  const name = data?.name ?? ''
  const instruction = data?.instruction ?? data?.content ?? ''
  return invoke('add_skill', { name, instruction })
}

/** 删除技能 */
export const deleteSkill = async (name: string): Promise<any> => {
  return invoke('delete_skill', { name })
}

/** 列出技能仓库 */
export const listSkillRepositories = async (): Promise<any> => {
  return invoke('list_skill_repositories')
}

/** 添加技能仓库 */
export const addSkillRepository = async (data: any): Promise<any> => {
  return invoke('add_skill_repository', { repo: data })
}

/** 移除技能仓库 */
export const removeSkillRepository = async (name: string): Promise<any> => {
  return invoke('remove_skill_repository', { name })
}

/** 扫描技能仓库 */
export const scanSkillRepository = async (urlOrName: string): Promise<any> => {
  return invoke('scan_skill_repository', { url: urlOrName })
}

// ── Skill Hub ──

/** 获取 SkillHub 趋势 */
export const getSkillHubTrending = async (): Promise<any> => {
  return invoke('skill_hub_trending')
}

/** 搜索 SkillHub 市场 */
export const searchSkillHubMarketplace = async (query: string, category?: string): Promise<any> => {
  return invoke('skill_hub_search', { query, category })
}

/** 获取 SkillHub Agents */
export const getSkillHubAgents = async (): Promise<any> => {
  return invoke('skill_hub_agents')
}

/** 获取 SkillHub Agent 技能 */
export const getSkillHubAgentSkills = async (agentName: string): Promise<any> => {
  return invoke('skill_hub_agent_skills', { agentName })
}

/** 安装 SkillHub 技能 */
export const installSkillHubSkill = async (data: any): Promise<any> => {
  const skillUrl = data?.url ?? data?.skill_url ?? data?.package ?? ''
  const targetDir = data?.target_dir ?? undefined
  return invoke('skill_hub_install', { skillUrl, targetDir })
}

/** 移除 SkillHub 技能 */
export const removeSkillHubSkill = async (skillPath: string): Promise<any> => {
  return invoke('skill_hub_remove', { skillPath })
}

/** 获取所有平台的 skills（统一查询） */
export const getSkillHubUnified = async (platform?: string): Promise<any> => {
  return invoke('skill_hub_unified', { platform: platform ?? null })
}

/** 读取 Skill 内容 */
export const getSkillHubSkillContent = async (skillDir: string): Promise<any> => {
  return invoke('skill_hub_skill_content', { skillDir })
}

/** 保存 Skill 内容 */
export const saveSkillHubSkillContent = async (skillDir: string, content: string): Promise<any> => {
  return invoke('skill_hub_save_skill_content', { skillDir, content })
}

/** 从 GitHub 导入技能 */
export const importSkillFromGithub = async (url: string, agents: string[], force?: boolean): Promise<any> => {
  return invoke('skill_hub_import_github', { url, agents, force: force ?? false })
}

/** 从本地目录导入技能 */
export const importSkillFromLocal = async (sourcePath: string, agents: string[], skillName?: string): Promise<any> => {
  return invoke('skill_hub_import_local', { sourcePath, agents, skillName: skillName ?? null })
}

/** 通过 npx 安装技能 */
export const importSkillViaNpx = async (packageName: string, agents: string[], global?: boolean): Promise<any> => {
  return invoke('skill_hub_import_npx', { packageName, agents, global: global ?? false })
}

/** 批量安装技能 */
export const batchInstallSkills = async (packages: string[], agents: string[], force?: boolean): Promise<any> => {
  return invoke('skill_hub_batch_install', { packages, agents, force: force ?? false })
}

/** 检查 npx 可用性 */
export const checkNpxAvailability = async (): Promise<any> => {
  return invoke('skill_hub_check_npx')
}

/** 打开文件夹选择对话框 */
export const browseForFolder = async (): Promise<any> => {
  return invoke('skill_hub_browse_folder')
}

/** 获取单个技能详情 */
export const getSkillDetail = async (name: string): Promise<any> => {
  return invoke('get_skill', { name })
}

/** 更新技能内容 */
export const updateSkillContent = async (name: string, instruction: string): Promise<any> => {
  return invoke('update_skill', { name, instruction })
}

// ── Axios / HTTP 核心（不再需要） ──

/** HTTP API 基础 URL（Tauri 模式返回空字符串） */
export const resolveApiBaseUrl = (): string => {
  return ''
}

/** 后端健康检查（使用 Tauri invoke 实现） */
export const getBackendHealth = async (): Promise<any> => {
  return invoke('health_check')
}
