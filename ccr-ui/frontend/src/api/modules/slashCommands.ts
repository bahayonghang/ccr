/**
 * Slash Command Management API Module
 *
 * Claude Code 斜杠命令管理 CRUD API
 */

import { api } from '../core'
import type {
    ApiResponse,
    SlashCommandRequest,
    SlashCommandsResponse,
} from '@/types'

export const listSlashCommands = async (): Promise<SlashCommandsResponse> => {
    const response = await api.get<ApiResponse<SlashCommandsResponse>>('/slash-commands')
    return response.data.data!
}

export const addSlashCommand = async (request: SlashCommandRequest): Promise<string> => {
    const response = await api.post<ApiResponse<string>>('/slash-commands', request)
    return response.data.data!
}

export const updateSlashCommand = async (name: string, request: SlashCommandRequest): Promise<string> => {
    const response = await api.put<ApiResponse<string>>(`/slash-commands/${encodeURIComponent(name)}`, request)
    return response.data.data!
}

export const deleteSlashCommand = async (name: string): Promise<string> => {
    const response = await api.delete<ApiResponse<string>>(`/slash-commands/${encodeURIComponent(name)}`)
    return response.data.data!
}

export const toggleSlashCommand = async (name: string): Promise<string> => {
    const response = await api.put<ApiResponse<string>>(`/slash-commands/${encodeURIComponent(name)}/toggle`)
    return response.data.data!
}
