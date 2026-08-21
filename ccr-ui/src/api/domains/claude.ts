/**
 * Claude Domain —— Claude Code 平台 Settings / MCP / Agents / Slash / Plugins / Output Styles /
 * Statusline / Hooks / Budgets / Prompts / Profiles / Auth 全量 API
 *
 * 真迁移自 tauri.ts 第 4 分组。对应后端 commands::claude::* 命令。
 */

import { invoke } from '@/api/invokeRuntime'
import * as claudeGenerated from '../generated/claude'
import {
  asRecord,
  isRecord,
  pickArray,
  resolveName,
  resolveNameAndConfig,
  toOpenJsonValue as asOpenJson,
} from '../_shared'
import type { OpenJsonValueDto } from '@/types/generated/common/OpenJsonValueDto'
import type {
  Agent,
  AgentsResponse,
  BudgetStatus,
  ClaudeProfile,
  ClaudeProfileOffResult,
  ClaudeProfilesResponse,
  ClaudeSettingsData,
  HookMap,
  OutputStyle,
  Plugin,
  SlashCommandsResponse,
  StatuslineConfig,
} from '@/types'
import type {
  ConfigLayersResult,
  RawFileGetResult,
  RawFileSaveResult,
  RawProfilesSaveResult,
} from './configRawTypes'

export interface ClaudeProfilesExport {
  content: string
  filename: string
}

const settingsFrom = (value: OpenJsonValueDto): ClaudeSettingsData => {
  return asRecord(value) as ClaudeSettingsData
}

const agentsFrom = (value: OpenJsonValueDto): AgentsResponse => {
  const source = asRecord(value)
  return {
    agents: pickArray(source, 'agents').filter(isRecord) as unknown as Agent[],
    folders: Array.isArray(source.folders)
      ? source.folders.filter((folder): folder is string => typeof folder === 'string')
      : undefined,
  }
}

const slashCommandsFrom = (value: OpenJsonValueDto): SlashCommandsResponse => {
  const source = asRecord(value)
  return {
    commands: pickArray(source, 'commands').filter(isRecord) as unknown as SlashCommandsResponse['commands'],
    folders: Array.isArray(source.folders)
      ? source.folders.filter((folder): folder is string => typeof folder === 'string')
      : undefined,
  }
}

const pluginsFrom = (value: OpenJsonValueDto): Plugin[] => {
  return pickArray(value, 'plugins').filter(isRecord) as unknown as Plugin[]
}

const outputStylesFrom = (value: OpenJsonValueDto): OutputStyle[] => {
  return pickArray(value, 'styles').flatMap((item) => {
    if (!isRecord(item) || typeof item.name !== 'string' || typeof item.content !== 'string') {
      return []
    }
    return [{ name: item.name, content: item.content }]
  })
}

const statuslineFrom = (value: OpenJsonValueDto): StatuslineConfig => {
  const source = asRecord(value)
  return {
    command: typeof source.command === 'string' ? source.command : undefined,
    enabled: source.enabled === true,
  }
}

const hooksFrom = (value: OpenJsonValueDto): HookMap => {
  return asRecord(asRecord(value).hooks) as HookMap
}

const numberOrNull = (value: unknown): number | null => {
  return typeof value === 'number' ? value : null
}

const numberOrZero = (value: unknown): number => {
  return typeof value === 'number' ? value : 0
}

const budgetStatusFrom = (value: OpenJsonValueDto): BudgetStatus => {
  const source = asRecord(value)
  const costs = asRecord(source.currentCosts)
  const warnings = Array.isArray(source.warnings) ? source.warnings : []

  return {
    enabled: source.enabled === true,
    daily_limit: numberOrNull(source.dailyLimit),
    weekly_limit: numberOrNull(source.weeklyLimit),
    monthly_limit: numberOrNull(source.monthlyLimit),
    warn_threshold: numberOrZero(source.warnAtPercent),
    current_costs: {
      today: numberOrZero(costs.today),
      this_week: numberOrZero(costs.thisWeek),
      this_month: numberOrZero(costs.thisMonth),
    },
    warnings: warnings.filter(isRecord).map((warning) => ({
      period: typeof warning.period === 'string' ? warning.period : '',
      current_cost: numberOrZero(warning.currentCost),
      limit: numberOrZero(warning.limit),
      usage_percent: numberOrZero(warning.usagePercent),
    })),
    last_updated: typeof source.lastUpdated === 'string' ? source.lastUpdated : '',
  }
}

const budgetRequestToOpenJson = (value: unknown): OpenJsonValueDto => {
  const source = asRecord(value)
  return asOpenJson({
    enabled: typeof source.enabled === 'boolean' ? source.enabled : undefined,
    dailyLimit: source.daily_limit,
    weeklyLimit: source.weekly_limit,
    monthlyLimit: source.monthly_limit,
    warnAtPercent: source.warn_threshold,
  })
}

const profilesFrom = (value: OpenJsonValueDto): ClaudeProfilesResponse => {
  const source = asRecord(value)
  return {
    profiles: pickArray(source, 'profiles').filter(isRecord) as unknown as ClaudeProfile[],
    current_profile: typeof source.current_profile === 'string' ? source.current_profile : null,
    can_off: source.can_off === true,
  }
}

const profileFrom = (value: OpenJsonValueDto): ClaudeProfile => {
  return asRecord(value) as unknown as ClaudeProfile
}

const profilesExportFrom = (value: OpenJsonValueDto): ClaudeProfilesExport => {
  const source = asRecord(value)
  return {
    content: typeof source.content === 'string' ? source.content : '',
    filename: typeof source.filename === 'string' ? source.filename : '',
  }
}

const rawFileGetFrom = (value: OpenJsonValueDto): RawFileGetResult => {
  const source = asRecord(value)
  if (source.status === 'unsupported_environment' && typeof source.envType === 'string') {
    return { status: source.status, envType: source.envType }
  }
  if (
    source.status === 'ok'
    && typeof source.content === 'string'
    && typeof source.token === 'string'
    && typeof source.path === 'string'
    && typeof source.exists === 'boolean'
  ) {
    return {
      status: source.status,
      content: source.content,
      token: source.token,
      path: source.path,
      exists: source.exists,
    }
  }
  throw new Error('Claude profiles raw response is invalid')
}

const rawProfilesSaveFrom = (value: OpenJsonValueDto): RawProfilesSaveResult => {
  const source = asRecord(value)
  if (source.status === 'unsupported_environment' && typeof source.envType === 'string') {
    return { status: source.status, envType: source.envType }
  }
  if (source.status === 'conflict') return { status: source.status }
  if (source.status === 'activation_conflict' && typeof source.current === 'string') {
    return { status: source.status, current: source.current }
  }
  if (
    source.status === 'saved'
    && typeof source.token === 'string'
    && typeof source.profiles_count === 'number'
  ) {
    return { status: source.status, token: source.token, profiles_count: source.profiles_count }
  }
  if (
    source.status === 'invalid'
    && (source.kind === 'syntax' || source.kind === 'semantic')
    && typeof source.message === 'string'
  ) {
    return {
      status: source.status,
      kind: source.kind,
      message: source.message,
      line: typeof source.line === 'number' ? source.line : undefined,
      column: typeof source.column === 'number' ? source.column : undefined,
    }
  }
  throw new Error('Claude profiles save response is invalid')
}

// ── Claude Settings ──

/** 获取 Claude Code 全局设置 */
export const getClaudeSettings = async (): Promise<ClaudeSettingsData> => {
  return settingsFrom(await claudeGenerated.getClaudeSettings())
}

/** 更新 Claude Code 全局设置 */
export const updateClaudeSettings = async (settings: unknown): Promise<ClaudeSettingsData> => {
  return settingsFrom(await claudeGenerated.updateClaudeSettings(asOpenJson(settings)))
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
export const listMcpServers = async (): Promise<OpenJsonValueDto> => {
  return await claudeGenerated.listClaudeMcpServers()
}

/** 添加 Claude Code MCP 服务器 */
export const addMcpServer = async (
  nameOrRequest: string | object,
  config?: unknown,
  scope?: string,
): Promise<OpenJsonValueDto> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  delete resolvedConfig.scope
  const requestScope = scope ?? (isRecord(nameOrRequest) && typeof nameOrRequest.scope === 'string'
    ? nameOrRequest.scope
    : undefined)
  return await claudeGenerated.addClaudeMcpServer(name, asOpenJson(resolvedConfig), requestScope)
}

/** 更新 Claude Code MCP 服务器 */
export const updateMcpServer = async (
  nameOrRequest: string | object,
  config?: unknown,
  scope?: string,
): Promise<OpenJsonValueDto> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  delete resolvedConfig.scope
  const requestScope = scope ?? (isRecord(nameOrRequest) && typeof nameOrRequest.scope === 'string'
    ? nameOrRequest.scope
    : isRecord(config) && typeof config.scope === 'string'
      ? config.scope
      : undefined)
  return await claudeGenerated.updateClaudeMcpServer(name, asOpenJson(resolvedConfig), requestScope)
}

/** 删除 Claude Code MCP 服务器 */
export const deleteMcpServer = async (
  nameOrRequest: string | object,
  scope?: string,
): Promise<string> => {
  const name = resolveName(nameOrRequest)
  const requestScope = scope ?? (isRecord(nameOrRequest) && typeof nameOrRequest.scope === 'string'
    ? nameOrRequest.scope
    : undefined)
  return await claudeGenerated.deleteClaudeMcpServer(name, requestScope)
}

/** 切换 MCP 服务器启用/禁用（通过 disabled 字段实现） */
export const toggleMcpServer = async (
  nameOrRequest: string | object,
  disabled?: boolean,
): Promise<OpenJsonValueDto> => {
  if (typeof nameOrRequest === 'string') {
    return await claudeGenerated.updateClaudeMcpServer(
      nameOrRequest,
      { disabled: !!disabled },
    )
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
  return await claudeGenerated.updateClaudeMcpServer(name, { disabled: resolvedDisabled }, scope)
}

// ── Claude Agents ──

/** 列出 Claude Code Agents */
export const listAgents = async (): Promise<AgentsResponse> => {
  return agentsFrom(await claudeGenerated.listClaudeAgents())
}

/** 获取单个 Agent 详情（从列表过滤） */
export const getAgent = async (name: string): Promise<Agent | null> => {
  const agents = (await listAgents()).agents
  const found = agents.find((item) => {
    if (!isRecord(item)) {
      return false
    }
    return String(item.name ?? '') === name
  })
  return found ?? null
}

/** 添加 Claude Code Agent */
export const addAgent = async (
  nameOrRequest: string | object,
  config?: unknown,
): Promise<AgentsResponse> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return agentsFrom(await claudeGenerated.addClaudeAgent(name, asOpenJson(resolvedConfig)))
}

/** 更新 Claude Code Agent */
export const updateAgent = async (
  nameOrRequest: string | object,
  config?: unknown,
): Promise<AgentsResponse> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return agentsFrom(await claudeGenerated.updateClaudeAgent(name, asOpenJson(resolvedConfig)))
}

/** 删除 Claude Code Agent */
export const deleteAgent = async (
  nameOrRequest: string | object,
): Promise<string> => {
  const name = resolveName(nameOrRequest)
  return await claudeGenerated.deleteClaudeAgent(name)
}

/** 切换 Agent 启用/禁用 */
export const toggleAgent = async (
  nameOrRequest: string | object,
  enabled?: boolean,
): Promise<AgentsResponse> => {
  const name = resolveName(nameOrRequest)
  const request = asRecord(nameOrRequest)
  const resolvedEnabled =
    typeof enabled === 'boolean'
      ? enabled
      : typeof request.enabled === 'boolean'
        ? request.enabled
        : true
  return agentsFrom(await claudeGenerated.updateClaudeAgent(name, { enabled: resolvedEnabled }))
}

// ── Claude Slash Commands ──

/** 列出斜杠命令 */
export const listSlashCommands = async (): Promise<SlashCommandsResponse> => {
  return slashCommandsFrom(await claudeGenerated.listClaudeSlashCommands())
}

/** 添加斜杠命令 */
export const addSlashCommand = async (
  name: string,
  config: unknown,
): Promise<SlashCommandsResponse> => {
  return slashCommandsFrom(await claudeGenerated.addClaudeSlashCommand(name, asOpenJson(config)))
}

/** 更新斜杠命令 */
export const updateSlashCommand = async (
  name: string,
  config: unknown,
): Promise<SlashCommandsResponse> => {
  return slashCommandsFrom(await claudeGenerated.updateClaudeSlashCommand(name, asOpenJson(config)))
}

/** 删除斜杠命令 */
export const deleteSlashCommand = async (name: string): Promise<string> => {
  return await claudeGenerated.deleteClaudeSlashCommand(name)
}

/** 切换斜杠命令启用/禁用 */
export const toggleSlashCommand = async (
  name: string,
  enabled: boolean,
): Promise<SlashCommandsResponse> => {
  return slashCommandsFrom(await claudeGenerated.updateClaudeSlashCommand(name, { enabled }))
}

// ── Claude Plugins ──

/** 列出插件 */
export const listPlugins = async (): Promise<Plugin[]> => {
  return pluginsFrom(await claudeGenerated.listClaudePlugins())
}

/** 添加插件 */
export const addPlugin = async (
  nameOrRequest: string | object,
  config?: unknown,
): Promise<Plugin[]> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return pluginsFrom(await claudeGenerated.addClaudePlugin(name, asOpenJson(resolvedConfig)))
}

/** 更新插件 */
export const updatePlugin = async (
  name: string,
  config: unknown,
): Promise<Plugin[]> => {
  return pluginsFrom(await claudeGenerated.updateClaudePlugin(name, asOpenJson(config)))
}

/** 删除插件 */
export const deletePlugin = async (name: string): Promise<string> => {
  return await claudeGenerated.deleteClaudePlugin(name)
}

/** 切换插件启用/禁用 */
export const togglePlugin = async (
  nameOrRequest: string | object,
  enabled?: boolean,
): Promise<Plugin[]> => {
  const name = resolveName(nameOrRequest)
  const request = asRecord(nameOrRequest)
  const resolvedEnabled =
    typeof enabled === 'boolean'
      ? enabled
      : typeof request.enabled === 'boolean'
        ? request.enabled
        : true
  return pluginsFrom(await claudeGenerated.updateClaudePlugin(name, { enabled: resolvedEnabled }))
}

// ── Claude Output Styles ──

/** 获取输出样式列表 */
export const listOutputStyles = async (): Promise<OutputStyle[]> => {
  return outputStylesFrom(await claudeGenerated.getClaudeOutputStyles())
}

/** 获取单个输出样式（别名，后端同一 endpoint 返回整体） */
export const getOutputStyle = listOutputStyles

/** 创建输出样式（通过 update 实现） */
export const createOutputStyle = async (styles: unknown): Promise<OutputStyle[]> => {
  return outputStylesFrom(await claudeGenerated.updateClaudeOutputStyles(asOpenJson(styles)))
}

/** 更新输出样式 */
export const updateOutputStyle = async (
  nameOrStyles: string | object,
  patch?: Record<string, unknown>,
): Promise<OutputStyle[]> => {
  if (typeof nameOrStyles === 'string') {
    return outputStylesFrom(
      await claudeGenerated.updateClaudeOutputStyles({
        [nameOrStyles]: asOpenJson(patch ?? {}),
      }),
    )
  }
  return outputStylesFrom(
    await claudeGenerated.updateClaudeOutputStyles(asOpenJson(nameOrStyles)),
  )
}

/** 删除输出样式（通过 update 清空实现） */
export const deleteOutputStyle = async (_name: string): Promise<OutputStyle[]> => {
  return outputStylesFrom(await claudeGenerated.updateClaudeOutputStyles({}))
}

// ── Claude Statusline ──

/** 获取状态栏配置 */
export const getStatusline = async (): Promise<StatuslineConfig> => {
  return statuslineFrom(await claudeGenerated.getClaudeStatusline())
}

/** 更新状态栏配置 */
export const updateStatusline = async (statusline: unknown): Promise<StatuslineConfig> => {
  return statuslineFrom(await claudeGenerated.updateClaudeStatusline(asOpenJson(statusline)))
}

// ── Claude Hooks ──

/** 列出 Hooks —— 后端返回 `{ hooks: {...} }`，前端拆包到具体 hooks 对象 */
export const listHooks = async (): Promise<HookMap> => {
  return hooksFrom(await claudeGenerated.listClaudeHooks())
}

/** 批量更新 Hooks —— 同样拆包后端响应 */
export const updateHooks = async (hooks: unknown): Promise<HookMap> => {
  return hooksFrom(await claudeGenerated.updateClaudeHooks(asOpenJson(hooks)))
}

// ── Claude Budgets ──

/** 获取预算状态 */
export const getBudgetStatus = async (): Promise<BudgetStatus> => {
  return budgetStatusFrom(await claudeGenerated.getClaudeBudgets())
}

/** 设置预算 */
export const setBudget = async (budgets: unknown): Promise<OpenJsonValueDto> => {
  return await claudeGenerated.updateClaudeBudgets(budgetRequestToOpenJson(budgets))
}

/** 重置预算（清空对象） */
export const resetBudget = async (): Promise<OpenJsonValueDto> => {
  return await claudeGenerated.updateClaudeBudgets({})
}

// ── Claude Prompts ──

/** 列出提示词 */
export const listPrompts = async (): Promise<OpenJsonValueDto> => {
  return await claudeGenerated.listClaudePrompts()
}

/** 批量更新提示词 */
export const updatePrompts = async (prompts: unknown): Promise<OpenJsonValueDto> => {
  return await claudeGenerated.updateClaudePrompts(asOpenJson(prompts))
}

// ── Claude Profiles ──

export const listClaudeProfiles = async (): Promise<ClaudeProfilesResponse> => {
  return profilesFrom(await claudeGenerated.listClaudeProfiles())
}

export const exportClaudeProfiles = async (includeSecrets = true): Promise<ClaudeProfilesExport> => {
  return profilesExportFrom(await claudeGenerated.exportClaudeProfiles(includeSecrets))
}

export const getClaudeProfilesRaw = async (): Promise<RawFileGetResult> => {
  return rawFileGetFrom(await claudeGenerated.getClaudeProfilesRaw())
}

export const saveClaudeProfilesRaw = async (
  content: string,
  token: string,
  force = false,
): Promise<RawProfilesSaveResult> => {
  return rawProfilesSaveFrom(await claudeGenerated.saveClaudeProfilesRaw(content, token, force))
}

export const getClaudeProfile = async (name: string): Promise<ClaudeProfile> => {
  return profileFrom(await claudeGenerated.getClaudeProfile(name))
}

export const addClaudeProfile = async (request: unknown): Promise<ClaudeProfile> => {
  return profileFrom(await claudeGenerated.addClaudeProfile(asOpenJson(request)))
}

export const updateClaudeProfile = async (
  name: string,
  request: unknown,
): Promise<ClaudeProfile> => {
  return profileFrom(await claudeGenerated.updateClaudeProfile(name, asOpenJson(request)))
}

export const deleteClaudeProfile = async (name: string): Promise<OpenJsonValueDto> => {
  return await claudeGenerated.deleteClaudeProfile(name)
}

export const applyClaudeProfile = async (name: string): Promise<OpenJsonValueDto> => {
  return await claudeGenerated.applyClaudeProfile(name)
}

export const claudeProfileOff = async (): Promise<ClaudeProfileOffResult> => {
  const value = await claudeGenerated.claudeProfileOff()
  const source = asRecord(value)
  if (source.status === 'unsupported_environment') {
    throw new Error('Claude profile off is only available in the local environment')
  }
  return {
    ok: source.ok === true,
    changed: source.changed === true,
    previous_profile: typeof source.previous_profile === 'string' ? source.previous_profile : null,
    runtime_mode: typeof source.runtime_mode === 'string' ? source.runtime_mode : 'official_auth',
    warnings: Array.isArray(source.warnings)
      ? source.warnings.filter((item): item is string => typeof item === 'string')
      : [],
    remaining_suppressors: pickArray(source, 'remaining_suppressors').filter(isRecord) as unknown as ClaudeProfileOffResult['remaining_suppressors'],
    cleared_managed_sources: Array.isArray(source.cleared_managed_sources)
      ? source.cleared_managed_sources.filter((item): item is string => typeof item === 'string')
      : [],
  }
}

// ── Claude Auth ──

export {
  claudeAuthOff,
  deleteClaudeAuth,
  getClaudeAuthCurrent,
  listClaudeAuthAccounts,
  saveClaudeAuth,
  switchClaudeAuth,
  type ClaudeAuthSaveRequest,
} from '../generated/claudeAuth'
