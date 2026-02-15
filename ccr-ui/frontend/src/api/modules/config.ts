/**
 * Config Management API Module
 * 
 * 包含配置管理、命令执行、历史记录、系统信息等 API
 */

import { api } from '../core'
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
    UpdateConfigRequest,
    CliVersionsResponse,
} from '@/types'

// ═══════════════════════════════════════════════════════════
// 🚀 命令执行 API (Command Execution)
// ═══════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════
// ⚙️ 配置管理 API (Config Management)
// ═══════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════
// 📜 历史记录和系统 API (History and System)
// ═══════════════════════════════════════════════════════════

export const getHistory = async (): Promise<HistoryResponse> => {
    const response = await api.get<HistoryResponse>('/history')
    return response.data
}

export const getSystemInfo = async (): Promise<SystemInfo> => {
    const response = await api.get<ApiResponse<SystemInfo>>('/system')
    return response.data.data!
}

// ═══════════════════════════════════════════════════════════
// 🔄 版本管理 API (Version Management)
// ═══════════════════════════════════════════════════════════

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

export const getCliVersions = async (): Promise<CliVersionsResponse> => {
    const response = await api.get<ApiResponse<CliVersionsResponse>>('/version/cli-versions')
    return response.data.data!
}
