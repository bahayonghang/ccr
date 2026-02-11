/**
 * Qwen Platform API Module
 *
 * Qwen MCP、Config、Agents/SlashCommands/Plugins 管理
 */

import { api } from '../core'
import type {
    ApiResponse,
    QwenMcpServer,
    QwenMcpServerRequest,
    QwenMcpServersResponse,
    QwenConfig,
    QwenConfigResponse,
    Plugin,
    PlatformAgentsResponse,
    PlatformAgentRequest,
    PlatformSlashCommandsResponse,
    PlatformSlashCommandRequest,
    PlatformPluginRequest,
} from '@/types'

// ═══════════════════════════════════════════════════════════
// 🔌 Qwen MCP Server Management
// ═══════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════
// ⚙️ Qwen Base Config
// ═══════════════════════════════════════════════════════════

export const getQwenConfig = async (): Promise<QwenConfig> => {
    const response = await api.get<ApiResponse<QwenConfigResponse>>('/qwen/config')
    return response.data.data!.config
}

export const updateQwenConfig = async (config: QwenConfig): Promise<string> => {
    const response = await api.put<ApiResponse<string>>('/qwen/config', config)
    return response.data.data!
}

// ═══════════════════════════════════════════════════════════
// 🤖 Qwen Agents (stub)
// ═══════════════════════════════════════════════════════════

export const listQwenAgents = async (): Promise<PlatformAgentsResponse> => {
    throw new Error('Qwen agents are not supported by backend')
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

// ═══════════════════════════════════════════════════════════
// ⚡ Qwen Slash Commands
// ═══════════════════════════════════════════════════════════

export const listQwenSlashCommands = async (): Promise<PlatformSlashCommandsResponse> => {
    const response = await api.get<ApiResponse<PlatformSlashCommandsResponse>>('/qwen/slash-commands')
    return response.data.data ?? (response.data as any)
}

export const addQwenSlashCommand = async (_request: PlatformSlashCommandRequest): Promise<string> => {
    const response = await api.post<ApiResponse<string>>('/qwen/slash-commands', _request)
    return response.data.message || 'OK'
}

export const updateQwenSlashCommand = async (_name: string, _request: PlatformSlashCommandRequest): Promise<string> => {
    const response = await api.put<ApiResponse<string>>(`/qwen/slash-commands/${encodeURIComponent(_name)}`, _request)
    return response.data.message || 'OK'
}

export const deleteQwenSlashCommand = async (_name: string): Promise<string> => {
    const response = await api.delete<ApiResponse<string>>(`/qwen/slash-commands/${encodeURIComponent(_name)}`)
    return response.data.message || 'OK'
}

export const toggleQwenSlashCommand = async (_name: string): Promise<string> => {
    const response = await api.put<ApiResponse<string>>(`/qwen/slash-commands/${encodeURIComponent(_name)}/toggle`)
    return response.data.message || 'OK'
}

// ═══════════════════════════════════════════════════════════
// 🔌 Qwen Plugins (stub)
// ═══════════════════════════════════════════════════════════

export const listQwenPlugins = async (): Promise<Plugin[]> => {
    throw new Error('Qwen plugins are not supported by backend')
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
