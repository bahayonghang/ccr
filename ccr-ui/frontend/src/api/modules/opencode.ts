/**
 * OpenCode 平台 API 模块
 *
 * Provider、MCP、Plugin、Config 管理接口
 */

import { api } from '../core'
import type {
    OpenCodeProvider,
    OpenCodeMcpServer,
    OpenCodePlugin,
    OpenCodeConfig,
    OpenCodeProviderRequest,
    OpenCodeMcpServerRequest,
    OpenCodePluginRequest,
} from '@/types/opencode'

// ═══════════════════════════════════════════════════════════
// 🔧 Provider 管理
// ═══════════════════════════════════════════════════════════

export const listOpenCodeProviders = async (): Promise<OpenCodeProvider[]> => {
    const response = await api.get<OpenCodeProvider[]>('/opencode/providers')
    return response.data
}

export const addOpenCodeProvider = async (request: OpenCodeProviderRequest): Promise<string> => {
    const response = await api.post<{ message: string }>('/opencode/providers', request)
    return response.data.message
}

export const updateOpenCodeProvider = async (
    id: string,
    request: OpenCodeProviderRequest
): Promise<string> => {
    const response = await api.put<{ message: string }>(
        `/opencode/providers/${encodeURIComponent(id)}`,
        request
    )
    return response.data.message
}

export const deleteOpenCodeProvider = async (id: string): Promise<string> => {
    const response = await api.delete<{ message: string }>(
        `/opencode/providers/${encodeURIComponent(id)}`
    )
    return response.data.message
}

// ═══════════════════════════════════════════════════════════
// 🔌 MCP 服务器管理（原生 OpenCode 格式）
// ═══════════════════════════════════════════════════════════

export const listOpenCodeMcpServers = async (): Promise<OpenCodeMcpServer[]> => {
    const response = await api.get<OpenCodeMcpServer[]>('/opencode/mcp')
    return response.data
}

export const addOpenCodeMcpServer = async (
    request: OpenCodeMcpServerRequest
): Promise<string> => {
    const response = await api.post<{ message: string }>('/opencode/mcp', request)
    return response.data.message
}

export const updateOpenCodeMcpServer = async (
    id: string,
    request: OpenCodeMcpServerRequest
): Promise<string> => {
    const response = await api.put<{ message: string }>(
        `/opencode/mcp/${encodeURIComponent(id)}`,
        request
    )
    return response.data.message
}

export const deleteOpenCodeMcpServer = async (id: string): Promise<string> => {
    const response = await api.delete<{ message: string }>(
        `/opencode/mcp/${encodeURIComponent(id)}`
    )
    return response.data.message
}

// ═══════════════════════════════════════════════════════════
// 📦 Plugin 管理
// ═══════════════════════════════════════════════════════════

export const listOpenCodePlugins = async (): Promise<OpenCodePlugin[]> => {
    const response = await api.get<OpenCodePlugin[]>('/opencode/plugins')
    return response.data
}

export const addOpenCodePlugin = async (request: OpenCodePluginRequest): Promise<string> => {
    const response = await api.post<{ message: string }>('/opencode/plugins', request)
    return response.data.message
}

export const deleteOpenCodePlugin = async (npm: string): Promise<string> => {
    const response = await api.delete<{ message: string }>(
        `/opencode/plugins/${encodeURIComponent(npm)}`
    )
    return response.data.message
}

// ═══════════════════════════════════════════════════════════
// ⚙️ 配置管理
// ═══════════════════════════════════════════════════════════

export const getOpenCodeConfig = async (): Promise<OpenCodeConfig> => {
    const response = await api.get<OpenCodeConfig>('/opencode/config')
    return response.data
}
