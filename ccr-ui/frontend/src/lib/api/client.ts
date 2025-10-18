// API Client for CCR UI Backend

import axios, { type AxiosInstance } from 'axios';
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
} from '../types';

// 创建 axios 实例
const createApiClient = (): AxiosInstance => {
  const api = axios.create({
    baseURL: '/api',
    timeout: 600000, // 10分钟超时，支持长时间编译更新
    headers: {
      'Content-Type': 'application/json',
    },
  });

  // 请求拦截器
  api.interceptors.request.use(
    (config) => {
      if (process.env.NODE_ENV === 'development') {
        console.log(`[API] ${config.method?.toUpperCase()} ${config.url}`);
      }
      return config;
    },
    (error) => {
      console.error('[API] Request error:', error);
      return Promise.reject(error);
    }
  );

  // 响应拦截器
  api.interceptors.response.use(
    (response) => {
      if (process.env.NODE_ENV === 'development') {
        console.log(`[API] Response:`, response.data);
      }
      return response;
    },
    (error) => {
      console.error('[API] Response error:', error);
      return Promise.reject(error);
    }
  );

  return api;
};

const api = createApiClient();

// ===================================
// Command execution APIs
// ===================================

export const executeCommand = async (
  request: CommandRequest
): Promise<CommandResponse> => {
  const response = await api.post<ApiResponse<CommandResponse>>(
    '/command/execute',
    request
  );
  return response.data.data!;
};

export const listCommands = async (): Promise<CommandInfo[]> => {
  const response = await api.get<ApiResponse<{ commands: CommandInfo[] }>>(
    '/command/list'
  );
  return response.data.data!.commands;
};

export const getCommandHelp = async (command: string): Promise<string> => {
  const response = await api.get<ApiResponse<string>>(`/command/help/${command}`);
  return response.data.data!;
};

// ===================================
// Config management APIs
// ===================================

export const listConfigs = async (): Promise<ConfigListResponse> => {
  const response = await api.get<ApiResponse<ConfigListResponse>>('/configs');
  return response.data.data!;
};

export const switchConfig = async (configName: string): Promise<string> => {
  const request: SwitchRequest = { config_name: configName };
  const response = await api.post<ApiResponse<string>>('/switch', request);
  return response.data.data!;
};

export const validateConfigs = async (): Promise<string> => {
  const response = await api.get<ApiResponse<string>>('/validate');
  return response.data.data!;
};

export const cleanBackups = async (
  request: CleanRequest
): Promise<CleanResponse> => {
  const response = await api.post<ApiResponse<CleanResponse>>('/clean', request);
  return response.data.data!;
};

export const exportConfig = async (
  request: ExportRequest
): Promise<ExportResponse> => {
  const response = await api.post<ApiResponse<ExportResponse>>('/export', request);
  return response.data.data!;
};

export const importConfig = async (
  request: ImportRequest
): Promise<ImportResponse> => {
  const response = await api.post<ApiResponse<ImportResponse>>('/import', request);
  return response.data.data!;
};

// ===================================
// History and System APIs
// ===================================

export const getHistory = async (): Promise<HistoryResponse> => {
  const response = await api.get<ApiResponse<HistoryResponse>>('/history');
  return response.data.data!;
};

export const getSystemInfo = async (): Promise<SystemInfo> => {
  const response = await api.get<ApiResponse<SystemInfo>>('/system');
  return response.data.data!;
};

// ===================================
// Version management APIs
// ===================================

export const getVersion = async (): Promise<VersionInfo> => {
  const response = await api.get<ApiResponse<VersionInfo>>('/version');
  return response.data.data!;
};

export const checkUpdate = async (): Promise<UpdateCheckResponse> => {
  const response = await api.get<ApiResponse<UpdateCheckResponse>>('/version/check-update');
  return response.data.data!;
};

export const updateCCR = async (): Promise<UpdateExecutionResponse> => {
  const response = await api.post<ApiResponse<UpdateExecutionResponse>>('/version/update');
  return response.data.data!;
};

// ===================================
// MCP Server Management APIs
// ===================================

export const listMcpServers = async (): Promise<McpServer[]> => {
  const response = await api.get<ApiResponse<McpServersResponse>>('/mcp');
  return response.data.data!.servers;
};

export const addMcpServer = async (request: McpServerRequest): Promise<string> => {
  const response = await api.post<ApiResponse<string>>('/mcp', request);
  return response.data.data!;
};

export const updateMcpServer = async (name: string, request: McpServerRequest): Promise<string> => {
  const response = await api.put<ApiResponse<string>>(`/mcp/${encodeURIComponent(name)}`, request);
  return response.data.data!;
};

export const deleteMcpServer = async (name: string): Promise<string> => {
  const response = await api.delete<ApiResponse<string>>(`/mcp/${encodeURIComponent(name)}`);
  return response.data.data!;
};

export const toggleMcpServer = async (name: string): Promise<string> => {
  const response = await api.put<ApiResponse<string>>(`/mcp/${encodeURIComponent(name)}/toggle`);
  return response.data.data!;
};

// ===================================
// Slash Command Management APIs
// ===================================

export const listSlashCommands = async (): Promise<SlashCommandsResponse> => {
  const response = await api.get<ApiResponse<SlashCommandsResponse>>('/slash-commands');
  return response.data.data!;
};

export const addSlashCommand = async (request: SlashCommandRequest): Promise<string> => {
  const response = await api.post<ApiResponse<string>>('/slash-commands', request);
  return response.data.data!;
};

export const updateSlashCommand = async (name: string, request: SlashCommandRequest): Promise<string> => {
  const response = await api.put<ApiResponse<string>>(`/slash-commands/${encodeURIComponent(name)}`, request);
  return response.data.data!;
};

export const deleteSlashCommand = async (name: string): Promise<string> => {
  const response = await api.delete<ApiResponse<string>>(`/slash-commands/${encodeURIComponent(name)}`);
  return response.data.data!;
};

export const toggleSlashCommand = async (name: string): Promise<string> => {
  const response = await api.put<ApiResponse<string>>(`/slash-commands/${encodeURIComponent(name)}/toggle`);
  return response.data.data!;
};

// ===================================
// Agent Management APIs
// ===================================

export const listAgents = async (): Promise<AgentsResponse> => {
  const response = await api.get<ApiResponse<AgentsResponse>>('/agents');
  return response.data.data!;
};

export const addAgent = async (request: AgentRequest): Promise<string> => {
  const response = await api.post<ApiResponse<string>>('/agents', request);
  return response.data.data!;
};

export const updateAgent = async (name: string, request: AgentRequest): Promise<string> => {
  const response = await api.put<ApiResponse<string>>(`/agents/${encodeURIComponent(name)}`, request);
  return response.data.data!;
};

export const deleteAgent = async (name: string): Promise<string> => {
  const response = await api.delete<ApiResponse<string>>(`/agents/${encodeURIComponent(name)}`);
  return response.data.data!;
};

export const toggleAgent = async (name: string): Promise<string> => {
  const response = await api.put<ApiResponse<string>>(`/agents/${encodeURIComponent(name)}/toggle`);
  return response.data.data!;
};

// ===================================
// Plugin Management APIs
// ===================================

export const listPlugins = async (): Promise<Plugin[]> => {
  const response = await api.get<ApiResponse<PluginsResponse>>('/plugins');
  return response.data.data!.plugins;
};

export const addPlugin = async (request: PluginRequest): Promise<string> => {
  const response = await api.post<ApiResponse<string>>('/plugins', request);
  return response.data.data!;
};

export const updatePlugin = async (id: string, request: PluginRequest): Promise<string> => {
  const response = await api.put<ApiResponse<string>>(`/plugins/${encodeURIComponent(id)}`, request);
  return response.data.data!;
};

export const deletePlugin = async (id: string): Promise<string> => {
  const response = await api.delete<ApiResponse<string>>(`/plugins/${encodeURIComponent(id)}`);
  return response.data.data!;
};

export const togglePlugin = async (id: string): Promise<string> => {
  const response = await api.put<ApiResponse<string>>(`/plugins/${encodeURIComponent(id)}/toggle`);
  return response.data.data!;
};

// ===================================
// Codex MCP Server Management APIs
// ===================================

export const listCodexMcpServers = async (): Promise<CodexMcpServer[]> => {
  const response = await api.get<ApiResponse<CodexMcpServersResponse>>('/codex/mcp');
  return response.data.data!.servers;
};

export const addCodexMcpServer = async (request: CodexMcpServerRequest): Promise<string> => {
  const response = await api.post<ApiResponse<string>>('/codex/mcp', request);
  return response.data.data!;
};

export const updateCodexMcpServer = async (
  name: string,
  request: CodexMcpServerRequest
): Promise<string> => {
  const response = await api.put<ApiResponse<string>>(
    `/codex/mcp/${encodeURIComponent(name)}`,
    request
  );
  return response.data.data!;
};

export const deleteCodexMcpServer = async (name: string): Promise<string> => {
  const response = await api.delete<ApiResponse<string>>(
    `/codex/mcp/${encodeURIComponent(name)}`
  );
  return response.data.data!;
};

// ===================================
// Codex Profile Management APIs
// ===================================

export const listCodexProfiles = async (): Promise<CodexProfile[]> => {
  const response = await api.get<ApiResponse<CodexProfilesResponse>>('/codex/profiles');
  return response.data.data!.profiles;
};

export const addCodexProfile = async (request: CodexProfileRequest): Promise<string> => {
  const response = await api.post<ApiResponse<string>>('/codex/profiles', request);
  return response.data.data!;
};

export const updateCodexProfile = async (
  name: string,
  request: CodexProfileRequest
): Promise<string> => {
  const response = await api.put<ApiResponse<string>>(
    `/codex/profiles/${encodeURIComponent(name)}`,
    request
  );
  return response.data.data!;
};

export const deleteCodexProfile = async (name: string): Promise<string> => {
  const response = await api.delete<ApiResponse<string>>(
    `/codex/profiles/${encodeURIComponent(name)}`
  );
  return response.data.data!;
};

// ===================================
// Codex Base Config Management APIs
// ===================================

export const getCodexConfig = async (): Promise<CodexConfig> => {
  const response = await api.get<ApiResponse<CodexConfigResponse>>('/codex/config');
  return response.data.data!.config;
};

export const updateCodexConfig = async (config: CodexConfig): Promise<string> => {
  const response = await api.put<ApiResponse<string>>('/codex/config', config);
  return response.data.data!;
};

// ===================================
// Config Converter APIs
// ===================================

export const convertConfig = async (request: ConverterRequest): Promise<ConverterResponse> => {
  const response = await api.post<ApiResponse<ConverterResponse>>('/converter/convert', request);
  return response.data.data!;
};

// ===================================
// Gemini MCP Server Management APIs
// ===================================

export const listGeminiMcpServers = async (): Promise<GeminiMcpServer[]> => {
  const response = await api.get<ApiResponse<GeminiMcpServersResponse>>('/gemini/mcp');
  return response.data.data!.servers;
};

export const addGeminiMcpServer = async (request: GeminiMcpServerRequest): Promise<string> => {
  const response = await api.post<ApiResponse<string>>('/gemini/mcp', request);
  return response.data.data!;
};

export const updateGeminiMcpServer = async (
  name: string,
  request: GeminiMcpServerRequest
): Promise<string> => {
  const response = await api.put<ApiResponse<string>>(
    `/gemini/mcp/${encodeURIComponent(name)}`,
    request
  );
  return response.data.data!;
};

export const deleteGeminiMcpServer = async (name: string): Promise<string> => {
  const response = await api.delete<ApiResponse<string>>(
    `/gemini/mcp/${encodeURIComponent(name)}`
  );
  return response.data.data!;
};

// ===================================
// Gemini Base Config Management APIs
// ===================================

export const getGeminiConfig = async (): Promise<GeminiConfig> => {
  const response = await api.get<ApiResponse<GeminiConfigResponse>>('/gemini/config');
  return response.data.data!.config;
};

export const updateGeminiConfig = async (config: GeminiConfig): Promise<string> => {
  const response = await api.put<ApiResponse<string>>('/gemini/config', config);
  return response.data.data!;
};

// ===================================
// Qwen MCP Server Management APIs
// ===================================

export const listQwenMcpServers = async (): Promise<QwenMcpServer[]> => {
  const response = await api.get<ApiResponse<QwenMcpServersResponse>>('/qwen/mcp');
  return response.data.data!.servers;
};

export const addQwenMcpServer = async (request: QwenMcpServerRequest): Promise<string> => {
  const response = await api.post<ApiResponse<string>>('/qwen/mcp', request);
  return response.data.data!;
};

export const updateQwenMcpServer = async (
  name: string,
  request: QwenMcpServerRequest
): Promise<string> => {
  const response = await api.put<ApiResponse<string>>(
    `/qwen/mcp/${encodeURIComponent(name)}`,
    request
  );
  return response.data.data!;
};

export const deleteQwenMcpServer = async (name: string): Promise<string> => {
  const response = await api.delete<ApiResponse<string>>(
    `/qwen/mcp/${encodeURIComponent(name)}`
  );
  return response.data.data!;
};

// ===================================
// Qwen Base Config Management APIs
// ===================================

export const getQwenConfig = async (): Promise<QwenConfig> => {
  const response = await api.get<ApiResponse<QwenConfigResponse>>('/qwen/config');
  return response.data.data!.config;
};

export const updateQwenConfig = async (config: QwenConfig): Promise<string> => {
  const response = await api.put<ApiResponse<string>>('/qwen/config', config);
  return response.data.data!;
};

export default api;


// ===================================
// Sync (WebDAV) APIs
// ===================================
import type { SyncStatusResponse, SyncOperationRequest, SyncOperationResponse, SyncInfoResponse } from '../types';

export const getSyncStatus = async (): Promise<SyncStatusResponse> => {
  const response = await api.get<ApiResponse<SyncStatusResponse>>('/sync/status');
  return response.data.data!;
};

export const getSyncInfo = async (): Promise<SyncInfoResponse> => {
  const response = await api.get<ApiResponse<SyncInfoResponse>>('/sync/info');
  return response.data.data!;
};

export const pushSync = async (req: SyncOperationRequest): Promise<SyncOperationResponse> => {
  const response = await api.post<ApiResponse<SyncOperationResponse>>('/sync/push', req);
  return response.data.data!;
};

export const pullSync = async (req: SyncOperationRequest): Promise<SyncOperationResponse> => {
  const response = await api.post<ApiResponse<SyncOperationResponse>>('/sync/pull', req);
  return response.data.data!;
};

