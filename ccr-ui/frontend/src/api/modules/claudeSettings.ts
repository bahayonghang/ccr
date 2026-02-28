/**
 * Claude Code Settings API Module
 *
 * Claude Code 全局设置读写 API
 */

import { api } from '../core'
import type { ApiResponse } from '@/types'

export interface PermissionsData {
    allow: string[]
    deny: string[]
    defaultMode?: string
    additionalDirectories?: string[]
}

export interface SandboxNetworkData {
    allowLocalBinding?: boolean
    allowedDomains?: string[]
}

export interface SandboxData {
    enabled?: boolean
    autoAllowBashIfSandboxed?: boolean
    excludedCommands?: string[]
    network?: SandboxNetworkData
}

export interface AttributionData {
    commit?: string
    pr?: string
}

export interface ClaudeSettingsData {
    // 模型与推理
    model?: string
    availableModels?: string[]
    alwaysThinkingEnabled?: boolean
    maxThinkingTokens?: string
    maxOutputTokens?: string
    effortLevel?: string
    // 权限
    permissions?: PermissionsData
    skipDangerousModePermissionPrompt?: boolean
    // 环境变量
    env: Record<string, string>
    // UI 体验
    theme?: string
    language?: string
    showTurnDuration?: boolean
    prefersReducedMotion?: boolean
    spinnerTipsEnabled?: boolean
    terminalProgressBarEnabled?: boolean
    showSpinnerTree?: boolean
    // 沙箱安全
    sandbox?: SandboxData
    // Git 归属
    attribution?: AttributionData
    includeCoAuthoredBy?: boolean
    // 其他
    autoUpdates?: boolean
    autoUpdatesChannel?: string
    cleanupPeriodDays?: number
    respectGitignore?: boolean
}

export const getClaudeSettings = async (): Promise<ClaudeSettingsData> => {
    const response = await api.get<ApiResponse<ClaudeSettingsData>>('/claude-settings')
    return response.data.data!
}

export const updateClaudeSettings = async (data: ClaudeSettingsData): Promise<string> => {
    const response = await api.put<ApiResponse<string>>('/claude-settings', data)
    return response.data.data!
}
