/**
 * 统一 MCP 管理类型定义
 *
 * 对应后端 UnifiedMcp 系列 DTO，用于跨平台 MCP 服务器统一管理。
 */

// ============ 响应类型 ============

/** 统一 MCP 服务器（来自后端聚合） */
export interface UnifiedMcpServer {
    platform: string
    name: string
    command?: string | null
    url?: string | null
    args: string[]
    env: Record<string, string>
    headers?: Record<string, string> | null
    timeout?: number | null
    disabled: boolean
    cwd?: string | null
    trust?: boolean | null
    include_tools?: string[] | null
    scope?: McpScope | string | null
    source_path?: string | null
    approval_state?: McpApprovalState | string | null
    effective?: boolean
    hidden_by?: string | null
    raw_config?: Record<string, unknown> | null
}

/** 平台 MCP 能力矩阵 */
export interface PlatformMcpCapability {
    platform: string
    supports_toggle: boolean
    supports_url: boolean
    supports_headers: boolean
    supports_timeout: boolean
    supports_cwd: boolean
    supports_trust: boolean
    supports_include_tools: boolean
}

/** 统一 MCP 列表响应 */
export interface UnifiedMcpListResponse {
    servers: UnifiedMcpServer[]
    capabilities: PlatformMcpCapability[]
    diagnostics?: UnifiedMcpDiagnostic[]
    total?: number
}

export interface UnifiedMcpDiagnostic {
    level: 'info' | 'warning' | 'error' | string
    message: string
    source_path?: string | null
    scope?: string | null
    matched?: boolean | null
}

// ============ 请求类型 ============

/** 统一 MCP 服务器请求（新增/更新） */
export interface UnifiedMcpRequest {
    platform: string
    name: string
    scope?: McpScope | string | null
    command?: string | null
    url?: string | null
    args?: string[] | null
    env?: Record<string, string> | null
    headers?: Record<string, string> | null
    timeout?: number | null
    disabled?: boolean | null
    cwd?: string | null
    trust?: boolean | null
    include_tools?: string[] | null
}

// ============ 辅助类型 ============

/** 支持的平台列表 */
export type UnifiedMcpPlatform = 'claude' | 'codex' | 'gemini'
export type McpScope = 'local' | 'project' | 'user'
export type McpScopeFilter = 'effective' | McpScope | 'hidden'
export type McpApprovalState = 'approved' | 'pending' | 'disabled'

/** 平台元信息（用于 UI 展示） */
export interface PlatformMeta {
    id: UnifiedMcpPlatform
    label: string
    color: string
    icon: string
}
