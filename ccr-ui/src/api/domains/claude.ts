/**
 * Claude Domain —— Claude Code 平台 Settings / MCP / Agents / Slash / Plugins / Output Styles /
 * Statusline / Hooks / Budgets / Prompts / Profiles / Auth 全量 API
 *
 * 真迁移自 tauri.ts 第 4 分组。对应后端 commands::claude::* 命令。
 */

import { invoke } from '@tauri-apps/api/core'
import {
  asRecord,
  isRecord,
  pickArray,
  resolveName,
  resolveNameAndConfig,
  type UnknownRecord,
} from '../_shared'
import type { ClaudeSettingsData } from '../tauri'
import type {
  ConfigLayersResult,
  RawFileGetResult,
  RawFileSaveResult,
  RawProfilesSaveResult,
} from './configRawTypes'

// ── Claude Settings ──

/** 获取 Claude Code 全局设置 */
export const getClaudeSettings = async <T = ClaudeSettingsData>(): Promise<T> => {
  return invoke('claude_get_settings')
}

/** 更新 Claude Code 全局设置 */
export const updateClaudeSettings = async <T = UnknownRecord>(settings: unknown): Promise<T> => {
  return invoke('claude_update_settings', { settings })
}

export const getClaudeSettingsRaw = async (): Promise<RawFileGetResult> => {
  return invoke('claude_get_settings_raw_text')
}

export const saveClaudeSettingsRaw = async (
  content: string,
  token: string,
): Promise<RawFileSaveResult> => {
  return invoke('claude_save_settings_raw_text', { content, token })
}

export const listClaudeSettingsLayers = async (): Promise<ConfigLayersResult> => {
  return invoke('claude_list_settings_layers')
}

export type { ConfigLayer, ConfigLayersResult, RawFileGetResult, RawFileSaveResult } from './configRawTypes'

// ── Claude MCP Servers ──

/** 列出 Claude Code MCP 服务器 */
export const listMcpServers = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('claude_list_mcp_servers')
}

/** 添加 Claude Code MCP 服务器 */
export const addMcpServer = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown,
  scope?: string,
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  delete resolvedConfig.scope
  const requestScope = scope ?? (isRecord(nameOrRequest) && typeof nameOrRequest.scope === 'string'
    ? nameOrRequest.scope
    : undefined)
  return invoke('claude_add_mcp_server', { name, config: resolvedConfig, scope: requestScope })
}

/** 更新 Claude Code MCP 服务器 */
export const updateMcpServer = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown,
  scope?: string,
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  delete resolvedConfig.scope
  const requestScope = scope ?? (isRecord(nameOrRequest) && typeof nameOrRequest.scope === 'string'
    ? nameOrRequest.scope
    : isRecord(config) && typeof config.scope === 'string'
      ? config.scope
      : undefined)
  return invoke('claude_update_mcp_server', { name, config: resolvedConfig, scope: requestScope })
}

/** 删除 Claude Code MCP 服务器 */
export const deleteMcpServer = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  scope?: string,
): Promise<T> => {
  const name = resolveName(nameOrRequest)
  const requestScope = scope ?? (isRecord(nameOrRequest) && typeof nameOrRequest.scope === 'string'
    ? nameOrRequest.scope
    : undefined)
  return invoke('claude_delete_mcp_server', { name, scope: requestScope })
}

/** 切换 MCP 服务器启用/禁用（通过 disabled 字段实现） */
export const toggleMcpServer = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  disabled?: boolean,
): Promise<T> => {
  if (typeof nameOrRequest === 'string') {
    return invoke('claude_update_mcp_server', {
      name: nameOrRequest,
      config: { disabled: !!disabled },
      scope: undefined,
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
  const scope = typeof request.scope === 'string' ? request.scope : undefined
  return invoke('claude_update_mcp_server', { name, config: { disabled: resolvedDisabled }, scope })
}

// ── Claude Agents ──

/** 列出 Claude Code Agents */
export const listAgents = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('claude_list_agents')
}

/** 获取单个 Agent 详情（从列表过滤） */
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
  config?: unknown,
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('claude_add_agent', { name, config: resolvedConfig })
}

/** 更新 Claude Code Agent */
export const updateAgent = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown,
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('claude_update_agent', { name, config: resolvedConfig })
}

/** 删除 Claude Code Agent */
export const deleteAgent = async <T = UnknownRecord>(
  nameOrRequest: string | object,
): Promise<T> => {
  const name = resolveName(nameOrRequest)
  return invoke('claude_delete_agent', { name })
}

/** 切换 Agent 启用/禁用 */
export const toggleAgent = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  enabled?: boolean,
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

/** 列出斜杠命令 */
export const listSlashCommands = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('claude_list_slash_commands')
}

/** 添加斜杠命令 */
export const addSlashCommand = async <T = UnknownRecord>(
  name: string,
  config: unknown,
): Promise<T> => {
  return invoke('claude_add_slash_command', { name, config })
}

/** 更新斜杠命令 */
export const updateSlashCommand = async <T = UnknownRecord>(
  name: string,
  config: unknown,
): Promise<T> => {
  return invoke('claude_update_slash_command', { name, config })
}

/** 删除斜杠命令 */
export const deleteSlashCommand = async <T = UnknownRecord>(name: string): Promise<T> => {
  return invoke('claude_delete_slash_command', { name })
}

/** 切换斜杠命令启用/禁用 */
export const toggleSlashCommand = async <T = UnknownRecord>(
  name: string,
  enabled: boolean,
): Promise<T> => {
  return invoke('claude_update_slash_command', { name, config: { enabled } })
}

// ── Claude Plugins ──

/** 列出插件 */
export const listPlugins = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('claude_list_plugins')
}

/** 添加插件 */
export const addPlugin = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown,
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('claude_add_plugin', { name, config: resolvedConfig })
}

/** 更新插件 */
export const updatePlugin = async <T = UnknownRecord>(
  name: string,
  config: unknown,
): Promise<T> => {
  return invoke('claude_update_plugin', { name, config })
}

/** 删除插件 */
export const deletePlugin = async <T = UnknownRecord>(name: string): Promise<T> => {
  return invoke('claude_delete_plugin', { name })
}

/** 切换插件启用/禁用 */
export const togglePlugin = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  enabled?: boolean,
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

/** 获取单个输出样式（别名，后端同一 endpoint 返回整体） */
export const getOutputStyle = listOutputStyles

/** 创建输出样式（通过 update 实现） */
export const createOutputStyle = async <T = UnknownRecord>(styles: unknown): Promise<T> => {
  return invoke('claude_update_output_styles', { styles })
}

/** 更新输出样式 */
export const updateOutputStyle = async <T = UnknownRecord>(
  nameOrStyles: string | object,
  patch?: Record<string, unknown>,
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

/** 列出 Hooks —— 后端返回 `{ hooks: {...} }`，前端拆包到具体 hooks 对象 */
export const listHooks = async <T = UnknownRecord>(): Promise<T> => {
  const response = asRecord(await invoke<unknown>('claude_list_hooks'))
  return asRecord(response.hooks) as T
}

/** 批量更新 Hooks —— 同样拆包后端响应 */
export const updateHooks = async <T = UnknownRecord>(hooks: unknown): Promise<T> => {
  const response = asRecord(await invoke<unknown>('claude_update_hooks', { hooks }))
  return asRecord(response.hooks) as T
}

// ── Claude Budgets ──

/** 获取预算状态 */
export const getBudgetStatus = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('claude_get_budgets')
}

/** 设置预算 */
export const setBudget = async <T = UnknownRecord>(budgets: unknown): Promise<T> => {
  return invoke('claude_update_budgets', { budgets })
}

/** 重置预算（清空对象） */
export const resetBudget = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('claude_update_budgets', { budgets: {} })
}

// ── Claude Prompts ──

/** 列出提示词 */
export const listPrompts = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('claude_list_prompts')
}

/** 批量更新提示词 */
export const updatePrompts = async <T = UnknownRecord>(prompts: unknown): Promise<T> => {
  return invoke('claude_update_prompts', { prompts })
}

// ── Claude Profiles ──

export const listClaudeProfiles = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('claude_list_profiles')
}

export const exportClaudeProfiles = async <T = UnknownRecord>(includeSecrets = true): Promise<T> => {
  return invoke('claude_export_profiles', { includeSecrets })
}

export const getClaudeProfilesRaw = async (): Promise<RawFileGetResult> => {
  return invoke('claude_get_profiles_raw')
}

export const saveClaudeProfilesRaw = async (
  content: string,
  token: string,
  force = false,
): Promise<RawProfilesSaveResult> => {
  return invoke('claude_save_profiles_raw', { content, token, force })
}

export const getClaudeProfile = async <T = UnknownRecord>(name: string): Promise<T> => {
  return invoke('claude_get_profile', { name })
}

export const addClaudeProfile = async <T = UnknownRecord>(request: unknown): Promise<T> => {
  return invoke('claude_add_profile', { request })
}

export const updateClaudeProfile = async <T = UnknownRecord>(
  name: string,
  request: unknown,
): Promise<T> => {
  return invoke('claude_update_profile', { name, request })
}

export const deleteClaudeProfile = async <T = UnknownRecord>(name: string): Promise<T> => {
  return invoke('claude_delete_profile', { name })
}

export const applyClaudeProfile = async <T = UnknownRecord>(name: string): Promise<T> => {
  return invoke('claude_apply_profile', { name })
}

// ── Claude Auth ──

/** 列出已保存的 Claude 官方账号 */
export const listClaudeAuthAccounts = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('claude_list_auth_accounts')
}

/** 获取当前 Claude 官方登录状态 */
export const getClaudeAuthCurrent = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('claude_get_auth_current')
}

/** 保存当前 Claude 官方登录 */
export const saveClaudeAuth = async <T = UnknownRecord>(request: {
  name: string
  description?: string | null
  force?: boolean
}): Promise<T> => {
  return invoke('claude_save_auth', {
    name: request.name,
    description: request.description ?? null,
    force: request.force ?? false,
  })
}

/** 切换到指定 Claude 官方账号 */
export const switchClaudeAuth = async <T = UnknownRecord>(name: string): Promise<T> => {
  return invoke('claude_switch_auth', { name })
}

/** 删除指定 Claude 官方账号 */
export const deleteClaudeAuth = async <T = UnknownRecord>(name: string): Promise<T> => {
  return invoke('claude_delete_auth', { name })
}
