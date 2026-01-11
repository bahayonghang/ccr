/**
 * @fileoverview CCR UI 后端 API 客户端
 *
 * 提供与 CCR 后端服务通信的所有 API 函数。
 */

import axios, { type AxiosInstance } from 'axios'
import type {
  ApiResponse,
  CommandRequest,
  CommandResponse,
  CommandInfo,
  ConfigListResponse,
  SwitchRequest,
  HistoryResponse,
  SystemInfo,
  CleanRequest,
  CleanResponse,
  ExportRequest,
  ExportResponse,
  ImportRequest,
  ImportResponse,
  VersionInfo,
  UpdateCheckResponse,
  UpdateExecutionResponse,
  McpServer,
  McpServerRequest,
  McpServersResponse,
  SlashCommandRequest,
  SlashCommandsResponse,
  Agent,
  AgentRequest,
  AgentsResponse,
  Plugin,
  PluginRequest,
  PluginsResponse,
  CodexMcpServer,
  CodexMcpServerRequest,
  CodexMcpServersResponse,
  CodexProfile,
  CodexProfileRequest,
  CodexProfilesResponse,
  CodexProfileResponse,
  CodexConfig,
  CodexConfigResponse,
  GeminiMcpServer,
  GeminiMcpServerRequest,
  GeminiMcpServersResponse,
  GeminiConfig,
  GeminiConfigResponse,
  QwenMcpServer,
  QwenMcpServerRequest,
  QwenMcpServersResponse,
  QwenConfig,
  QwenConfigResponse,
  ConverterRequest,
  ConverterResponse,
  SyncStatusResponse,
  SyncOperationRequest,
  SyncOperationResponse,
  SyncInfoResponse,
  CostStats,
  DailyCost,
  TopSession,
  StatsSummary,
  UsageRecordsResponse,
  UpdateConfigRequest,
  BudgetStatus,
  SetBudgetRequest,
  PricingListResponse,
  SetPricingRequest,
  DailyStatsResponse,
  Hook,
  HookRequest,
  HooksResponse,
  ConfigDetail,
  ConfigUpdateData,
  PlatformAgentsResponse,
  PlatformSlashCommandsResponse,
  PlatformAgentRequest,
  PlatformSlashCommandRequest,
  PlatformPluginRequest,
  PlatformMcpServerRequest,
  DroidMcpServer,
  DroidMcpServerRequest,
  CodexAuthListResponse,
  CodexAuthCurrentResponse,
  CodexAuthSaveRequest,
  CodexAuthProcessResponse,
} from '@/types'

/**
 * 创建 API 客户端实例
 *
 * @returns 配置好的 Axios 实例
 */
const isTauriEnvironment = (): boolean => {
  return typeof window !== 'undefined' && '__TAURI__' in window
}

const resolveApiBaseUrl = (): string => {
  if (isTauriEnvironment()) {
    const port = import.meta.env.VITE_TAURI_BACKEND_PORT || '38081'
    return `http://127.0.0.1:${port}/api`
  }

  return '/api'
}

export const getBackendHealth = async (): Promise<void> => {
  const baseUrl = resolveApiBaseUrl()
  const rootUrl = baseUrl.endsWith('/api') ? baseUrl.slice(0, -4) : baseUrl
  // Tauri 环境下后端 sidecar 启动可能需要更长时间
  const timeout = isTauriEnvironment() ? 20000 : 4000
  await axios.get(`${rootUrl}/health`, { timeout })
}

const createApiClient = (): AxiosInstance => {
  const api = axios.create({
    baseURL: resolveApiBaseUrl(),
    timeout: 600000, // 10分钟超时，支持长时间编译更新
    headers: {
      'Content-Type': 'application/json',
    },
  })

  // 请求拦截器
  api.interceptors.request.use(
    (config) => {
      if (import.meta.env.DEV) {
        console.log(`[API] ${config.method?.toUpperCase()} ${config.url}`)
      }
      return config
    },
    (error) => {
      console.error('[API] Request error:', error)
      return Promise.reject(error)
    }
  )

  // 响应拦截器：统一处理 ApiResponse 格式的响应
  api.interceptors.response.use(
    (response) => {
      if (import.meta.env.DEV) {
        console.log(`[API] Response:`, response.data)
      }

      // 如果响应是 ApiResponse 格式（包含 success 字段）
      if (response.data && typeof response.data.success === 'boolean') {
        // success=false 时统一抛出错误
        if (!response.data.success) {
          const errorMessage = response.data.message || 'Unknown API error'
          console.error('[API] API returned error:', errorMessage)
          return Promise.reject(new Error(errorMessage))
        }
      }

      return response
    },
    (error) => {
      // 统一处理网络错误和 HTTP 错误
      let errorMessage = 'Network error or server unreachable'

      if (error.response) {
        // 服务器返回了错误状态码
        const status = error.response.status
        const data = error.response.data

        if (typeof data === 'string' && data.trim()) {
          errorMessage = data
        } else if (data && typeof data.message === 'string') {
          errorMessage = data.message
        } else if (data && typeof data.error === 'string') {
          errorMessage = data.error
        } else {
          errorMessage = `HTTP ${status}: ${error.message}`
        }
      } else if (error.request) {
        // 请求已发送但没有收到响应
        errorMessage = 'No response from server'
      } else {
        // 请求配置出错
        errorMessage = error.message
      }

      console.error('[API] Response error:', errorMessage)
      return Promise.reject(new Error(errorMessage))
    }
  )

  return api
}

/** 导出 Axios 实例供其他组件直接使用 */
export const api = createApiClient()

// ─────────────────────────────────────
// Statistics APIs
// ─────────────────────────────────────

/**
 * 获取成本概览
 *
 * @param range - 时间范围，默认 'today'
 * @returns 成本统计数据
 */
export const getCostOverview = async (range: string = 'today'): Promise<CostStats> => {
  const response = await api.get<CostStats>(`/stats/cost?range=${range}`)
  return response.data
}

/**
 * 获取今日成本统计
 *
 * @returns 今日成本数据
 */
export const getCostToday = async (): Promise<CostStats> => {
  const response = await api.get<CostStats>('/stats/cost/today')
  return response.data
}

/**
 * 获取本周成本统计
 *
 * @returns 本周成本数据
 */
export const getCostWeek = async (): Promise<CostStats> => {
  const response = await api.get<CostStats>('/stats/cost/week')
  return response.data
}

/**
 * 获取本月成本统计
 *
 * @returns 本月成本数据
 */
export const getCostMonth = async (): Promise<CostStats> => {
  const response = await api.get<CostStats>('/stats/cost/month')
  return response.data
}

/**
 * 获取成本趋势数据
 *
 * @param range - 时间范围，默认 'month'
 * @returns 每日成本数组
 */
export const getCostTrend = async (range: string = 'month'): Promise<DailyCost[]> => {
  const response = await api.get<DailyCost[]>(`/stats/cost/trend?range=${range}`)
  return response.data
}

/**
 * 按模型获取成本分布
 *
 * @param range - 时间范围，默认 'month'
 * @returns 模型名称到成本的映射
 */
export const getCostByModel = async (range: string = 'month'): Promise<Record<string, number>> => {
  const response = await api.get<Record<string, number>>(`/stats/cost/by-model?range=${range}`)
  return response.data
}

/**
 * 按项目获取成本分布
 *
 * @param range - 时间范围，默认 'month'
 * @returns 项目名称到成本的映射
 */
export const getCostByProject = async (range: string = 'month'): Promise<Record<string, number>> => {
  const response = await api.get<Record<string, number>>(`/stats/cost/by-project?range=${range}`)
  return response.data
}

/**
 * 获取提供商使用情况
 *
 * @returns 提供商名称到使用次数的映射
 */
export const getProviderUsage = async (): Promise<Record<string, number>> => {
  const response = await api.get<Record<string, number>>('/stats/provider-usage')
  return response.data
}

/**
 * 获取成本最高的会话
 *
 * @param limit - 返回数量限制，默认 10
 * @returns 会话列表
 */
export const getTopSessions = async (limit: number = 10): Promise<TopSession[]> => {
  const response = await api.get<TopSession[]>(`/stats/cost/top-sessions?limit=${limit}`)
  return response.data
}

export const getStatsSummary = async (): Promise<StatsSummary> => {
  const response = await api.get<StatsSummary>('/stats/summary')
  return response.data
}

// ===================================
// Budget Management APIs
// ===================================

export const getBudgetStatus = async (): Promise<BudgetStatus> => {
  const response = await api.get<BudgetStatus>('/budget/status')
  return response.data
}

export const setBudget = async (request: SetBudgetRequest): Promise<void> => {
  await api.post('/budget/set', request)
}

export const resetBudget = async (): Promise<void> => {
  await api.post('/budget/reset')
}

// ===================================
// Pricing Management APIs
// ===================================

export const getPricingList = async (): Promise<PricingListResponse> => {
  const response = await api.get<PricingListResponse>('/pricing/list')
  return response.data
}

export const setPricing = async (request: SetPricingRequest): Promise<void> => {
  await api.post('/pricing/set', request)
}

export const removePricing = async (model: string): Promise<void> => {
  await api.delete(`/pricing/remove/${encodeURIComponent(model)}`)
}

export const resetPricing = async (): Promise<void> => {
  await api.post('/pricing/reset')
}

// ===================================
// Usage Analytics APIs
// ===================================

export const getUsageRecords = async (
  platform: string = 'claude',
  limit: number = 10000
): Promise<UsageRecordsResponse> => {
  const params = new URLSearchParams()
  params.set('platform', platform)
  params.set('limit', limit.toString())
  const response = await api.get<UsageRecordsResponse>(`/usage/records?${params}`)
  return response.data
}

/**
 * 获取每日使用统计 - 支持 CodMate 风格的三视图切换
 * @param days 查询天数，默认 30
 */
export const getDailyStats = async (days: number = 30): Promise<DailyStatsResponse> => {
  const response = await api.get<DailyStatsResponse>(`/sessions/stats/daily?days=${days}`)
  return response.data
}

// ===================================
// Command execution APIs
// ===================================

export const executeCommand = async (
  request: CommandRequest
): Promise<CommandResponse> => {
  const response = await api.post<ApiResponse<CommandResponse>>(
    '/command/execute',
    request
  )
  return response.data.data!
}

export const listCommands = async (): Promise<CommandInfo[]> => {
  const response = await api.get<ApiResponse<{ commands: CommandInfo[] }>>(
    '/command/list'
  )
  return response.data.data!.commands
}

export const getCommandHelp = async (command: string): Promise<string> => {
  const response = await api.get<ApiResponse<string>>(`/command/help/${command}`)
  return response.data.data!
}

// ===================================
// Config management APIs
// ===================================

export const listConfigs = async (): Promise<ConfigListResponse> => {
  const response = await api.get<ConfigListResponse>('/configs')
  return response.data
}

export const switchConfig = async (configName: string): Promise<string> => {
  const request: SwitchRequest = { config_name: configName }
  const response = await api.post<string>('/switch', request)
  return response.data
}

export const validateConfigs = async (): Promise<string> => {
  const response = await api.get<string>('/validate')
  return response.data
}

export const deleteConfig = async (configName: string): Promise<string> => {
  const response = await api.delete<string>(`/configs/${configName}`)
  return response.data
}

// 📊 启用/禁用配置
export const enableConfig = async (configName: string): Promise<string> => {
  const response = await api.patch<ApiResponse<string>>(`/configs/${configName}/enable`)
  return response.data.data || 'Configuration enabled successfully'
}

export const disableConfig = async (configName: string): Promise<string> => {
  const response = await api.patch<ApiResponse<string>>(`/configs/${configName}/disable`)
  return response.data.data || 'Configuration disabled successfully'
}

export const getConfig = async (configName: string): Promise<ConfigDetail> => {
  const response = await api.get<ConfigDetail>(`/configs/${configName}`)
  return response.data
}

export const updateConfig = async (configName: string, configData: ConfigUpdateData): Promise<string> => {
  const response = await api.put<string>(`/configs/${configName}`, configData)
  return response.data
}

// 📝 添加新配置
export const addConfig = async (configData: UpdateConfigRequest): Promise<string> => {
  const response = await api.post<ApiResponse<string>>('/configs', configData)
  return response.data.data || 'Configuration added successfully'
}

export const cleanBackups = async (
  request: CleanRequest
): Promise<CleanResponse> => {
  const response = await api.post<ApiResponse<CleanResponse>>('/clean', request)
  return response.data.data!
}

export const exportConfig = async (
  request: ExportRequest
): Promise<ExportResponse> => {
  const response = await api.post<ApiResponse<ExportResponse>>('/export', request)
  return response.data.data!
}

export const importConfig = async (
  request: ImportRequest
): Promise<ImportResponse> => {
  const response = await api.post<ApiResponse<ImportResponse>>('/import', request)
  return response.data.data!
}

// ===================================
// History and System APIs
// ===================================

export const getHistory = async (): Promise<HistoryResponse> => {
  const response = await api.get<HistoryResponse>('/history')
  return response.data
}

export const getSystemInfo = async (): Promise<SystemInfo> => {
  const response = await api.get<ApiResponse<SystemInfo>>('/system')
  return response.data.data!
}

// ===================================
// Version management APIs
// ===================================

export const getVersion = async (): Promise<VersionInfo> => {
  const response = await api.get<ApiResponse<VersionInfo>>('/version')
  return response.data.data!
}

export const checkUpdate = async (): Promise<UpdateCheckResponse> => {
  const response = await api.get<ApiResponse<UpdateCheckResponse>>('/version/check-update')
  return response.data.data!
}

export const updateCCR = async (): Promise<UpdateExecutionResponse> => {
  const response = await api.post<ApiResponse<UpdateExecutionResponse>>('/version/update')
  return response.data.data!
}

// ===================================
// MCP Server Management APIs
// ===================================

export const listMcpServers = async (): Promise<McpServer[]> => {
  const response = await api.get<ApiResponse<McpServersResponse>>('/mcp')
  return response.data.data!.servers
}

export const addMcpServer = async (request: McpServerRequest): Promise<string> => {
  const response = await api.post<ApiResponse<string>>('/mcp', request)
  return response.data.data!
}

export const updateMcpServer = async (name: string, request: McpServerRequest): Promise<string> => {
  const response = await api.put<ApiResponse<string>>(`/mcp/${encodeURIComponent(name)}`, request)
  return response.data.data!
}

export const deleteMcpServer = async (name: string): Promise<string> => {
  const response = await api.delete<ApiResponse<string>>(`/mcp/${encodeURIComponent(name)}`)
  return response.data.data!
}

export const toggleMcpServer = async (name: string): Promise<string> => {
  const response = await api.put<ApiResponse<string>>(`/mcp/${encodeURIComponent(name)}/toggle`)
  return response.data.data!
}

// ===================================
// MCP Preset Management APIs
// ===================================

export interface McpPreset {
  id: string
  name: string
  description: string
  command: string | null
  args: string[]
  tags: string[]
  homepage: string | null
  docs: string | null
  requires_api_key: boolean
  api_key_env: string | null
}

export interface InstallPresetRequest {
  preset_id: string
  platforms: string[]
  env?: Record<string, string>
}

export interface InstallPresetResult {
  platform: string
  success: boolean
  message: string | null
}

export interface InstallPresetResponse {
  message: string
  results: InstallPresetResult[]
}

export const listMcpPresets = async (): Promise<McpPreset[]> => {
  const response = await api.get<ApiResponse<McpPreset[]>>('/mcp/presets')
  return response.data.data!
}

export const getMcpPreset = async (id: string): Promise<McpPreset> => {
  const response = await api.get<ApiResponse<McpPreset>>(`/mcp/presets/${encodeURIComponent(id)}`)
  return response.data.data!
}

export const installMcpPreset = async (
  presetId: string,
  platforms: string[],
  env?: Record<string, string>
): Promise<InstallPresetResponse> => {
  const request: InstallPresetRequest = {
    preset_id: presetId,
    platforms,
    env: env || {}
  }
  const response = await api.post<ApiResponse<InstallPresetResponse>>(
    `/mcp/presets/${encodeURIComponent(presetId)}/install`,
    request
  )
  return response.data.data!
}

export const installMcpPresetSingle = async (
  presetId: string,
  platform: string,
  env?: Record<string, string>
): Promise<{ message: string; platform: string }> => {
  const request: InstallPresetRequest = {
    preset_id: presetId,
    platforms: [platform],
    env: env || {}
  }
  const response = await api.post<ApiResponse<{ message: string; platform: string }>>(
    '/mcp/presets/install',
    request
  )
  return response.data.data!
}

// ===================================
// MCP Sync APIs
// ===================================

export interface McpServerInfo {
  name: string
  command: string | null
  args: string[]
  env: Record<string, string>
}

export interface SyncResult {
  platform: string
  success: boolean
  message: string | null
}

export interface SyncResponse {
  message: string
  results: SyncResult[]
}

export interface SyncAllResponse {
  message: string
  servers: Record<string, SyncResult[]>
}

export const listSourceMcpServers = async (): Promise<McpServerInfo[]> => {
  const response = await api.get<ApiResponse<McpServerInfo[]>>('/mcp/sync/source')
  return response.data.data!
}

export const syncMcpServer = async (
  serverName: string,
  targetPlatforms: string[]
): Promise<SyncResponse> => {
  const response = await api.post<ApiResponse<SyncResponse>>(
    `/mcp/sync/${encodeURIComponent(serverName)}`,
    { platforms: targetPlatforms }
  )
  return response.data.data!
}

export const syncAllMcpServers = async (
  targetPlatforms: string[]
): Promise<SyncAllResponse> => {
  const response = await api.post<ApiResponse<SyncAllResponse>>(
    '/mcp/sync/all',
    { platforms: targetPlatforms }
  )
  return response.data.data!
}

// ===================================
// Builtin Prompts Management APIs
// ===================================

export interface BuiltinPrompt {
  id: string
  name: string
  description: string
  content: string
  category: string
  tags: string[]
}

export const listBuiltinPrompts = async (): Promise<BuiltinPrompt[]> => {
  const response = await api.get<ApiResponse<BuiltinPrompt[]>>('/prompts/builtin')
  return response.data.data!
}

export const getBuiltinPrompt = async (id: string): Promise<BuiltinPrompt> => {
  const response = await api.get<ApiResponse<BuiltinPrompt>>(`/prompts/builtin/${id}`)
  return response.data.data!
}

export const getBuiltinPromptsByCategory = async (category: string): Promise<BuiltinPrompt[]> => {
  const response = await api.get<ApiResponse<BuiltinPrompt[]>>(`/prompts/builtin/category/${category}`)
  return response.data.data!
}

// ===================================
// Slash Command Management APIs
// ===================================

export const listSlashCommands = async (): Promise<SlashCommandsResponse> => {
  const response = await api.get<ApiResponse<SlashCommandsResponse>>('/slash-commands')
  return response.data.data!
}

export const addSlashCommand = async (request: SlashCommandRequest): Promise<string> => {
  const response = await api.post<ApiResponse<string>>('/slash-commands', request)
  return response.data.data!
}

export const updateSlashCommand = async (name: string, request: SlashCommandRequest): Promise<string> => {
  const response = await api.put<ApiResponse<string>>(`/slash-commands/${encodeURIComponent(name)}`, request)
  return response.data.data!
}

export const deleteSlashCommand = async (name: string): Promise<string> => {
  const response = await api.delete<ApiResponse<string>>(`/slash-commands/${encodeURIComponent(name)}`)
  return response.data.data!
}

export const toggleSlashCommand = async (name: string): Promise<string> => {
  const response = await api.put<ApiResponse<string>>(`/slash-commands/${encodeURIComponent(name)}/toggle`)
  return response.data.data!
}

// ===================================
// Agent Management APIs
// ===================================

export const listAgents = async (): Promise<AgentsResponse> => {
  const response = await api.get<ApiResponse<AgentsResponse>>('/agents')
  return response.data.data!
}

export const getAgent = async (name: string): Promise<Agent> => {
  const response = await api.get<ApiResponse<Agent>>(`/agents/${encodeURIComponent(name)}`)
  return response.data.data!
}

export const addAgent = async (request: AgentRequest): Promise<string> => {
  const response = await api.post<ApiResponse<string>>('/agents', request)
  return response.data.data!
}

export const updateAgent = async (name: string, request: AgentRequest): Promise<string> => {
  const response = await api.put<ApiResponse<string>>(`/agents/${encodeURIComponent(name)}`, request)
  return response.data.data!
}

export const deleteAgent = async (name: string): Promise<string> => {
  const response = await api.delete<ApiResponse<string>>(`/agents/${encodeURIComponent(name)}`)
  return response.data.data!
}

export const toggleAgent = async (name: string): Promise<string> => {
  const response = await api.put<ApiResponse<string>>(`/agents/${encodeURIComponent(name)}/toggle`)
  return response.data.data!
}

// ===================================
// Plugin Management APIs
// ===================================

export const listPlugins = async (): Promise<Plugin[]> => {
  const response = await api.get<ApiResponse<PluginsResponse>>('/plugins')
  return response.data.data!.plugins
}

export const addPlugin = async (request: PluginRequest): Promise<string> => {
  const response = await api.post<ApiResponse<string>>('/plugins', request)
  return response.data.data!
}

export const updatePlugin = async (id: string, request: PluginRequest): Promise<string> => {
  const response = await api.put<ApiResponse<string>>(`/plugins/${encodeURIComponent(id)}`, request)
  return response.data.data!
}

export const deletePlugin = async (id: string): Promise<string> => {
  const response = await api.delete<ApiResponse<string>>(`/plugins/${encodeURIComponent(id)}`)
  return response.data.data!
}

export const togglePlugin = async (id: string): Promise<string> => {
  const response = await api.put<ApiResponse<string>>(`/plugins/${encodeURIComponent(id)}/toggle`)
  return response.data.data!
}

// ===================================
// Codex MCP Server Management APIs
// ===================================

export const listCodexMcpServers = async (): Promise<CodexMcpServer[]> => {
  const response = await api.get<ApiResponse<CodexMcpServersResponse>>('/codex/mcp')
  return response.data.data!.servers
}

export const addCodexMcpServer = async (request: CodexMcpServerRequest): Promise<string> => {
  const response = await api.post<ApiResponse<string>>('/codex/mcp', request)
  return response.data.data!
}

export const updateCodexMcpServer = async (
  name: string,
  request: CodexMcpServerRequest
): Promise<string> => {
  const response = await api.put<ApiResponse<string>>(
    `/codex/mcp/${encodeURIComponent(name)}`,
    request
  )
  return response.data.data!
}

export const deleteCodexMcpServer = async (name: string): Promise<string> => {
  const response = await api.delete<ApiResponse<string>>(
    `/codex/mcp/${encodeURIComponent(name)}`
  )
  return response.data.data!
}

// ===================================
// Codex Profile Management APIs
// ===================================

export const listCodexProfiles = async (): Promise<CodexProfilesResponse> => {
  const response = await api.get<ApiResponse<CodexProfilesResponse>>('/codex/profiles')
  return response.data.data!
}

export const addCodexProfile = async (request: CodexProfileRequest): Promise<string> => {
  const response = await api.post<ApiResponse<string>>('/codex/profiles', request)
  return response.data.data!
}

export const updateCodexProfile = async (
  name: string,
  request: CodexProfileRequest
): Promise<string> => {
  const response = await api.put<ApiResponse<string>>(
    `/codex/profiles/${encodeURIComponent(name)}`,
    request
  )
  return response.data.data!
}

export const deleteCodexProfile = async (name: string): Promise<string> => {
  const response = await api.delete<ApiResponse<string>>(
    `/codex/profiles/${encodeURIComponent(name)}`
  )
  return response.data.data!
}

export const getCodexProfile = async (name: string): Promise<CodexProfile> => {
  const response = await api.get<ApiResponse<CodexProfileResponse>>(
    `/codex/profiles/${encodeURIComponent(name)}`
  )
  return response.data.data!.profile
}

export const applyCodexProfile = async (name: string): Promise<string> => {
  const response = await api.post<ApiResponse<string>>(
    `/codex/profiles/${encodeURIComponent(name)}/apply`
  )
  return response.data.data!
}

// ===================================
// Codex Base Config Management APIs
// ===================================

export const getCodexConfig = async (): Promise<CodexConfig> => {
  const response = await api.get<ApiResponse<CodexConfigResponse>>('/codex/config')
  return response.data.data!.config
}

export const updateCodexConfig = async (config: CodexConfig): Promise<string> => {
  const response = await api.put<ApiResponse<string>>('/codex/config', config)
  return response.data.data!
}

// ===================================
// Codex Auth Management APIs
// ===================================

export const listCodexAuthAccounts = async (): Promise<CodexAuthListResponse> => {
  const response = await api.get<ApiResponse<CodexAuthListResponse>>('/codex/auth/accounts')
  return response.data.data!
}

export const getCodexAuthCurrent = async (): Promise<CodexAuthCurrentResponse> => {
  const response = await api.get<ApiResponse<CodexAuthCurrentResponse>>('/codex/auth/current')
  return response.data.data!
}

export const saveCodexAuth = async (request: CodexAuthSaveRequest): Promise<string> => {
  const response = await api.post<ApiResponse<string>>('/codex/auth/save', request)
  return response.data.data!
}

export const switchCodexAuth = async (name: string): Promise<string> => {
  const response = await api.post<ApiResponse<string>>(
    `/codex/auth/switch/${encodeURIComponent(name)}`
  )
  return response.data.data!
}

export const deleteCodexAuth = async (name: string): Promise<string> => {
  const response = await api.delete<ApiResponse<string>>(
    `/codex/auth/${encodeURIComponent(name)}`
  )
  return response.data.data!
}

export const detectCodexProcess = async (): Promise<CodexAuthProcessResponse> => {
  const response = await api.get<ApiResponse<CodexAuthProcessResponse>>('/codex/auth/process')
  return response.data.data!
}

// ===================================
// Config Converter APIs
// ===================================

export const convertConfig = async (request: ConverterRequest): Promise<ConverterResponse> => {
  const response = await api.post<ApiResponse<ConverterResponse>>('/converter/convert', request)
  return response.data.data!
}

// ===================================
// Gemini MCP Server Management APIs
// ===================================

export const listGeminiMcpServers = async (): Promise<GeminiMcpServer[]> => {
  const response = await api.get<ApiResponse<GeminiMcpServersResponse>>('/gemini/mcp')
  return response.data.data!.servers
}

export const addGeminiMcpServer = async (request: GeminiMcpServerRequest): Promise<string> => {
  const response = await api.post<ApiResponse<string>>('/gemini/mcp', request)
  return response.data.data!
}

export const updateGeminiMcpServer = async (
  name: string,
  request: GeminiMcpServerRequest
): Promise<string> => {
  const response = await api.put<ApiResponse<string>>(
    `/gemini/mcp/${encodeURIComponent(name)}`,
    request
  )
  return response.data.data!
}

export const deleteGeminiMcpServer = async (name: string): Promise<string> => {
  const response = await api.delete<ApiResponse<string>>(
    `/gemini/mcp/${encodeURIComponent(name)}`
  )
  return response.data.data!
}

// ===================================
// Gemini Base Config Management APIs
// ===================================

export const getGeminiConfig = async (): Promise<GeminiConfig> => {
  const response = await api.get<ApiResponse<GeminiConfigResponse>>('/gemini/config')
  return response.data.data!.config
}

export const updateGeminiConfig = async (config: GeminiConfig): Promise<string> => {
  const response = await api.put<ApiResponse<string>>('/gemini/config', config)
  return response.data.data!
}

// ===================================
// Qwen MCP Server Management APIs
// ===================================

export const listQwenMcpServers = async (): Promise<QwenMcpServer[]> => {
  const response = await api.get<ApiResponse<QwenMcpServersResponse>>('/qwen/mcp')
  return response.data.data!.servers
}

export const addQwenMcpServer = async (request: QwenMcpServerRequest): Promise<string> => {
  const response = await api.post<ApiResponse<string>>('/qwen/mcp', request)
  return response.data.data!
}

export const updateQwenMcpServer = async (
  name: string,
  request: QwenMcpServerRequest
): Promise<string> => {
  const response = await api.put<ApiResponse<string>>(
    `/qwen/mcp/${encodeURIComponent(name)}`,
    request
  )
  return response.data.data!
}

export const deleteQwenMcpServer = async (name: string): Promise<string> => {
  const response = await api.delete<ApiResponse<string>>(
    `/qwen/mcp/${encodeURIComponent(name)}`
  )
  return response.data.data!
}

// ===================================
// Qwen Base Config Management APIs
// ===================================

export const getQwenConfig = async (): Promise<QwenConfig> => {
  const response = await api.get<ApiResponse<QwenConfigResponse>>('/qwen/config')
  return response.data.data!.config
}

export const updateQwenConfig = async (config: QwenConfig): Promise<string> => {
  const response = await api.put<ApiResponse<string>>('/qwen/config', config)
  return response.data.data!
}

// ===================================
// Sync (WebDAV) APIs
// ===================================

export const getSyncStatus = async (): Promise<SyncStatusResponse> => {
  const response = await api.get<ApiResponse<SyncStatusResponse>>('/sync/status')
  return response.data.data!
}

export const getSyncInfo = async (): Promise<SyncInfoResponse> => {
  const response = await api.get<ApiResponse<SyncInfoResponse>>('/sync/info')
  return response.data.data!
}

export const pushSync = async (req: SyncOperationRequest): Promise<SyncOperationResponse> => {
  const response = await api.post<ApiResponse<SyncOperationResponse>>('/sync/push', req)
  return response.data.data!
}

export const pullSync = async (req: SyncOperationRequest): Promise<SyncOperationResponse> => {
  const response = await api.post<ApiResponse<SyncOperationResponse>>('/sync/pull', req)
  return response.data.data!
}
// ===================================
// Gemini Agents/SlashCommands/Plugins APIs (stub implementations)
// ===================================

export const listGeminiAgents = async (): Promise<PlatformAgentsResponse> => {
  // TODO: Backend implementation needed
  return { agents: [], folders: [] }
}

export const addGeminiAgent = async (_request: PlatformAgentRequest): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const updateGeminiAgent = async (_name: string, _request: PlatformAgentRequest): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const deleteGeminiAgent = async (_name: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const toggleGeminiAgent = async (_name: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const listGeminiSlashCommands = async (): Promise<PlatformSlashCommandsResponse> => {
  return { commands: [], folders: [] }
}

export const addGeminiSlashCommand = async (_request: PlatformSlashCommandRequest): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const updateGeminiSlashCommand = async (_name: string, _request: PlatformSlashCommandRequest): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const deleteGeminiSlashCommand = async (_name: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const toggleGeminiSlashCommand = async (_name: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const listGeminiPlugins = async (): Promise<Plugin[]> => {
  return []
}

export const addGeminiPlugin = async (_request: PlatformPluginRequest): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const updateGeminiPlugin = async (_id: string, _request: PlatformPluginRequest): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const deleteGeminiPlugin = async (_id: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const toggleGeminiPlugin = async (_id: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

// ===================================
// Qwen Agents/SlashCommands/Plugins APIs (stub implementations)
// ===================================

export const listQwenAgents = async (): Promise<PlatformAgentsResponse> => {
  return { agents: [], folders: [] }
}

export const addQwenAgent = async (_request: PlatformAgentRequest): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const updateQwenAgent = async (_name: string, _request: PlatformAgentRequest): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const deleteQwenAgent = async (_name: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const toggleQwenAgent = async (_name: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const listQwenSlashCommands = async (): Promise<PlatformSlashCommandsResponse> => {
  return { commands: [], folders: [] }
}

export const addQwenSlashCommand = async (_request: PlatformSlashCommandRequest): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const updateQwenSlashCommand = async (_name: string, _request: PlatformSlashCommandRequest): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const deleteQwenSlashCommand = async (_name: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const toggleQwenSlashCommand = async (_name: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const listQwenPlugins = async (): Promise<Plugin[]> => {
  return []
}

export const addQwenPlugin = async (_request: PlatformPluginRequest): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const updateQwenPlugin = async (_id: string, _request: PlatformPluginRequest): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const deleteQwenPlugin = async (_id: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const toggleQwenPlugin = async (_id: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

// ===================================
// iFlow Agents/SlashCommands/Plugins/MCP APIs (stub implementations)
// ===================================

export const listIflowMcpServers = async (): Promise<PlatformMcpServerRequest[]> => {
  return []
}

export const addIflowMcpServer = async (_request: PlatformMcpServerRequest): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const updateIflowMcpServer = async (_name: string, _request: PlatformMcpServerRequest): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const deleteIflowMcpServer = async (_name: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const listIflowAgents = async (): Promise<PlatformAgentsResponse> => {
  return { agents: [], folders: [] }
}

export const addIflowAgent = async (_request: PlatformAgentRequest): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const updateIflowAgent = async (_name: string, _request: PlatformAgentRequest): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const deleteIflowAgent = async (_name: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const toggleIflowAgent = async (_name: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const listIflowSlashCommands = async (): Promise<PlatformSlashCommandsResponse> => {
  return { commands: [], folders: [] }
}

export const addIflowSlashCommand = async (_request: PlatformSlashCommandRequest): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const updateIflowSlashCommand = async (_name: string, _request: PlatformSlashCommandRequest): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const deleteIflowSlashCommand = async (_name: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const toggleIflowSlashCommand = async (_name: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const listIflowPlugins = async (): Promise<Plugin[]> => {
  return []
}

export const addIflowPlugin = async (_request: PlatformPluginRequest): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const updateIflowPlugin = async (_id: string, _request: PlatformPluginRequest): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const deleteIflowPlugin = async (_id: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const toggleIflowPlugin = async (_id: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

// ===================================
// Droid MCP APIs
// ===================================

export const listDroidMcpServers = async (): Promise<DroidMcpServer[]> => {
  const response = await api.get<DroidMcpServer[]>('/droid/mcp')
  return response.data
}

export const addDroidMcpServer = async (request: DroidMcpServerRequest): Promise<string> => {
  const response = await api.post<{ message: string }>('/droid/mcp', request)
  return response.data.message
}

export const updateDroidMcpServer = async (name: string, request: DroidMcpServerRequest): Promise<string> => {
  const response = await api.put<{ message: string }>(`/droid/mcp/${encodeURIComponent(name)}`, request)
  return response.data.message
}

export const deleteDroidMcpServer = async (name: string): Promise<string> => {
  const response = await api.delete<{ message: string }>(`/droid/mcp/${encodeURIComponent(name)}`)
  return response.data.message
}

// ===================================
// Droid Agents APIs
// ===================================

export const listDroidAgents = async (): Promise<PlatformAgentsResponse> => {
  const response = await api.get<PlatformAgentsResponse>('/droid/agents')
  return response.data
}

export const getDroidAgent = async (name: string): Promise<Agent> => {
  const response = await api.get<Agent>(`/droid/agents/${encodeURIComponent(name)}`)
  return response.data
}

export const addDroidAgent = async (request: PlatformAgentRequest): Promise<string> => {
  const response = await api.post<{ message: string }>('/droid/agents', request)
  return response.data.message
}

export const updateDroidAgent = async (name: string, request: PlatformAgentRequest): Promise<string> => {
  const response = await api.put<{ message: string }>(`/droid/agents/${encodeURIComponent(name)}`, request)
  return response.data.message
}

export const deleteDroidAgent = async (name: string): Promise<string> => {
  const response = await api.delete<{ message: string }>(`/droid/agents/${encodeURIComponent(name)}`)
  return response.data.message
}

// ===================================
// Droid Slash Commands APIs
// ===================================

export const listDroidSlashCommands = async (): Promise<PlatformSlashCommandsResponse> => {
  const response = await api.get<PlatformSlashCommandsResponse>('/droid/slash-commands')
  return response.data
}

export const addDroidSlashCommand = async (request: PlatformSlashCommandRequest): Promise<string> => {
  const response = await api.post<{ message: string }>('/droid/slash-commands', request)
  return response.data.message
}

export const updateDroidSlashCommand = async (name: string, request: PlatformSlashCommandRequest): Promise<string> => {
  const response = await api.put<{ message: string }>(`/droid/slash-commands/${encodeURIComponent(name)}`, request)
  return response.data.message
}

export const deleteDroidSlashCommand = async (name: string): Promise<string> => {
  const response = await api.delete<{ message: string }>(`/droid/slash-commands/${encodeURIComponent(name)}`)
  return response.data.message
}

// ===================================
// Droid Plugins APIs
// ===================================

export interface DroidPlugin {
  id: string
  data: Record<string, unknown>
}

export const listDroidPlugins = async (): Promise<DroidPlugin[]> => {
  const response = await api.get<DroidPlugin[]>('/droid/plugins')
  return response.data
}

export const addDroidPlugin = async (id: string, data: Record<string, unknown>): Promise<string> => {
  const response = await api.post<{ message: string }>('/droid/plugins', { id, data })
  return response.data.message
}

export const updateDroidPlugin = async (id: string, data: Record<string, unknown>): Promise<string> => {
  const response = await api.put<{ message: string }>(`/droid/plugins/${encodeURIComponent(id)}`, { data })
  return response.data.message
}

export const deleteDroidPlugin = async (id: string): Promise<string> => {
  const response = await api.delete<{ message: string }>(`/droid/plugins/${encodeURIComponent(id)}`)
  return response.data.message
}

// ===================================
// Codex Agents/SlashCommands/Plugins APIs (stub implementations)
// ===================================

export const listCodexAgents = async (): Promise<PlatformAgentsResponse> => {
  return { agents: [], folders: [] }
}

export const addCodexAgent = async (_request: PlatformAgentRequest): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const updateCodexAgent = async (_name: string, _request: PlatformAgentRequest): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const deleteCodexAgent = async (_name: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const toggleCodexAgent = async (_name: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const listCodexSlashCommands = async (): Promise<PlatformSlashCommandsResponse> => {
  return { commands: [], folders: [] }
}

export const addCodexSlashCommand = async (_request: PlatformSlashCommandRequest): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const updateCodexSlashCommand = async (_name: string, _request: PlatformSlashCommandRequest): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const deleteCodexSlashCommand = async (_name: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const toggleCodexSlashCommand = async (_name: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const listCodexPlugins = async (): Promise<Plugin[]> => {
  return []
}

export const addCodexPlugin = async (_request: PlatformPluginRequest): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const updateCodexPlugin = async (_id: string, _request: PlatformPluginRequest): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const deleteCodexPlugin = async (_id: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const toggleCodexPlugin = async (_id: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

// ===================================
// Hooks Management APIs
// ===================================

export const listHooks = async (): Promise<Hook[]> => {
  const response = await api.get<ApiResponse<HooksResponse>>('/hooks')
  return response.data.data!.hooks
}

export const addHook = async (request: HookRequest): Promise<string> => {
  const response = await api.post<ApiResponse<string>>('/hooks', request)
  return response.data.data!
}

export const updateHook = async (name: string, request: HookRequest): Promise<string> => {
  const response = await api.put<ApiResponse<string>>(`/hooks/${encodeURIComponent(name)}`, request)
  return response.data.data!
}

export const deleteHook = async (name: string): Promise<string> => {
  const response = await api.delete<ApiResponse<string>>(`/hooks/${encodeURIComponent(name)}`)
  return response.data.data!
}

export const toggleHook = async (name: string): Promise<string> => {
  const response = await api.put<ApiResponse<string>>(`/hooks/${encodeURIComponent(name)}/toggle`)
  return response.data.data!
}

// ===================================
// Skills Management APIs
// ===================================

export interface Skill {
  name: string
  description?: string
  path: string
  instruction: string
  metadata?: {
    author?: string
    version?: string
    license?: string
    tags?: string[]
    updated_at?: string
  }
  is_remote?: boolean
  repository?: string
}

export interface SkillRepository {
  name: string
  url: string
  branch: string
  description?: string
  skill_count?: number
  last_synced?: string
  is_official?: boolean
}

export interface AddRepositoryRequest {
  name: string
  url: string
  branch?: string
  description?: string
}

export const listSkills = async (): Promise<Skill[]> => {
  const response = await api.get<ApiResponse<Skill[]>>('/skills')
  return response.data.data!
}

export const addSkill = async (name: string, instruction: string): Promise<string> => {
  const response = await api.post<ApiResponse<string>>('/skills', { name, instruction })
  return response.data.data!
}

export const deleteSkill = async (name: string): Promise<string> => {
  const response = await api.delete<ApiResponse<string>>(`/skills/${encodeURIComponent(name)}`)
  return response.data.data!
}

export const listSkillRepositories = async (): Promise<SkillRepository[]> => {
  const response = await api.get<ApiResponse<SkillRepository[]>>('/skills/repositories')
  return response.data.data!
}

export const addSkillRepository = async (request: AddRepositoryRequest): Promise<string> => {
  const response = await api.post<ApiResponse<string>>('/skills/repositories', request)
  return response.data.data!
}

export const removeSkillRepository = async (name: string): Promise<string> => {
  const response = await api.delete<ApiResponse<string>>(`/skills/repositories/${encodeURIComponent(name)}`)
  return response.data.data!
}

export const scanSkillRepository = async (name: string): Promise<Skill[]> => {
  const response = await api.get<ApiResponse<Skill[]>>(`/skills/repositories/${encodeURIComponent(name)}/scan`)
  return response.data.data!
}

// ═══════════════════════════════════════════════════════════
// 📝 签到管理 API (Checkin Management)
// ═══════════════════════════════════════════════════════════

import type {
  CheckinProvider,
  CreateProviderRequest,
  UpdateProviderRequest,
  ProvidersResponse,
  AccountInfo,
  CreateAccountRequest,
  UpdateAccountRequest,
  AccountsResponse,
  CheckinRequest,
  CheckinResponse,
  CheckinExecutionResult,
  CheckinRecordsResponse,
  BalanceSnapshot,
  BalanceHistoryResponse,
  TodayCheckinStats,
  CheckinAccountDashboardResponse,
  ExportOptions as CheckinExportOptions,
  ExportData as CheckinExportData,
  ImportPreviewResponse as CheckinImportPreviewResponse,
  ImportRequest as CheckinImportRequest,
  ImportResult as CheckinImportResult,
  TestConnectionResponse,
} from '@/types/checkin'

// --- 提供商管理 ---

/** 获取所有签到提供商 */
export const listCheckinProviders = async (): Promise<ProvidersResponse> => {
  const response = await api.get<ProvidersResponse>('/checkin/providers')
  return response.data
}

/** 获取单个签到提供商 */
export const getCheckinProvider = async (id: string): Promise<CheckinProvider> => {
  const response = await api.get<CheckinProvider>(`/checkin/providers/${id}`)
  return response.data
}

/** 创建签到提供商 */
export const createCheckinProvider = async (request: CreateProviderRequest): Promise<CheckinProvider> => {
  const response = await api.post<CheckinProvider>('/checkin/providers', request)
  return response.data
}

/** 更新签到提供商 */
export const updateCheckinProvider = async (id: string, request: UpdateProviderRequest): Promise<CheckinProvider> => {
  const response = await api.put<CheckinProvider>(`/checkin/providers/${id}`, request)
  return response.data
}

/** 删除签到提供商 */
export const deleteCheckinProvider = async (id: string): Promise<void> => {
  await api.delete(`/checkin/providers/${id}`)
}

// --- 内置提供商 ---

import type {
  BuiltinProvidersResponse,
  AddBuiltinProviderRequest,
} from '@/types/checkin'

/** 获取所有内置提供商 */
export const listBuiltinProviders = async (): Promise<BuiltinProvidersResponse> => {
  const response = await api.get<BuiltinProvidersResponse>('/checkin/providers/builtin')
  return response.data
}

/** 添加内置提供商到用户配置 */
export const addBuiltinProvider = async (builtinId: string): Promise<CheckinProvider> => {
  const request: AddBuiltinProviderRequest = { builtin_id: builtinId }
  const response = await api.post<CheckinProvider>('/checkin/providers/builtin/add', request)
  return response.data
}

// --- 账号管理 ---

/** 获取所有签到账号 */
export const listCheckinAccounts = async (providerId?: string): Promise<AccountsResponse> => {
  const params = providerId ? { provider_id: providerId } : {}
  const response = await api.get<AccountsResponse>('/checkin/accounts', { params })
  return response.data
}

/** 获取单个签到账号 */
export const getCheckinAccount = async (id: string): Promise<AccountInfo> => {
  const response = await api.get<AccountInfo>(`/checkin/accounts/${id}`)
  return response.data
}

/** 获取账号 Dashboard 聚合数据 */
export const getCheckinAccountDashboard = async (
  id: string,
  params?: { year?: number; month?: number; days?: number }
): Promise<CheckinAccountDashboardResponse> => {
  const response = await api.get<CheckinAccountDashboardResponse>(`/checkin/accounts/${id}/dashboard`, { params })
  return response.data
}

/** 创建签到账号 */
export const createCheckinAccount = async (request: CreateAccountRequest): Promise<AccountInfo> => {
  const response = await api.post<AccountInfo>('/checkin/accounts', request)
  return response.data
}

/** 更新签到账号 */
export const updateCheckinAccount = async (id: string, request: UpdateAccountRequest): Promise<AccountInfo> => {
  const response = await api.put<AccountInfo>(`/checkin/accounts/${id}`, request)
  return response.data
}

/** 删除签到账号 */
export const deleteCheckinAccount = async (id: string): Promise<void> => {
  await api.delete(`/checkin/accounts/${id}`)
}

/** 账号 Cookies 响应（用于编辑） */
export interface AccountCookiesResponse {
  cookies_json: string
  api_user: string
}

/** 获取账号的解密后 Cookies（用于编辑） */
export const getCheckinAccountCookies = async (id: string): Promise<AccountCookiesResponse> => {
  const response = await api.get<AccountCookiesResponse>(`/checkin/accounts/${id}/cookies`)
  return response.data
}

// --- 签到操作 ---

/** 执行签到（批量或全部） */
export const executeCheckin = async (request?: CheckinRequest): Promise<CheckinResponse> => {
  const response = await api.post<CheckinResponse>('/checkin/execute', request || {})
  return response.data
}

/** 执行单个账号签到 */
export const checkinAccount = async (id: string): Promise<CheckinExecutionResult> => {
  const response = await api.post<CheckinExecutionResult>(`/checkin/accounts/${id}/checkin`)
  return response.data
}

// --- 余额查询 ---

/** 查询账号余额 */
export const queryCheckinBalance = async (id: string): Promise<BalanceSnapshot> => {
  const response = await api.post<BalanceSnapshot>(`/checkin/accounts/${id}/balance`)
  return response.data
}

/** 获取账号余额历史 */
export const getCheckinBalanceHistory = async (id: string, limit?: number): Promise<BalanceHistoryResponse> => {
  const params = limit ? { limit } : {}
  const response = await api.get<BalanceHistoryResponse>(`/checkin/accounts/${id}/balance/history`, { params })
  return response.data
}

// --- 签到记录 ---

/** 获取所有签到记录 */
export const listCheckinRecords = async (limit?: number): Promise<CheckinRecordsResponse> => {
  const params = limit ? { limit } : {}
  const response = await api.get<CheckinRecordsResponse>('/checkin/records', { params })
  return response.data
}

/** 获取账号签到记录 */
export const getAccountCheckinRecords = async (id: string, limit?: number): Promise<CheckinRecordsResponse> => {
  const params = limit ? { limit } : {}
  const response = await api.get<CheckinRecordsResponse>(`/checkin/accounts/${id}/records`, { params })
  return response.data
}

// --- 统计 ---

/** 获取今日签到统计 */
export const getTodayCheckinStats = async (): Promise<TodayCheckinStats> => {
  const response = await api.get<TodayCheckinStats>('/checkin/stats/today')
  return response.data
}

// --- 导入/导出 ---

/** 导出签到配置 */
export const exportCheckinConfig = async (options?: CheckinExportOptions): Promise<CheckinExportData> => {
  const response = await api.post<CheckinExportData>('/checkin/export', options || {})
  return response.data
}

/** 预览导入 */
export const previewCheckinImport = async (data: CheckinExportData): Promise<CheckinImportPreviewResponse> => {
  const response = await api.post<CheckinImportPreviewResponse>('/checkin/import/preview', data)
  return response.data
}

/** 执行导入 */
export const importCheckinConfig = async (request: CheckinImportRequest): Promise<CheckinImportResult> => {
  const response = await api.post<CheckinImportResult>('/checkin/import', request)
  return response.data
}

// --- 连接测试 ---

/** 测试账号连接 */
export const testCheckinConnection = async (id: string): Promise<TestConnectionResponse> => {
  const response = await api.post<TestConnectionResponse>(`/checkin/accounts/${id}/test`)
  return response.data
}

// ===================================
// Output Styles Management APIs
// ===================================

import type {
  OutputStyle,
  OutputStyleRequest,
  UpdateOutputStyleRequest,
} from '@/types'

export const listOutputStyles = async (): Promise<OutputStyle[]> => {
  const response = await api.get<ApiResponse<OutputStyle[]>>('/output-styles')
  return response.data.data!
}

export const getOutputStyle = async (name: string): Promise<OutputStyle> => {
  const response = await api.get<ApiResponse<OutputStyle>>(`/output-styles/${encodeURIComponent(name)}`)
  return response.data.data!
}

export const createOutputStyle = async (request: OutputStyleRequest): Promise<string> => {
  const response = await api.post<ApiResponse<string>>('/output-styles', request)
  return response.data.data!
}

export const updateOutputStyle = async (name: string, request: UpdateOutputStyleRequest): Promise<string> => {
  const response = await api.put<ApiResponse<string>>(`/output-styles/${encodeURIComponent(name)}`, request)
  return response.data.data!
}

export const deleteOutputStyle = async (name: string): Promise<string> => {
  const response = await api.delete<ApiResponse<string>>(`/output-styles/${encodeURIComponent(name)}`)
  return response.data.data!
}

// ===================================
// Statusline Configuration APIs
// ===================================

import type { StatuslineConfig } from '@/types'

export const getStatusline = async (): Promise<StatuslineConfig> => {
  const response = await api.get<ApiResponse<StatuslineConfig>>('/statusline')
  return response.data.data!
}

export const updateStatusline = async (config: StatuslineConfig): Promise<string> => {
  const response = await api.put<ApiResponse<string>>('/statusline', config)
  return response.data.data!
}
