/**
 * Hooks Management API Module
 *
 * Claude Code Hooks 管理 CRUD API
 */

import { api } from '../core'
import type {
    ApiResponse,
    Hook,
    HookRequest,
    HooksResponse,
} from '@/types'

export const listHooks = async (): Promise<Hook[]> => {
    const response = await api.get<ApiResponse<HooksResponse>>('/hooks')
    return response.data.data!.hooks
}

export const addHook = async (request: HookRequest): Promise<string> => {
    const response = await api.post<ApiResponse<string>>('/hooks', request)
    return response.data.data!
}

export const updateHook = async (name: string, request: HookRequest): Promise<string> => {
    const response = await api.put<ApiResponse<string>>(`/hooks/${encodeURIComponent(name)}`, request)
    return response.data.data!
}

export const deleteHook = async (name: string): Promise<string> => {
    const response = await api.delete<ApiResponse<string>>(`/hooks/${encodeURIComponent(name)}`)
    return response.data.data!
}

export const toggleHook = async (name: string): Promise<string> => {
    const response = await api.put<ApiResponse<string>>(`/hooks/${encodeURIComponent(name)}/toggle`)
    return response.data.data!
}
