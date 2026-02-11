/**
 * Droid Platform API Module
 *
 * Droid MCP、Agents、SlashCommands、Plugins 管理
 */

import { api } from '../core'
import type {
    Agent,
    DroidMcpServer,
    DroidMcpServerRequest,
    PlatformAgentsResponse,
    PlatformAgentRequest,
    PlatformSlashCommandsResponse,
    PlatformSlashCommandRequest,
} from '@/types'

// ═══════════════════════════════════════════════════════════
// 📋 本地类型定义 (Local Types)
// ═══════════════════════════════════════════════════════════

export interface DroidPlugin {
    id: string
    data: Record<string, unknown>
}

// ═══════════════════════════════════════════════════════════
// 🔌 Droid MCP Server Management
// ═══════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════
// 🤖 Droid Agents
// ═══════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════
// ⚡ Droid Slash Commands
// ═══════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════
// 🔌 Droid Plugins
// ═══════════════════════════════════════════════════════════

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
