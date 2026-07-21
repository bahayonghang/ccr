/**
 * Codex Domain —— Codex Platform Profiles / Settings / MCP / Agents / Agent Sources /
 * Models / Sessions / Auth / OAuth / Dashboard / Usage 全量 API
 *
 * 真迁移自 tauri.ts 第 5 分组。对应后端 commands::codex::* / commands::codex_* 命令集。
 *
 * Codex 平台不支持 Slash Commands / Plugins，保留相应桩函数返回"不支持"信号，
 * 业务方按统一接口调用不会抛错；Auth/Agent 相关 helper（context/mutation/name）
 * 迁移时一并带入本文件作为内部实现细节。
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
import type {
  ConfigLayersResult,
  RawFileGetResult,
  RawFileSaveResult,
  RawProfilesSaveResult,
} from './configRawTypes'

// ── Internal Codex Agent helpers —— 与 _shared 中的 resolveName/resolveNameAndConfig 协同 ──

function resolveCodexAgentContext(value: unknown): UnknownRecord | undefined {
  const request = asRecord(value)
  if (Object.keys(request).length === 0) {
    return undefined
  }
  return request
}

function resolveCodexAgentMutation(
  arg1: string | object,
  arg2?: unknown,
): { name: string; config: UnknownRecord; context?: UnknownRecord } {
  if (typeof arg1 === 'string') {
    return {
      name: arg1,
      config: asRecord(arg2),
    }
  }

  const request = { ...asRecord(arg1) }
  const name = String(request.name ?? request.id ?? '')
  const context = resolveCodexAgentContext(request.context ?? request.agentContext)
  delete request.name
  delete request.id
  delete request.context
  delete request.agentContext
  return { name, config: request, context }
}

function resolveCodexAgentNameAndContext(arg1: string | object): {
  name: string
  context?: UnknownRecord
} {
  if (typeof arg1 === 'string') {
    return { name: arg1 }
  }

  const request = asRecord(arg1)
  return {
    name: String(request.name ?? request.id ?? ''),
    context: resolveCodexAgentContext(request.context ?? request.agentContext),
  }
}

// ── Codex Profiles / Settings ──

/** 列出 Codex Profiles */
export const listCodexProfiles = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('codex_list_profiles')
}

/** 获取 Codex 配置 */
export const exportCodexProfiles = async <T = UnknownRecord>(includeSecrets = true): Promise<T> => {
  return invoke('codex_export_profiles', { includeSecrets })
}

export const getCodexProfilesRaw = async (): Promise<RawFileGetResult> => {
  return invoke('codex_get_profiles_raw')
}

export const saveCodexProfilesRaw = async (
  content: string,
  token: string,
  force = false,
): Promise<RawProfilesSaveResult> => {
  return invoke('codex_save_profiles_raw', { content, token, force })
}

export const getCodexConfig = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('codex_get_settings')
}

/** 更新 Codex 配置 */
export const updateCodexConfig = async <T = UnknownRecord>(settings: unknown): Promise<T> => {
  return invoke('codex_update_settings', { settings })
}

export const getCodexConfigRaw = async (): Promise<RawFileGetResult> => {
  return invoke('codex_get_config_raw_text')
}

export const saveCodexConfigRaw = async (
  content: string,
  token: string,
): Promise<RawFileSaveResult> => {
  return invoke('codex_save_config_raw_text', { content, token })
}

export const listCodexConfigLayers = async (): Promise<ConfigLayersResult> => {
  return invoke('codex_list_config_layers')
}

export type { ConfigLayer, ConfigLayersResult, RawFileGetResult, RawFileSaveResult } from './configRawTypes'

// ── Codex MCP Servers ──

export const listCodexMcpServers = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('codex_list_mcp_servers')
}

export const addCodexMcpServer = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown,
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('codex_add_mcp_server', { name, config: resolvedConfig })
}

export const updateCodexMcpServer = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown,
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return invoke('codex_update_mcp_server', { name, config: resolvedConfig })
}

export const deleteCodexMcpServer = async <T = UnknownRecord>(
  nameOrRequest: string | object,
): Promise<T> => {
  const name = resolveName(nameOrRequest)
  return invoke('codex_delete_mcp_server', { name })
}

// ── Codex Agents ──

/** 列出 Codex Agents */
export const listCodexAgents = async <T = UnknownRecord>(context?: unknown): Promise<T> => {
  return invoke('codex_list_agents', {
    context: resolveCodexAgentContext(context),
  })
}

/** 添加 Codex Agent */
export const addCodexAgent = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown,
): Promise<T> => {
  const { name, config: resolvedConfig, context } = resolveCodexAgentMutation(nameOrRequest, config)
  return invoke('codex_add_agent', { name, config: resolvedConfig, context })
}

/** 更新 Codex Agent */
export const updateCodexAgent = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  config?: unknown,
): Promise<T> => {
  const { name, config: resolvedConfig, context } = resolveCodexAgentMutation(nameOrRequest, config)
  return invoke('codex_update_agent', { name, config: resolvedConfig, context })
}

/** 删除 Codex Agent */
export const deleteCodexAgent = async <T = UnknownRecord>(
  nameOrRequest: string | object,
): Promise<T> => {
  const { name, context } = resolveCodexAgentNameAndContext(nameOrRequest)
  return invoke('codex_delete_agent', { name, context })
}

/** Codex custom agents 不支持 enabled/disabled 切换，直接拒绝 */
export const toggleCodexAgent = async <T = UnknownRecord>(
  nameOrRequest: string | object,
  _enabled?: boolean,
): Promise<T> => {
  const { name } = resolveCodexAgentNameAndContext(nameOrRequest)
  return Promise.reject(new Error(`Codex agent '${name}' does not support toggle`))
}

/** 重命名 Codex Agent */
export const renameCodexAgent = async <T = UnknownRecord>(payload: {
  name: string
  newName: string
  context?: unknown
}): Promise<T> => {
  return invoke('codex_rename_agent', {
    name: payload.name,
    newName: payload.newName,
    context: resolveCodexAgentContext(payload.context),
  })
}

/** 复制 Codex Agent */
export const copyCodexAgent = async <T = UnknownRecord>(payload: {
  name: string
  sourceContext?: unknown
  targetContext?: unknown
  targetName?: string
}): Promise<T> => {
  return invoke('codex_copy_agent', {
    name: payload.name,
    sourceContext: resolveCodexAgentContext(payload.sourceContext),
    targetContext: resolveCodexAgentContext(payload.targetContext),
    targetName: payload.targetName ?? null,
  })
}

/** 校验 Codex Agent 原始 TOML */
export const validateCodexAgentToml = async <T = UnknownRecord>(payload: {
  name: string
  context?: unknown
}): Promise<T> => {
  return invoke('codex_validate_agent_toml', {
    name: payload.name,
    context: resolveCodexAgentContext(payload.context),
  })
}

// ── Codex Agent Sources（GitHub 远程源） ──

export const listCodexAgentSources = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('codex_list_agent_sources')
}

export const addCodexAgentSource = async <T = UnknownRecord>(url: string): Promise<T> => {
  return invoke('codex_add_agent_source', { request: { url } })
}

export const removeCodexAgentSource = async <T = UnknownRecord>(sourceId: string): Promise<T> => {
  return invoke('codex_remove_agent_source', { sourceId })
}

export const syncCodexAgentSource = async <T = UnknownRecord>(sourceId: string): Promise<T> => {
  return invoke('codex_sync_agent_source', { sourceId })
}

export const getCodexAgentSourceCatalog = async <T = UnknownRecord>(
  sourceId: string,
): Promise<T> => {
  return invoke('codex_get_agent_source_catalog', { sourceId })
}

export const installCodexSourceAgent = async <T = UnknownRecord>(payload: {
  sourceId: string
  agentId: string
  targetName?: string | null
  conflictMode?: string | null
}): Promise<T> => {
  return invoke('codex_install_source_agent', {
    request: {
      sourceId: payload.sourceId,
      agentId: payload.agentId,
      targetName: payload.targetName ?? null,
      conflictMode: payload.conflictMode ?? null,
    },
  })
}

export const syncCodexSourceInstall = async <T = UnknownRecord>(installId: string): Promise<T> => {
  return invoke('codex_sync_source_install', { request: { installId } })
}

export const forceSyncCodexSourceInstall = async <T = UnknownRecord>(
  installId: string,
): Promise<T> => {
  return invoke('codex_sync_source_install', { request: { installId, force: true } })
}

export const acceptLocalCodexSourceInstall = async <T = UnknownRecord>(
  installId: string,
): Promise<T> => {
  return invoke('codex_accept_local_source_install', { request: { installId } })
}

export const untrackCodexSourceInstall = async <T = UnknownRecord>(
  installId: string,
): Promise<T> => {
  return invoke('codex_untrack_source_install', { request: { installId } })
}

// ── Codex Models ──

export const listCodexModels = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('codex_list_models')
}

// ── Codex Profile 管理（CCR profiles.toml） ──

export const addCodexProfile = async <T = UnknownRecord>(
  profileOrName: string | object,
  config?: unknown,
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(profileOrName, config)
  return invoke('codex_add_profile', { name, config: resolvedConfig })
}

export const updateCodexProfile = async <T = UnknownRecord>(
  profileOrName: string | object,
  config?: unknown,
): Promise<T> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(profileOrName, config)
  return invoke('codex_update_profile', { name, config: resolvedConfig })
}

export const deleteCodexProfile = async <T = UnknownRecord>(
  nameOrRequest: string | object,
): Promise<T> => {
  const name = resolveName(nameOrRequest)
  return invoke('codex_delete_profile', { name })
}

/** 获取 Codex Profile 详情（从列表过滤） */
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

export const getCodexProfileEnv = async <T = UnknownRecord>(name: string): Promise<T> => {
  return invoke('codex_get_profile_env', { name })
}

export const applyCodexProfile = async <T = UnknownRecord>(name: string): Promise<T> => {
  return invoke('codex_apply_profile', { name })
}

// ── Codex Sessions ──

export const listCodexSessions = async <T = UnknownRecord>(options?: {
  limit?: number
  query?: string
}): Promise<T> => {
  return invoke('codex_list_sessions', {
    limit: options?.limit,
    query: options?.query,
  })
}

export const getCodexSessionDetail = async <T = UnknownRecord>(
  filePath: string,
  messageLimit?: number,
): Promise<T> => {
  return invoke('codex_get_session_detail', { filePath, messageLimit })
}

export const exportCodexSession = async <T = UnknownRecord>(
  filePath: string,
  maxMessages?: number,
): Promise<T> => {
  return invoke('codex_export_session', { filePath, maxMessages })
}

export const cloneCodexSession = async <T = UnknownRecord>(filePath: string): Promise<T> => {
  return invoke('codex_clone_session', { filePath })
}

export const deleteCodexSession = async <T = UnknownRecord>(filePath: string): Promise<T> => {
  return invoke('codex_delete_session', { filePath })
}

// ── Codex Auth 管理 ──

export const listCodexAuthAccounts = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('codex_list_auth_accounts')
}

export const getCodexAuthCurrent = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('codex_get_auth_current')
}

export const saveCodexAuth = async <T = UnknownRecord>(data: unknown): Promise<T> => {
  return invoke('codex_save_auth', asRecord(data))
}

export const switchCodexAuth = async <T = UnknownRecord>(name: string): Promise<T> => {
  return invoke('codex_switch_auth', { name })
}

export const deleteCodexAuth = async <T = UnknownRecord>(name: string): Promise<T> => {
  return invoke('codex_delete_auth', { name })
}

export const renameCodexAuth = async <T = UnknownRecord>(
  oldName: string,
  newName: string,
  force = false,
): Promise<T> => {
  return invoke('codex_rename_auth', { oldName, newName, force })
}

// ── Codex Tray / Process ──

export const getCodexTraySnapshot = async <T = UnknownRecord>(force?: boolean): Promise<T> => {
  return invoke('codex_get_tray_snapshot', { force })
}

export const detectCodexProcess = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('codex_detect_process')
}

// ── Codex OAuth ──

export const codexOAuthLoginStart = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('codex_oauth_login_start')
}

export const codexOAuthLoginCompleted = async <T = UnknownRecord>(
  loginId: string,
  preferredAccountName?: string | null,
): Promise<T> => {
  return invoke('codex_oauth_login_completed', {
    loginId,
    preferredAccountName: preferredAccountName ?? null,
  })
}

export const codexOAuthLoginCancel = async (loginId?: string | null): Promise<void> => {
  await invoke('codex_oauth_login_cancel', { loginId: loginId ?? null })
}

export const codexOAuthSubmitCallbackUrl = async (
  loginId: string,
  callbackUrl: string,
): Promise<void> => {
  await invoke('codex_oauth_submit_callback_url', { loginId, callbackUrl })
}

export const codexIsOAuthPortInUse = async <T = boolean>(): Promise<T> => {
  return invoke('codex_is_oauth_port_in_use')
}

export const codexReleaseOAuthPort = async <T = number>(): Promise<T> => {
  return invoke('codex_release_oauth_port')
}

export const codexOpenExternalUrl = async (url: string): Promise<void> => {
  await invoke('codex_open_external_url', { url })
}

export const codexImportAuthPayload = async <T = UnknownRecord>(payload: unknown): Promise<T> => {
  return invoke('codex_import_auth_payload', { payload: asRecord(payload) })
}

export const codexImportAuthFromLocal = async <T = UnknownRecord>(
  preferredAccountName?: string | null,
): Promise<T> => {
  return invoke('codex_import_auth_from_local', {
    preferredAccountName: preferredAccountName ?? null,
  })
}

export const codexAddAuthWithApiKey = async <T = UnknownRecord>(payload: unknown): Promise<T> => {
  return invoke('codex_add_auth_with_api_key', { payload: asRecord(payload) })
}

export const codexListModelProviders = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('codex_list_model_providers')
}

export const codexSaveModelProvider = async <T = UnknownRecord>(payload: unknown): Promise<T> => {
  return invoke('codex_save_model_provider', { payload: asRecord(payload) })
}

export const codexDeleteModelProvider = async <T = UnknownRecord>(
  providerId: string,
): Promise<T> => {
  return invoke('codex_delete_model_provider', { providerId })
}

// ── Codex Dashboard 类型 ──

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
    current?: {
      name?: string | null
      account_id?: string
      email?: string
      plan_type?: string
      last_refresh?: string | null
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
  options?: CodexUsageCommandOptions,
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

// ── 平台限制 —— Codex 不支持 Slash Commands 与 Plugins，保留桩函数 ──

export const listCodexSlashCommands = async <T = UnknownRecord>(): Promise<T> => {
  return { commands: [], folders: [] } as T
}

export const addCodexSlashCommand = async <T = UnknownRecord>(
  _name: string,
  _config: unknown,
): Promise<T> => {
  return { success: false, message: 'Codex 平台不支持斜杠命令' } as T
}

export const updateCodexSlashCommand = async <T = UnknownRecord>(
  _name: string,
  _config: unknown,
): Promise<T> => {
  return { success: false, message: 'Codex 平台不支持斜杠命令' } as T
}

export const deleteCodexSlashCommand = async <T = UnknownRecord>(_name: string): Promise<T> => {
  return { success: false, message: 'Codex 平台不支持斜杠命令' } as T
}

export const toggleCodexSlashCommand = async <T = UnknownRecord>(
  _name: string,
  _enabled: boolean,
): Promise<T> => {
  return { success: false, message: 'Codex 平台不支持斜杠命令' } as T
}

export const listCodexPlugins = async <T = UnknownRecord>(): Promise<T> => {
  return { plugins: [] } as T
}

export const addCodexPlugin = async <T = UnknownRecord>(
  _name: string,
  _config: unknown,
): Promise<T> => {
  return { success: false, message: 'Codex 平台不支持插件' } as T
}

export const updateCodexPlugin = async <T = UnknownRecord>(
  _pluginOrName: string | object,
  _config?: unknown,
): Promise<T> => {
  return { success: false, message: 'Codex 平台不支持插件' } as T
}

export const deleteCodexPlugin = async <T = UnknownRecord>(_name: string): Promise<T> => {
  return { success: false, message: 'Codex 平台不支持插件' } as T
}

export const toggleCodexPlugin = async <T = UnknownRecord>(
  _name: string,
  _enabled: boolean,
): Promise<T> => {
  return { success: false, message: 'Codex 平台不支持插件' } as T
}
