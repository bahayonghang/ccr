/**
 * Codex Platform API Module
 *
 * Codex MCP、配置、Profile、Auth、Usage、Agents/SlashCommands/Plugins 管理
 */

import { api } from '../core'
import type {
    ApiResponse,
    CodexMcpServer,
    CodexMcpServerRequest,
    CodexMcpServersResponse,
    CodexProfile,
    CodexProfileRequest,
    CodexProfilesResponse,
    CodexProfileResponse,
    CodexConfig,
    CodexConfigResponse,
    CodexAuthListResponse,
    CodexAuthCurrentResponse,
    CodexAuthSaveRequest,
    CodexAuthProcessResponse,
    CodexUsageResponse,
    LoginState,
    Plugin,
    PlatformAgentsResponse,
    PlatformAgentRequest,
    PlatformSlashCommandsResponse,
    PlatformSlashCommandRequest,
    PlatformPluginRequest,
} from '@/types'

// ═══════════════════════════════════════════════════════════
// 🔌 Codex MCP Server Management
// ═══════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════
// 📋 Codex Profile Management
// ═══════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════
// ⚙️ Codex Base Config
// ═══════════════════════════════════════════════════════════

export const getCodexConfig = async (): Promise<CodexConfig> => {
    const response = await api.get<ApiResponse<CodexConfigResponse>>('/codex/config')
    return response.data.data!.config
}

export const updateCodexConfig = async (config: CodexConfig): Promise<string> => {
    const response = await api.put<ApiResponse<string>>('/codex/config', config)
    return response.data.data!
}

// ═══════════════════════════════════════════════════════════
// 🔑 Codex Auth Management
// ═══════════════════════════════════════════════════════════

const normalizeLoginState = (value: unknown): LoginState => {
    if (!value || typeof value !== 'object' || Array.isArray(value)) {
        return { type: 'Unknown', raw_type: 'InvalidShape', raw: {} }
    }

    const raw = value as Record<string, unknown>
    const stateType = typeof raw.type === 'string' ? raw.type : '<missing>'

    switch (stateType) {
        case 'NotLoggedIn':
            return { type: 'NotLoggedIn' }
        case 'LoggedInUnsaved':
            return { type: 'LoggedInUnsaved' }
        case 'LoggedInSaved':
            if (typeof raw.account_name === 'string') {
                return { type: 'LoggedInSaved', account_name: raw.account_name }
            }
            return { type: 'Unknown', raw_type: stateType, raw }
        default:
            return { type: 'Unknown', raw_type: stateType, raw }
    }
}

export const listCodexAuthAccounts = async (): Promise<CodexAuthListResponse> => {
    const response = await api.get<ApiResponse<CodexAuthListResponse>>('/codex/auth/accounts')
    const data = response.data.data!
    return {
        ...data,
        login_state: normalizeLoginState(data.login_state),
    }
}

export const getCodexAuthCurrent = async (): Promise<CodexAuthCurrentResponse> => {
    const response = await api.get<ApiResponse<CodexAuthCurrentResponse>>('/codex/auth/current')
    const data = response.data.data!
    return {
        ...data,
        login_state: normalizeLoginState(data.login_state),
    }
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

// ═══════════════════════════════════════════════════════════
// 📊 Codex Usage
// ═══════════════════════════════════════════════════════════

export const getCodexUsage = async (): Promise<CodexUsageResponse> => {
    const response = await api.get<ApiResponse<CodexUsageResponse>>('/codex/usage')
    return response.data.data!
}

// ═══════════════════════════════════════════════════════════
// 🤖 Codex Agents (stub)
// ═══════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════
// ⚡ Codex Slash Commands (stub)
// ═══════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════
// 🔌 Codex Plugins (stub)
// ═══════════════════════════════════════════════════════════

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
