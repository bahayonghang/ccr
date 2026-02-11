/**
 * Agent Management API Module
 *
 * Claude Code Agent 管理 CRUD API
 */

import { api } from '../core'
import type {
    ApiResponse,
    Agent,
    AgentRequest,
    AgentsResponse,
} from '@/types'

export const listAgents = async (): Promise<AgentsResponse> => {
    const response = await api.get<ApiResponse<AgentsResponse>>('/agents')
    return response.data.data!
}

export const getAgent = async (name: string): Promise<Agent> => {
    const response = await api.get<ApiResponse<Agent>>(`/agents/${encodeURIComponent(name)}`)
    return response.data.data!
}

export const addAgent = async (request: AgentRequest): Promise<string> => {
    const response = await api.post<ApiResponse<string>>('/agents', request)
    return response.data.data!
}

export const updateAgent = async (name: string, request: AgentRequest): Promise<string> => {
    const response = await api.put<ApiResponse<string>>(`/agents/${encodeURIComponent(name)}`, request)
    return response.data.data!
}

export const deleteAgent = async (name: string): Promise<string> => {
    const response = await api.delete<ApiResponse<string>>(`/agents/${encodeURIComponent(name)}`)
    return response.data.data!
}

export const toggleAgent = async (name: string): Promise<string> => {
    const response = await api.put<ApiResponse<string>>(`/agents/${encodeURIComponent(name)}/toggle`)
    return response.data.data!
}
