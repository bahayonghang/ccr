/**
 * Plugin Management API Module
 *
 * Claude Code 插件管理 CRUD API
 */

import { api } from '../core'
import type {
    ApiResponse,
    Plugin,
    PluginRequest,
    PluginsResponse,
} from '@/types'

export const listPlugins = async (): Promise<Plugin[]> => {
    const response = await api.get<ApiResponse<PluginsResponse>>('/plugins')
    return response.data.data!.plugins
}

export const addPlugin = async (request: PluginRequest): Promise<string> => {
    const response = await api.post<ApiResponse<string>>('/plugins', request)
    return response.data.data!
}

export const updatePlugin = async (id: string, request: PluginRequest): Promise<string> => {
    const response = await api.put<ApiResponse<string>>(`/plugins/${encodeURIComponent(id)}`, request)
    return response.data.data!
}

export const deletePlugin = async (id: string): Promise<string> => {
    const response = await api.delete<ApiResponse<string>>(`/plugins/${encodeURIComponent(id)}`)
    return response.data.data!
}

export const togglePlugin = async (id: string): Promise<string> => {
    const response = await api.put<ApiResponse<string>>(`/plugins/${encodeURIComponent(id)}/toggle`)
    return response.data.data!
}
