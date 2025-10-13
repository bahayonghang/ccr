// API Client for CCR UI Backend

import axios from 'axios';
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
} from '../types';

// Create axios instance with base configuration
const api = axios.create({
  baseURL: '/api',
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Request interceptor
api.interceptors.request.use(
  (config) => {
    console.log(`[API] ${config.method?.toUpperCase()} ${config.url}`);
    return config;
  },
  (error) => {
    console.error('[API] Request error:', error);
    return Promise.reject(error);
  }
);

// Response interceptor
api.interceptors.response.use(
  (response) => {
    console.log(`[API] Response:`, response.data);
    return response;
  },
  (error) => {
    console.error('[API] Response error:', error);
    return Promise.reject(error);
  }
);

// Command execution APIs
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

// Config management APIs
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
  const response = await api.post<ApiResponse<string>>('/validate');
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

export const getHistory = async (): Promise<HistoryResponse> => {
  const response = await api.get<ApiResponse<HistoryResponse>>('/history');
  return response.data.data!;
};

export const getSystemInfo = async (): Promise<SystemInfo> => {
  const response = await api.get<ApiResponse<SystemInfo>>('/system');
  return response.data.data!;
};

export default api;

