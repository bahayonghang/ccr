/**
 * Skills Management API Module
 *
 * 技能管理和技能仓库管理 API
 */

import { api } from '../core'
import type { ApiResponse } from '@/types'

// ═══════════════════════════════════════════════════════════
// 📋 类型定义 (Types)
// ═══════════════════════════════════════════════════════════

export interface Skill {
    name: string
    description?: string
    path: string
    instruction: string
    metadata?: {
        author?: string
        version?: string
        license?: string
        tags?: string[]
        updated_at?: string
    }
    is_remote?: boolean
    repository?: string
}

export interface SkillRepository {
    name: string
    url: string
    branch: string
    description?: string
    skill_count?: number
    last_synced?: string
    is_official?: boolean
}

export interface AddRepositoryRequest {
    name: string
    url: string
    branch?: string
    description?: string
}

// ═══════════════════════════════════════════════════════════
// 🛠️ 技能管理 API (Skills)
// ═══════════════════════════════════════════════════════════

export const listSkills = async (): Promise<Skill[]> => {
    const response = await api.get<ApiResponse<Skill[]>>('/skills')
    return response.data.data!
}

export const addSkill = async (name: string, instruction: string): Promise<string> => {
    const response = await api.post<ApiResponse<string>>('/skills', { name, instruction })
    return response.data.data!
}

export const deleteSkill = async (name: string): Promise<string> => {
    const response = await api.delete<ApiResponse<string>>(`/skills/${encodeURIComponent(name)}`)
    return response.data.data!
}

// ═══════════════════════════════════════════════════════════
// 📦 技能仓库管理 API (Skill Repositories)
// ═══════════════════════════════════════════════════════════

export const listSkillRepositories = async (): Promise<SkillRepository[]> => {
    const response = await api.get<ApiResponse<SkillRepository[]>>('/skills/repositories')
    return response.data.data!
}

export const addSkillRepository = async (request: AddRepositoryRequest): Promise<string> => {
    const response = await api.post<ApiResponse<string>>('/skills/repositories', request)
    return response.data.data!
}

export const removeSkillRepository = async (name: string): Promise<string> => {
    const response = await api.delete<ApiResponse<string>>(`/skills/repositories/${encodeURIComponent(name)}`)
    return response.data.data!
}

export const scanSkillRepository = async (name: string): Promise<Skill[]> => {
    const response = await api.get<ApiResponse<Skill[]>>(`/skills/repositories/${encodeURIComponent(name)}/scan`)
    return response.data.data!
}
