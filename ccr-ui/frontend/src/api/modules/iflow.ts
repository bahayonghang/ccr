/**
 * iFlow Platform API Module
 *
 * iFlow MCP、Agents/SlashCommands/Plugins 管理
 */

import { api } from '../core'
import type {
    ApiResponse,
    Plugin,
    PlatformAgentsResponse,
    PlatformAgentRequest,
    PlatformSlashCommandsResponse,
    PlatformSlashCommandRequest,
    PlatformPluginRequest,
    PlatformMcpServerRequest,
} from '@/types'

// ═══════════════════════════════════════════════════════════
// 🔌 iFlow MCP Server Management
// ═══════════════════════════════════════════════════════════

export const listIflowMcpServers = async (): Promise<PlatformMcpServerRequest[]> => {
    const response = await api.get<ApiResponse<PlatformMcpServerRequest[]>>('/iflow/mcp')
    return response.data.data ?? (response.data as any)
}

export const addIflowMcpServer = async (_request: PlatformMcpServerRequest): Promise<string> => {
    const response = await api.post<ApiResponse<string>>('/iflow/mcp', _request)
    return response.data.message || 'OK'
}

export const updateIflowMcpServer = async (_name: string, _request: PlatformMcpServerRequest): Promise<string> => {
    const response = await api.put<ApiResponse<string>>(`/iflow/mcp/${encodeURIComponent(_name)}`, _request)
    return response.data.message || 'OK'
}

export const deleteIflowMcpServer = async (_name: string): Promise<string> => {
    const response = await api.delete<ApiResponse<string>>(`/iflow/mcp/${encodeURIComponent(_name)}`)
    return response.data.message || 'OK'
}

// ═══════════════════════════════════════════════════════════
// 🤖 iFlow Agents
// ═══════════════════════════════════════════════════════════

export const listIflowAgents = async (): Promise<PlatformAgentsResponse> => {
    const response = await api.get<ApiResponse<PlatformAgentsResponse>>('/iflow/agents')
    return response.data.data ?? (response.data as any)
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
    const response = await api.put<ApiResponse<string>>(`/iflow/agents/${encodeURIComponent(_name)}/toggle`)
    return response.data.message || 'OK'
}

// ═══════════════════════════════════════════════════════════
// ⚡ iFlow Slash Commands
// ═══════════════════════════════════════════════════════════

export const listIflowSlashCommands = async (): Promise<PlatformSlashCommandsResponse> => {
    const response = await api.get<ApiResponse<PlatformSlashCommandsResponse>>('/iflow/slash-commands')
    return response.data.data ?? (response.data as any)
}

export const addIflowSlashCommand = async (_request: PlatformSlashCommandRequest): Promise<string> => {
    const response = await api.post<ApiResponse<string>>('/iflow/slash-commands', _request)
    return response.data.message || 'OK'
}

export const updateIflowSlashCommand = async (_name: string, _request: PlatformSlashCommandRequest): Promise<string> => {
    const response = await api.put<ApiResponse<string>>(`/iflow/slash-commands/${encodeURIComponent(_name)}`, _request)
    return response.data.message || 'OK'
}

export const deleteIflowSlashCommand = async (_name: string): Promise<string> => {
    const response = await api.delete<ApiResponse<string>>(`/iflow/slash-commands/${encodeURIComponent(_name)}`)
    return response.data.message || 'OK'
}

export const toggleIflowSlashCommand = async (_name: string): Promise<string> => {
    const response = await api.put<ApiResponse<string>>(`/iflow/slash-commands/${encodeURIComponent(_name)}/toggle`)
    return response.data.message || 'OK'
}

// ═══════════════════════════════════════════════════════════
// 🔌 iFlow Plugins
// ═══════════════════════════════════════════════════════════

export const listIflowPlugins = async (): Promise<Plugin[]> => {
    const response = await api.get<ApiResponse<Plugin[]>>('/iflow/plugins')
    return response.data.data ?? (response.data as any)
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
