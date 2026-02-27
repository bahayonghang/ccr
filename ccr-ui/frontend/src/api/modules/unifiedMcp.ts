/**
 * 统一 MCP 管理 API 模块
 *
 * 对接后端 /api/unified/mcp 系列端点，提供跨平台 MCP 服务器 CRUD。
 */

import { api } from '../core'
import type {
    ApiResponse,
} from '@/types'
import type {
    UnifiedMcpListResponse,
    UnifiedMcpRequest,
} from '@/types/unifiedMcp'

// ═══════════════════════════════════════════════════════════
// 🔌 统一 MCP 服务器管理 API
// ═══════════════════════════════════════════════════════════

/**
 * 列出所有平台的 MCP 服务器
 * @param platform 可选，筛选平台（逗号分隔多个）
 */
export const listUnifiedMcp = async (platform?: string): Promise<UnifiedMcpListResponse> => {
    const params = platform ? { platform } : {}
    const response = await api.get<ApiResponse<UnifiedMcpListResponse>>('/unified/mcp', { params })
    return response.data.data!
}

/**
 * 添加 MCP 服务器到指定平台
 */
export const addUnifiedMcp = async (request: UnifiedMcpRequest): Promise<string> => {
    const response = await api.post<ApiResponse<{ message: string }>>('/unified/mcp', request)
    return response.data.data!.message
}

/**
 * 更新指定平台的 MCP 服务器
 */
export const updateUnifiedMcp = async (
    platform: string,
    name: string,
    request: UnifiedMcpRequest
): Promise<string> => {
    const response = await api.put<ApiResponse<{ message: string }>>(
        `/unified/mcp/${encodeURIComponent(platform)}/${encodeURIComponent(name)}`,
        request
    )
    return response.data.data!.message
}

/**
 * 删除指定平台的 MCP 服务器
 */
export const deleteUnifiedMcp = async (
    platform: string,
    name: string
): Promise<string> => {
    const response = await api.delete<ApiResponse<{ message: string }>>(
        `/unified/mcp/${encodeURIComponent(platform)}/${encodeURIComponent(name)}`
    )
    return response.data.data!.message
}

/**
 * 切换 MCP 服务器启用/禁用（仅 Claude 支持）
 */
export const toggleUnifiedMcp = async (
    platform: string,
    name: string
): Promise<{ message: string; disabled: boolean }> => {
    const response = await api.put<ApiResponse<{ message: string; disabled: boolean }>>(
        `/unified/mcp/${encodeURIComponent(platform)}/${encodeURIComponent(name)}/toggle`
    )
    return response.data.data!
}
