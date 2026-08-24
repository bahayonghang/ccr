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

import { invoke } from '@/api/invokeRuntime'
import * as codexClient from '../generated/codex'
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
import type { OpenJsonValueDto } from '@/types/generated/common/OpenJsonValueDto'
import type {
  CodexAccountQuota,
  CodexAgentMutationResponse,
  CodexAgentSourceCatalogResponse,
  CodexAgentSourceRecord,
  CodexAgentSourcesResponse,
  CodexAgentsResponse,
  CodexCloneSessionResponse,
  CodexConfig,
  CodexMcpServersResponse,
  CodexModelsResponse,
  CodexProfile,
  CodexProfileOffResult,
  CodexProfilesResponse,
  CodexSessionDetailResponse,
  CodexSessionExportResponse,
  CodexSessionsResponse,
  CodexTraySnapshot,
  CodexUsageResponse,
} from '@/types/codex'

// ── Internal Codex Agent helpers —— 与 _shared 中的 resolveName/resolveNameAndConfig 协同 ──

function toOpenJson(value: unknown): OpenJsonValueDto {
  if (value === null || typeof value === 'string' || typeof value === 'boolean') return value
  if (typeof value === 'number' && Number.isFinite(value)) return value
  if (Array.isArray(value)) return value.map(toOpenJson)
  if (isRecord(value)) {
    return Object.fromEntries(
      Object.entries(value)
        .filter(([, item]) => item !== undefined)
        .map(([key, item]) => [key, toOpenJson(item)]),
    )
  }
  throw new TypeError('Codex command payload must be JSON-compatible')
}

function objectResponse(value: OpenJsonValueDto, label: string): object {
  if (value === null || Array.isArray(value) || typeof value !== 'object') {
    throw new Error(`${label} response is not an object`)
  }
  return value
}

function arrayResponse(value: OpenJsonValueDto, label: string): OpenJsonValueDto[] {
  if (!Array.isArray(value)) throw new Error(`${label} response is not an array`)
  return value
}

const recordOf = (value: object): UnknownRecord => asRecord(value)

const isCodexConfig = (value: object): value is CodexConfig => !Array.isArray(value)

const isCodexProfile = (value: object): value is CodexProfile =>
  typeof recordOf(value).name === 'string'

const isCodexProfilesResponse = (value: object): value is CodexProfilesResponse => {
  const source = recordOf(value)
  return Array.isArray(source.profiles)
    && source.profiles.every(item => isRecord(item) && isCodexProfile(item))
    && (source.can_off === undefined || typeof source.can_off === 'boolean')
}

const isCodexModelsResponse = (value: object): value is CodexModelsResponse => {
  const source = recordOf(value)
  return Array.isArray(source.models)
    && Array.isArray(source.builtin_models)
    && Array.isArray(source.custom_models)
}

const isCodexMcpServersResponse = (value: object): value is CodexMcpServersResponse =>
  Array.isArray(recordOf(value).servers)

const isCodexAgentsResponse = (value: object): value is CodexAgentsResponse => {
  const source = recordOf(value)
  return isRecord(source.context)
    && Array.isArray(source.agents)
    && Array.isArray(source.diagnostics)
}

const isCodexAgentMutationResponse = (value: object): value is CodexAgentMutationResponse => {
  const source = recordOf(value)
  return isRecord(source.context) && isRecord(source.agent)
}

const isCodexAgentSourceRecord = (value: object): value is CodexAgentSourceRecord => {
  const source = recordOf(value)
  return typeof source.id === 'string'
    && typeof source.repoUrl === 'string'
    && typeof source.owner === 'string'
    && typeof source.repo === 'string'
}

const isCodexAgentSourcesResponse = (value: object): value is CodexAgentSourcesResponse => {
  const sources = recordOf(value).sources
  return Array.isArray(sources)
    && sources.every(item => isRecord(item) && isCodexAgentSourceRecord(item))
}

const isCodexAgentSourceCatalogResponse = (
  value: object,
): value is CodexAgentSourceCatalogResponse => {
  const source = recordOf(value)
  return isRecord(source.source)
    && isCodexAgentSourceRecord(source.source)
    && Array.isArray(source.agents)
    && Array.isArray(source.diagnostics)
    && Array.isArray(source.installs)
}

const isCodexSessionsResponse = (value: object): value is CodexSessionsResponse =>
  Array.isArray(recordOf(value).sessions)

const isCodexSessionDetailResponse = (value: object): value is CodexSessionDetailResponse => {
  const source = recordOf(value)
  return isRecord(source.session)
    && Array.isArray(source.messages)
    && typeof source.clipped === 'boolean'
    && typeof source.message_limit === 'number'
}

const isCodexSessionExportResponse = (value: object): value is CodexSessionExportResponse => {
  const source = recordOf(value)
  return typeof source.session_id === 'string'
    && typeof source.file_name === 'string'
    && typeof source.content === 'string'
    && typeof source.truncated === 'boolean'
    && typeof source.max_messages === 'number'
}

const isCodexCloneSessionResponse = (value: object): value is CodexCloneSessionResponse => {
  const source = recordOf(value)
  return typeof source.message === 'string' && isRecord(source.session)
}

const isCodexTraySnapshot = (value: object): value is CodexTraySnapshot => {
  const source = recordOf(value)
  return typeof source.fetched_at === 'string'
    && typeof source.runtime_mode === 'string'
    && Array.isArray(source.accounts)
}

const isCodexDashboardOverview = (value: object): value is CodexDashboardOverview => {
  const source = recordOf(value)
  return isRecord(source.auth)
    && isRecord(source.profiles)
    && isRecord(source.config)
    && isRecord(source.inventory)
}

const optionalString = (value: unknown): string | null | undefined =>
  typeof value === 'string' ? value : value == null ? (value as null | undefined) : String(value)

/** 只留下仪表盘用到的字段，让 IPC 多余键在 parse 后可被回收。 */
function pickCodexDashboardOverview(value: object): CodexDashboardOverview {
  const source = recordOf(value)
  const auth = isRecord(source.auth) ? recordOf(source.auth) : {}
  const currentAuth = isRecord(auth.current) ? recordOf(auth.current) : null
  const profiles = isRecord(source.profiles) ? recordOf(source.profiles) : {}
  const currentProfile = isRecord(profiles.current) ? recordOf(profiles.current) : null
  const config = isRecord(source.config) ? recordOf(source.config) : {}
  const inventory = isRecord(source.inventory) ? recordOf(source.inventory) : {}
  return {
    auth: {
      logged_in: Boolean(auth.logged_in),
      login_state: typeof auth.login_state === 'string' ? auth.login_state : undefined,
      store: typeof auth.store === 'string' ? auth.store : undefined,
      saved_accounts_total: Number(auth.saved_accounts_total) || 0,
      current: currentAuth
        ? {
            name: optionalString(currentAuth.name),
            account_id: typeof currentAuth.account_id === 'string' ? currentAuth.account_id : undefined,
            email: typeof currentAuth.email === 'string' ? currentAuth.email : undefined,
            plan_type: typeof currentAuth.plan_type === 'string' ? currentAuth.plan_type : undefined,
            last_refresh: optionalString(currentAuth.last_refresh),
          }
        : null,
    },
    profiles: {
      current_profile: optionalString(profiles.current_profile),
      total: Number(profiles.total) || 0,
      enabled_total: Number(profiles.enabled_total) || 0,
      disabled_total: Number(profiles.disabled_total) || 0,
      current: currentProfile,
    },
    config: {
      model: optionalString(config.model),
      model_provider: optionalString(config.model_provider),
      approval_policy: optionalString(config.approval_policy),
      sandbox_mode: optionalString(config.sandbox_mode),
      model_reasoning_effort: optionalString(config.model_reasoning_effort),
      model_reasoning_summary: optionalString(config.model_reasoning_summary),
      web_search: optionalString(config.web_search),
      disable_response_storage:
        typeof config.disable_response_storage === 'boolean' ? config.disable_response_storage : null,
    },
    inventory: {
      mcp_servers_total: Number(inventory.mcp_servers_total) || 0,
      agents_total: Number(inventory.agents_total) || 0,
      sessions_total: Number(inventory.sessions_total) || 0,
      config_profiles_total: Number(inventory.config_profiles_total) || 0,
    },
  }
}

const isCodexDashboardUsageSummary = (
  value: object,
): value is CodexDashboardUsageSummary => {
  const source = recordOf(value)
  return typeof source.freshness === 'string'
    && isRecord(source.five_hour)
    && isRecord(source.seven_day)
    && isRecord(source.all_time)
}

const isCodexUsageResponse = (value: object): value is CodexUsageResponse => {
  const source = recordOf(value)
  return isRecord(source.five_hour)
    && isRecord(source.seven_day)
    && isRecord(source.all_time)
    && isRecord(source.by_model)
}

const isCodexAccountQuota = (value: object): value is CodexAccountQuota => {
  const source = recordOf(value)
  return typeof source.account_name === 'string' && typeof source.fetched_at === 'string'
}

function rawFileGetFrom(value: OpenJsonValueDto): RawFileGetResult {
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
  throw new Error('Codex profiles raw response is invalid')
}

function rawProfilesSaveFrom(value: OpenJsonValueDto): RawProfilesSaveResult {
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
  throw new Error('Codex profiles save response is invalid')
}

function resolveCodexAgentContext(
  value: unknown,
): codexClient.CodexAgentContextRequest | undefined {
  const request = asRecord(value)
  if (Object.keys(request).length === 0) {
    return undefined
  }
  return {
    mode: typeof request.mode === 'string' ? request.mode : undefined,
    projectRoot: typeof request.projectRoot === 'string' ? request.projectRoot : undefined,
  }
}

function resolveCodexAgentMutation(
  arg1: string | object,
  arg2?: unknown,
): {
  name: string
  config: OpenJsonValueDto
  context?: codexClient.CodexAgentContextRequest
} {
  if (typeof arg1 === 'string') {
    return {
      name: arg1,
      config: toOpenJson(asRecord(arg2)),
    }
  }

  const request = { ...asRecord(arg1) }
  const name = String(request.name ?? request.id ?? '')
  const context = resolveCodexAgentContext(request.context ?? request.agentContext)
  delete request.name
  delete request.id
  delete request.context
  delete request.agentContext
  return { name, config: toOpenJson(request), context }
}

function resolveCodexAgentNameAndContext(arg1: string | object): {
  name: string
  context?: codexClient.CodexAgentContextRequest
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
export const listCodexProfiles = async (): Promise<CodexProfilesResponse> => {
  const value = objectResponse(await codexClient.listCodexProfiles(), 'Codex profiles')
  if (!isCodexProfilesResponse(value)) throw new Error('Codex profiles response is invalid')
  return value
}

/** 获取 Codex 配置 */
export const exportCodexProfiles = async (
  includeSecrets = true,
): Promise<{ content: string; filename: string }> => {
  const value = objectResponse(await codexClient.exportCodexProfiles(includeSecrets), 'Codex export')
  const source = recordOf(value)
  if (typeof source.content !== 'string' || typeof source.filename !== 'string') {
    throw new Error('Codex export response is invalid')
  }
  return { content: source.content, filename: source.filename }
}

export const getCodexProfilesRaw = async (): Promise<RawFileGetResult> => {
  return rawFileGetFrom(await codexClient.getCodexProfilesRaw())
}

export const saveCodexProfilesRaw = async (
  content: string,
  token: string,
  force = false,
): Promise<RawProfilesSaveResult> => {
  return rawProfilesSaveFrom(await codexClient.saveCodexProfilesRaw(content, token, force))
}

export const getCodexConfig = async (): Promise<CodexConfig> => {
  const value = objectResponse(await codexClient.getCodexSettings(), 'Codex config')
  if (!isCodexConfig(value)) throw new Error('Codex config response is invalid')
  return value
}

/** 更新 Codex 配置 */
export const updateCodexConfig = async (settings: unknown): Promise<CodexConfig> => {
  const value = objectResponse(
    await codexClient.updateCodexSettings(toOpenJson(settings)),
    'Codex config update',
  )
  if (!isCodexConfig(value)) throw new Error('Codex config update response is invalid')
  return value
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

export const listCodexMcpServers = async (): Promise<CodexMcpServersResponse> => {
  const value = objectResponse(await codexClient.listCodexMcpServers(), 'Codex MCP servers')
  if (!isCodexMcpServersResponse(value)) throw new Error('Codex MCP servers response is invalid')
  return value
}

export const addCodexMcpServer = async (
  nameOrRequest: string | object,
  config?: unknown,
): Promise<OpenJsonValueDto> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return codexClient.addCodexMcpServer(name, toOpenJson(resolvedConfig))
}

export const updateCodexMcpServer = async (
  nameOrRequest: string | object,
  config?: unknown,
): Promise<OpenJsonValueDto> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(nameOrRequest, config)
  return codexClient.updateCodexMcpServer(name, toOpenJson(resolvedConfig))
}

export const deleteCodexMcpServer = async (
  nameOrRequest: string | object,
): Promise<string> => {
  const name = resolveName(nameOrRequest)
  return codexClient.deleteCodexMcpServer(name)
}

// ── Codex Agents ──

/** 列出 Codex Agents */
export const listCodexAgents = async (context?: unknown): Promise<CodexAgentsResponse> => {
  const value = objectResponse(
    await codexClient.listCodexAgents(resolveCodexAgentContext(context)),
    'Codex agents',
  )
  if (!isCodexAgentsResponse(value)) throw new Error('Codex agents response is invalid')
  return value
}

/** 添加 Codex Agent */
export const addCodexAgent = async (
  nameOrRequest: string | object,
  config?: unknown,
): Promise<CodexAgentMutationResponse> => {
  const { name, config: resolvedConfig, context } = resolveCodexAgentMutation(nameOrRequest, config)
  const value = objectResponse(
    await codexClient.addCodexAgent(name, resolvedConfig, context),
    'Codex agent add',
  )
  if (!isCodexAgentMutationResponse(value)) throw new Error('Codex agent add response is invalid')
  return value
}

/** 更新 Codex Agent */
export const updateCodexAgent = async (
  nameOrRequest: string | object,
  config?: unknown,
): Promise<CodexAgentMutationResponse> => {
  const { name, config: resolvedConfig, context } = resolveCodexAgentMutation(nameOrRequest, config)
  const value = objectResponse(
    await codexClient.updateCodexAgent(name, resolvedConfig, context),
    'Codex agent update',
  )
  if (!isCodexAgentMutationResponse(value)) throw new Error('Codex agent update response is invalid')
  return value
}

/** 删除 Codex Agent */
export const deleteCodexAgent = async (
  nameOrRequest: string | object,
): Promise<string> => {
  const { name, context } = resolveCodexAgentNameAndContext(nameOrRequest)
  return codexClient.deleteCodexAgent(name, context)
}

/** Codex custom agents 不支持 enabled/disabled 切换，直接拒绝 */
export const toggleCodexAgent = async (
  nameOrRequest: string | object,
  _enabled?: boolean,
): Promise<never> => {
  const { name } = resolveCodexAgentNameAndContext(nameOrRequest)
  return Promise.reject(new Error(`Codex agent '${name}' does not support toggle`))
}

/** 重命名 Codex Agent */
export const renameCodexAgent = async (payload: {
  name: string
  newName: string
  context?: unknown
}): Promise<CodexAgentMutationResponse> => {
  const value = objectResponse(
    await codexClient.renameCodexAgent(
      payload.name,
      payload.newName,
      resolveCodexAgentContext(payload.context),
    ),
    'Codex agent rename',
  )
  if (!isCodexAgentMutationResponse(value)) throw new Error('Codex agent rename response is invalid')
  return value
}

/** 复制 Codex Agent */
export const copyCodexAgent = async (payload: {
  name: string
  sourceContext?: unknown
  targetContext?: unknown
  targetName?: string
}): Promise<CodexAgentMutationResponse> => {
  const value = objectResponse(
    await codexClient.copyCodexAgent(
      payload.name,
      resolveCodexAgentContext(payload.sourceContext),
      resolveCodexAgentContext(payload.targetContext),
      payload.targetName,
    ),
    'Codex agent copy',
  )
  if (!isCodexAgentMutationResponse(value)) throw new Error('Codex agent copy response is invalid')
  return value
}

/** 校验 Codex Agent 原始 TOML */
export const validateCodexAgentToml = async (payload: {
  name: string
  context?: unknown
}): Promise<CodexAgentMutationResponse> => {
  const value = objectResponse(
    await codexClient.validateCodexAgentToml(
      payload.name,
      resolveCodexAgentContext(payload.context),
    ),
    'Codex agent validation',
  )
  if (!isCodexAgentMutationResponse(value)) {
    throw new Error('Codex agent validation response is invalid')
  }
  return value
}

// ── Codex Agent Sources（GitHub 远程源） ──

export const listCodexAgentSources = async (): Promise<CodexAgentSourcesResponse> => {
  const value = objectResponse(await codexClient.listCodexAgentSources(), 'Codex agent sources')
  if (!isCodexAgentSourcesResponse(value)) throw new Error('Codex agent sources response is invalid')
  return value
}

export const addCodexAgentSource = async (url: string): Promise<CodexAgentSourceRecord> => {
  const value = objectResponse(
    await codexClient.addCodexAgentSource(url),
    'Codex agent source add',
  )
  if (!isCodexAgentSourceRecord(value)) throw new Error('Codex agent source add response is invalid')
  return value
}

export const removeCodexAgentSource = async (sourceId: string): Promise<void> => {
  return codexClient.removeCodexAgentSource(sourceId)
}

export const syncCodexAgentSource = async (sourceId: string): Promise<OpenJsonValueDto> => {
  return codexClient.syncCodexAgentSource(sourceId)
}

export const getCodexAgentSourceCatalog = async (
  sourceId: string,
): Promise<CodexAgentSourceCatalogResponse> => {
  const value = objectResponse(
    await codexClient.getCodexAgentSourceCatalog(sourceId),
    'Codex agent source catalog',
  )
  if (!isCodexAgentSourceCatalogResponse(value)) {
    throw new Error('Codex agent source catalog response is invalid')
  }
  return value
}

export const installCodexSourceAgent = async (payload: {
  sourceId: string
  agentId: string
  targetName?: string | null
  conflictMode?: string | null
}): Promise<OpenJsonValueDto> => {
  return codexClient.installCodexSourceAgent({
    sourceId: payload.sourceId,
    agentId: payload.agentId,
    targetName: payload.targetName ?? null,
    conflictMode: payload.conflictMode ?? null,
  })
}

export const syncCodexSourceInstall = async (installId: string): Promise<OpenJsonValueDto> => {
  return codexClient.syncCodexSourceInstall({ installId })
}

export const forceSyncCodexSourceInstall = async (
  installId: string,
): Promise<OpenJsonValueDto> => {
  return codexClient.syncCodexSourceInstall({ installId, force: true })
}

export const acceptLocalCodexSourceInstall = async (
  installId: string,
): Promise<OpenJsonValueDto> => {
  return codexClient.acceptLocalCodexSourceInstall(installId)
}

export const untrackCodexSourceInstall = async (
  installId: string,
): Promise<OpenJsonValueDto> => {
  return codexClient.untrackCodexSourceInstall(installId)
}

// ── Codex Models ──

export const listCodexModels = async (): Promise<CodexModelsResponse> => {
  const value = objectResponse(await codexClient.listCodexModels(), 'Codex models')
  if (!isCodexModelsResponse(value)) throw new Error('Codex models response is invalid')
  return value
}

// ── Codex Profile 管理（CCR profiles.toml） ──

export const addCodexProfile = async (
  profileOrName: string | object,
  config?: unknown,
): Promise<OpenJsonValueDto> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(profileOrName, config)
  return codexClient.addCodexProfile(name, toOpenJson(resolvedConfig))
}

export const updateCodexProfile = async (
  profileOrName: string | object,
  config?: unknown,
): Promise<OpenJsonValueDto> => {
  const { name, config: resolvedConfig } = resolveNameAndConfig(profileOrName, config)
  return codexClient.updateCodexProfile(name, toOpenJson(resolvedConfig))
}

export const deleteCodexProfile = async (
  nameOrRequest: string | object,
): Promise<void> => {
  const name = resolveName(nameOrRequest)
  await codexClient.deleteCodexProfile(name)
}

/** 获取 Codex Profile 详情（从列表过滤） */
export const getCodexProfile = async (name: string): Promise<CodexProfile | null> => {
  const profiles = await listCodexProfiles()
  const arr: unknown[] = Array.isArray(profiles) ? profiles : pickArray(profiles, 'profiles')
  const found = arr.find((item) => {
    if (!isRecord(item)) {
      return false
    }
    return String(item.name ?? '') === name
  })
  if (found === undefined || !isRecord(found)) return null
  if (!isCodexProfile(found)) throw new Error('Codex profile response is invalid')
  return found
}

export const getCodexProfileEnv = async (name: string): Promise<OpenJsonValueDto> => {
  return codexClient.getCodexProfileEnv(name)
}

export const applyCodexProfile = async (name: string): Promise<void> => {
  await codexClient.applyCodexProfile(name)
}

export const codexProfileOff = async (): Promise<CodexProfileOffResult> => {
  const value = objectResponse(await codexClient.codexProfileOff(), 'Codex profile off')
  const source = recordOf(value)
  if (source.status === 'unsupported_environment') {
    throw new Error('Codex profile off is only available in the local environment')
  }
  return {
    ok: source.ok === true,
    changed: source.changed === true,
    previous_profile: typeof source.previous_profile === 'string' ? source.previous_profile : null,
    runtime_mode: typeof source.runtime_mode === 'string' ? source.runtime_mode : 'official_auth',
  }
}

// ── Codex Sessions ──

export const listCodexSessions = async (options?: {
  limit?: number
  query?: string
}): Promise<CodexSessionsResponse> => {
  const value = objectResponse(
    await codexClient.listCodexSessions(options?.limit, options?.query),
    'Codex sessions',
  )
  if (!isCodexSessionsResponse(value)) throw new Error('Codex sessions response is invalid')
  return value
}

export const getCodexSessionDetail = async (
  filePath: string,
  messageLimit?: number,
): Promise<CodexSessionDetailResponse> => {
  const value = objectResponse(
    await codexClient.getCodexSessionDetail(filePath, messageLimit),
    'Codex session detail',
  )
  if (!isCodexSessionDetailResponse(value)) throw new Error('Codex session detail response is invalid')
  return value
}

export const exportCodexSession = async (
  filePath: string,
  maxMessages?: number,
): Promise<CodexSessionExportResponse> => {
  const value = objectResponse(
    await codexClient.exportCodexSession(filePath, maxMessages),
    'Codex session export',
  )
  if (!isCodexSessionExportResponse(value)) throw new Error('Codex session export response is invalid')
  return value
}

export const cloneCodexSession = async (filePath: string): Promise<CodexCloneSessionResponse> => {
  const value = objectResponse(
    await codexClient.cloneCodexSession(filePath),
    'Codex session clone',
  )
  if (!isCodexCloneSessionResponse(value)) throw new Error('Codex session clone response is invalid')
  return value
}

export const deleteCodexSession = async (filePath: string): Promise<OpenJsonValueDto> => {
  return codexClient.deleteCodexSession(filePath)
}

// ── Codex Auth 管理 ──

export {
  codexAuthOff,
  deleteCodexAuth,
  getCodexAuthCurrent,
  listCodexAuthAccounts,
  renameCodexAuth,
  saveCodexAuth,
  switchCodexAuth,
  type CodexAuthSaveRequest,
} from '../generated/codexAuth'

// ── Codex Tray / Process ──

export const getCodexTraySnapshot = async (force?: boolean): Promise<CodexTraySnapshot> => {
  const value = objectResponse(
    await codexClient.getCodexTraySnapshot(force),
    'Codex tray snapshot',
  )
  if (!isCodexTraySnapshot(value)) throw new Error('Codex tray snapshot response is invalid')
  return value
}

export { detectCodexProcess } from '../generated/codexAuth'

// ── Codex OAuth ──

export {
  codexAddAuthWithApiKey,
  codexDeleteModelProvider,
  codexImportAuthFromLocal,
  codexImportAuthPayload,
  codexIsOAuthPortInUse,
  codexListModelProviders,
  codexOAuthLoginCancel,
  codexOAuthLoginCompleted,
  codexOAuthLoginStart,
  codexOAuthSubmitCallbackUrl,
  codexOpenExternalUrl,
  codexReleaseOAuthPort,
  codexSaveModelProvider,
} from '../generated/codexAuth'

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
export const getCodexDashboardOverview = async (
  options?: CodexCommandOptions,
): Promise<CodexDashboardOverview> => {
  const value = objectResponse(
    await codexClient.getCodexDashboardOverview(options?.force),
    'Codex dashboard overview',
  )
  if (!isCodexDashboardOverview(value)) throw new Error('Codex dashboard overview response is invalid')
  return pickCodexDashboardOverview(value)
}

/** 获取 Codex 仪表盘用量摘要 */
export const getCodexDashboardUsageSummary = async (
  options?: CodexCommandOptions,
): Promise<CodexDashboardUsageSummary> => {
  const value = objectResponse(
    await codexClient.getCodexDashboardUsageSummary(options?.force),
    'Codex dashboard usage summary',
  )
  if (!isCodexDashboardUsageSummary(value)) {
    throw new Error('Codex dashboard usage summary response is invalid')
  }
  const source = recordOf(value)
  const section = (raw: unknown): CodexDashboardUsageSection => {
    const row = isRecord(raw) ? recordOf(raw) : {}
    return {
      total_requests: Number(row.total_requests) || 0,
      total_input_tokens: Number(row.total_input_tokens) || 0,
      total_output_tokens: Number(row.total_output_tokens) || 0,
    }
  }
  const top = isRecord(source.top_model) ? recordOf(source.top_model) : null
  return {
    last_activity_at: optionalString(source.last_activity_at),
    freshness: source.freshness as CodexDashboardUsageSummary['freshness'],
    freshness_description: String(source.freshness_description ?? ''),
    five_hour: section(source.five_hour),
    seven_day: section(source.seven_day),
    all_time: section(source.all_time),
    top_model: top
      ? {
          model: String(top.model ?? ''),
          total_requests: Number(top.total_requests) || 0,
          total_input_tokens: Number(top.total_input_tokens) || 0,
          total_output_tokens: Number(top.total_output_tokens) || 0,
          window_end: optionalString(top.window_end),
        }
      : null,
  }
}

export interface CodexUsageCommandOptions {
  force?: boolean
}

/** 获取 Codex 使用量 */
export const getCodexUsage = async (
  options?: CodexUsageCommandOptions,
): Promise<CodexUsageResponse> => {
  const value = objectResponse(
    await codexClient.getCodexUsage(options?.force),
    'Codex usage',
  )
  if (!isCodexUsageResponse(value)) throw new Error('Codex usage response is invalid')
  return value
}

/** 获取所有 Codex 账号的配额余额 */
export const getCodexAllQuotas = async (): Promise<CodexAccountQuota[]> => {
  return arrayResponse(await codexClient.getCodexAllQuotas(), 'Codex quotas').map((item) => {
    const value = objectResponse(item, 'Codex quota')
    if (!isCodexAccountQuota(value)) throw new Error('Codex quota response is invalid')
    return value
  })
}

/** 获取指定 Codex 账号的配额余额 */
export const getCodexQuota = async (account: string): Promise<CodexAccountQuota> => {
  const value = objectResponse(
    await codexClient.getCodexQuota(account),
    'Codex quota',
  )
  if (!isCodexAccountQuota(value)) throw new Error('Codex quota response is invalid')
  return value
}

// ── 平台限制 —— Codex 不支持 Slash Commands 与 Plugins，保留桩函数 ──

export const listCodexSlashCommands = async (): Promise<{
  commands: never[]
  folders: never[]
}> => {
  return { commands: [], folders: [] }
}

export const addCodexSlashCommand = async (
  _name: string,
  _config: unknown,
): Promise<{ success: false; message: string }> => {
  return { success: false, message: 'Codex 平台不支持斜杠命令' }
}

export const updateCodexSlashCommand = async (
  _name: string,
  _config: unknown,
): Promise<{ success: false; message: string }> => {
  return { success: false, message: 'Codex 平台不支持斜杠命令' }
}

export const deleteCodexSlashCommand = async (
  _name: string,
): Promise<{ success: false; message: string }> => {
  return { success: false, message: 'Codex 平台不支持斜杠命令' }
}

export const toggleCodexSlashCommand = async (
  _name: string,
  _enabled: boolean,
): Promise<{ success: false; message: string }> => {
  return { success: false, message: 'Codex 平台不支持斜杠命令' }
}

export const listCodexPlugins = async (): Promise<{ plugins: never[] }> => {
  return { plugins: [] }
}

export const addCodexPlugin = async (
  _name: string,
  _config: unknown,
): Promise<{ success: false; message: string }> => {
  return { success: false, message: 'Codex 平台不支持插件' }
}

export const updateCodexPlugin = async (
  _pluginOrName: string | object,
  _config?: unknown,
): Promise<{ success: false; message: string }> => {
  return { success: false, message: 'Codex 平台不支持插件' }
}

export const deleteCodexPlugin = async (
  _name: string,
): Promise<{ success: false; message: string }> => {
  return { success: false, message: 'Codex 平台不支持插件' }
}

export const toggleCodexPlugin = async (
  _name: string,
  _enabled: boolean,
): Promise<{ success: false; message: string }> => {
  return { success: false, message: 'Codex 平台不支持插件' }
}
