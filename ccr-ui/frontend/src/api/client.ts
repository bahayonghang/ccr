// API Client for CCR UI Backend

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
} from '@/types'

// 创建 axios 实例
const createApiClient = (): AxiosInstance => {
  const api = axios.create({
    baseURL: '/api',
    timeout: 600000, // 10分钟超时，支持长时间编译更新
    headers: {
      'Content-Type': 'application/json',
    },
  })

  // 请求拦截器
  api.interceptors.request.use(
    (config) => {
      if (process.env.NODE_ENV === 'development') {
        console.log(`[API] ${config.method?.toUpperCase()} ${config.url}`)
      }
      return config
    },
    (error) => {
      console.error('[API] Request error:', error)
      return Promise.reject(error)
    }
  )

  // 响应拦截器
  api.interceptors.response.use(
    (response) => {
      if (process.env.NODE_ENV === 'development') {
        console.log(`[API] Response:`, response.data)
      }
      return response
    },
    (error) => {
      console.error('[API] Response error:', error)
      return Promise.reject(error)
    }
  )

  return api
}

const api = createApiClient()

// ===================================
// Statistics APIs
// ===================================

export const getCostOverview = async (range: string = 'today'): Promise<CostStats> => {
  const response = await api.get<CostStats>(`/stats/cost?range=${range}`)
  return response.data
}

export const getCostToday = async (): Promise<CostStats> => {
  const response = await api.get<CostStats>('/stats/cost/today')
  return response.data
}

export const getCostWeek = async (): Promise<CostStats> => {
  const response = await api.get<CostStats>('/stats/cost/week')
  return response.data
}

export const getCostMonth = async (): Promise<CostStats> => {
  const response = await api.get<CostStats>('/stats/cost/month')
  return response.data
}

export const getCostTrend = async (range: string = 'month'): Promise<DailyCost[]> => {
  const response = await api.get<DailyCost[]>(`/stats/cost/trend?range=${range}`)
  return response.data
}

export const getCostByModel = async (range: string = 'month'): Promise<Record<string, number>> => {
  const response = await api.get<Record<string, number>>(`/stats/cost/by-model?range=${range}`)
  return response.data
}

export const getCostByProject = async (range: string = 'month'): Promise<Record<string, number>> => {
  const response = await api.get<Record<string, number>>(`/stats/cost/by-project?range=${range}`)
  return response.data
}

export const getProviderUsage = async (): Promise<Record<string, number>> => {
  const response = await api.get<Record<string, number>>('/stats/provider-usage')
  return response.data
}

export const getTopSessions = async (limit: number = 10): Promise<TopSession[]> => {
  const response = await api.get<TopSession[]>(`/stats/cost/top-sessions?limit=${limit}`)
  return response.data
}

export const getStatsSummary = async (): Promise<StatsSummary> => {
  const response = await api.get<StatsSummary>('/stats/summary')
  return response.data
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

export const getConfig = async (configName: string): Promise<any> => {
  const response = await api.get<any>(`/configs/${configName}`)
  return response.data
}

export const updateConfig = async (configName: string, configData: any): Promise<string> => {
  const response = await api.put<string>(`/configs/${configName}`, configData)
  return response.data
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

export const listCodexProfiles = async (): Promise<CodexProfile[]> => {
  const response = await api.get<ApiResponse<CodexProfilesResponse>>('/codex/profiles')
  return response.data.data!.profiles
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

export const listGeminiAgents = async (): Promise<any> => {
  // TODO: Backend implementation needed
  return { agents: [], folders: [] }
}

export const addGeminiAgent = async (_request: any): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const updateGeminiAgent = async (_name: string, _request: any): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const deleteGeminiAgent = async (name: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const toggleGeminiAgent = async (name: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const listGeminiSlashCommands = async (): Promise<any> => {
  return { commands: [], folders: [] }
}

export const addGeminiSlashCommand = async (_request: any): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const updateGeminiSlashCommand = async (_name: string, _request: any): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const deleteGeminiSlashCommand = async (name: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const toggleGeminiSlashCommand = async (name: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const listGeminiPlugins = async (): Promise<any[]> => {
  return []
}

export const addGeminiPlugin = async (_request: any): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const updateGeminiPlugin = async (_id: string, _request: any): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const deleteGeminiPlugin = async (id: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const toggleGeminiPlugin = async (id: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

// ===================================
// Qwen Agents/SlashCommands/Plugins APIs (stub implementations)
// ===================================

export const listQwenAgents = async (): Promise<any> => {
  return { agents: [], folders: [] }
}

export const addQwenAgent = async (_request: any): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const updateQwenAgent = async (name: string, _request: any): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const deleteQwenAgent = async (name: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const toggleQwenAgent = async (name: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const listQwenSlashCommands = async (): Promise<any> => {
  return { commands: [], folders: [] }
}

export const addQwenSlashCommand = async (_request: any): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const updateQwenSlashCommand = async (name: string, _request: any): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const deleteQwenSlashCommand = async (name: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const toggleQwenSlashCommand = async (name: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const listQwenPlugins = async (): Promise<any[]> => {
  return []
}

export const addQwenPlugin = async (_request: any): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const updateQwenPlugin = async (id: string, _request: any): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const deleteQwenPlugin = async (id: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const toggleQwenPlugin = async (id: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

// ===================================
// iFlow Agents/SlashCommands/Plugins/MCP APIs (stub implementations)
// ===================================

export const listIflowMcpServers = async (): Promise<any[]> => {
  return []
}

export const addIflowMcpServer = async (_request: any): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const updateIflowMcpServer = async (name: string, _request: any): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const deleteIflowMcpServer = async (name: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const listIflowAgents = async (): Promise<any> => {
  return { agents: [], folders: [] }
}

export const addIflowAgent = async (_request: any): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const updateIflowAgent = async (name: string, _request: any): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const deleteIflowAgent = async (name: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const toggleIflowAgent = async (name: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const listIflowSlashCommands = async (): Promise<any> => {
  return { commands: [], folders: [] }
}

export const addIflowSlashCommand = async (_request: any): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const updateIflowSlashCommand = async (name: string, _request: any): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const deleteIflowSlashCommand = async (name: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const toggleIflowSlashCommand = async (name: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const listIflowPlugins = async (): Promise<any[]> => {
  return []
}

export const addIflowPlugin = async (_request: any): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const updateIflowPlugin = async (_id: string, _request: any): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const deleteIflowPlugin = async (id: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const toggleIflowPlugin = async (id: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

// ===================================
// Codex Agents/SlashCommands/Plugins APIs (stub implementations)
// ===================================

export const listCodexAgents = async (): Promise<any> => {
  return { agents: [], folders: [] }
}

export const addCodexAgent = async (_request: any): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const updateCodexAgent = async (_name: string, _request: any): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const deleteCodexAgent = async (name: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const toggleCodexAgent = async (name: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const listCodexSlashCommands = async (): Promise<any> => {
  return { commands: [], folders: [] }
}

export const addCodexSlashCommand = async (_request: any): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const updateCodexSlashCommand = async (_name: string, _request: any): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const deleteCodexSlashCommand = async (name: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const toggleCodexSlashCommand = async (name: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const listCodexPlugins = async (): Promise<any[]> => {
  return []
}

export const addCodexPlugin = async (_request: any): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const updateCodexPlugin = async (_id: string, _request: any): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const deleteCodexPlugin = async (id: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}

export const toggleCodexPlugin = async (id: string): Promise<string> => {
  throw new Error('Backend API not implemented yet')
}
