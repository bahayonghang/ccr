/**
 * Output Styles Management API Module
 *
 * Claude Code 输出样式管理 API
 */

import { api } from '../core'
import type {
    ApiResponse,
    OutputStyle,
    OutputStyleRequest,
    UpdateOutputStyleRequest,
} from '@/types'

export const listOutputStyles = async (): Promise<OutputStyle[]> => {
    const response = await api.get<ApiResponse<OutputStyle[]>>('/output-styles')
    return response.data.data!
}

export const getOutputStyle = async (name: string): Promise<OutputStyle> => {
    const response = await api.get<ApiResponse<OutputStyle>>(`/output-styles/${encodeURIComponent(name)}`)
    return response.data.data!
}

export const createOutputStyle = async (request: OutputStyleRequest): Promise<string> => {
    const response = await api.post<ApiResponse<string>>('/output-styles', request)
    return response.data.data!
}

export const updateOutputStyle = async (name: string, request: UpdateOutputStyleRequest): Promise<string> => {
    const response = await api.put<ApiResponse<string>>(`/output-styles/${encodeURIComponent(name)}`, request)
    return response.data.data!
}

export const deleteOutputStyle = async (name: string): Promise<string> => {
    const response = await api.delete<ApiResponse<string>>(`/output-styles/${encodeURIComponent(name)}`)
    return response.data.data!
}
