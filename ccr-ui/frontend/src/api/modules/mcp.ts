/**
 * MCP Server Management API Module
 * 
 * 包含 MCP 服务器管理、预设管理、同步等 API
 */

import { api } from '../core'
import type {
    ApiResponse,
    McpServer,
    McpServerRequest,
    McpServersResponse,
} from '@/types'

// ═══════════════════════════════════════════════════════════
// 🔌 MCP 服务器管理 API (Claude Code Platform)
// ═══════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════
// 📦 MCP 预设管理 API (Presets)
// ═══════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════
// 🔄 MCP 同步 API (Sync)
// ═══════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════
// 📝 内置提示词 API (Builtin Prompts)
// ═══════════════════════════════════════════════════════════

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
