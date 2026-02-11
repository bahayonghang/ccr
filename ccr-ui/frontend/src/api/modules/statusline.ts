/**
 * Statusline Configuration API Module
 *
 * Claude Code 状态栏配置管理 API
 */

import { api } from '../core'
import type { ApiResponse, StatuslineConfig } from '@/types'

export const getStatusline = async (): Promise<StatuslineConfig> => {
    const response = await api.get<ApiResponse<StatuslineConfig>>('/statusline')
    return response.data.data!
}

export const updateStatusline = async (config: StatuslineConfig): Promise<string> => {
    const response = await api.put<ApiResponse<string>>('/statusline', config)
    return response.data.data!
}
