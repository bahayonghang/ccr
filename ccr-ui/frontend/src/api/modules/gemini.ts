/**
 * Gemini CLI Platform API Module
 *
 * Gemini MCP、Config、Agents/SlashCommands/Plugins 管理
 */

import { api } from '../core'
import type {
    ApiResponse,
    GeminiMcpServer,
    GeminiMcpServerRequest,
    GeminiMcpServersResponse,
    GeminiConfig,
    GeminiConfigResponse,
    Plugin,
    PlatformAgentsResponse,
    PlatformAgentRequest,
    PlatformSlashCommandsResponse,
    PlatformSlashCommandRequest,
    PlatformPluginRequest,
} from '@/types'

// ═══════════════════════════════════════════════════════════
// 🔌 Gemini MCP Server Management
// ═══════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════
// ⚙️ Gemini Base Config
// ═══════════════════════════════════════════════════════════

export const getGeminiConfig = async (): Promise<GeminiConfig> => {
    const response = await api.get<ApiResponse<GeminiConfigResponse>>('/gemini/config')
    return response.data.data!.config
}

export const updateGeminiConfig = async (config: GeminiConfig): Promise<string> => {
    const response = await api.put<ApiResponse<string>>('/gemini/config', config)
    return response.data.data!
}

// ═══════════════════════════════════════════════════════════
// 🤖 Gemini Agents (stub)
// ═══════════════════════════════════════════════════════════

export const listGeminiAgents = async (): Promise<PlatformAgentsResponse> => {
    throw new Error('Gemini agents are not supported by backend')
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

// ═══════════════════════════════════════════════════════════
// ⚡ Gemini Slash Commands
// ═══════════════════════════════════════════════════════════

export const listGeminiSlashCommands = async (): Promise<PlatformSlashCommandsResponse> => {
    const response = await api.get<ApiResponse<PlatformSlashCommandsResponse>>('/gemini/slash-commands')
    return response.data.data ?? (response.data as any)
}

export const addGeminiSlashCommand = async (_request: PlatformSlashCommandRequest): Promise<string> => {
    const response = await api.post<ApiResponse<string>>('/gemini/slash-commands', _request)
    return response.data.message || 'OK'
}

export const updateGeminiSlashCommand = async (_name: string, _request: PlatformSlashCommandRequest): Promise<string> => {
    const response = await api.put<ApiResponse<string>>(`/gemini/slash-commands/${encodeURIComponent(_name)}`, _request)
    return response.data.message || 'OK'
}

export const deleteGeminiSlashCommand = async (_name: string): Promise<string> => {
    const response = await api.delete<ApiResponse<string>>(`/gemini/slash-commands/${encodeURIComponent(_name)}`)
    return response.data.message || 'OK'
}

export const toggleGeminiSlashCommand = async (_name: string): Promise<string> => {
    const response = await api.put<ApiResponse<string>>(`/gemini/slash-commands/${encodeURIComponent(_name)}/toggle`)
    return response.data.message || 'OK'
}

// ═══════════════════════════════════════════════════════════
// 🔌 Gemini Plugins (stub)
// ═══════════════════════════════════════════════════════════

export const listGeminiPlugins = async (): Promise<Plugin[]> => {
    throw new Error('Gemini plugins are not supported by backend')
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
