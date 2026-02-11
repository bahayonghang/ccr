import { ref } from 'vue'
import { api } from '@/api/client'

export interface SkillMetadata {
    author?: string
    version?: string
    license?: string
    category?: string
    tags?: string[]
    updated_at?: string
}

export interface Skill {
    name: string
    description?: string
    path: string
    instruction: string
    metadata?: SkillMetadata
    is_remote?: boolean
    /** Plugin name (for plugin skills, e.g., "plugin-name@marketplace-name") */
    repository?: string
}

export interface CreateSkillRequest {
    name: string
    instruction: string
}

export interface UpdateSkillRequest {
    instruction: string
}

export function useSkills() {
    const skills = ref<Skill[]>([])
    const currentSkill = ref<Skill | null>(null)
    const loading = ref(false)
    const error = ref<string | null>(null)

    const listSkills = async () => {
        loading.value = true
        error.value = null
        try {
            const response = await api.get('/skills')
            // API returns ApiResponse<Skill[]> format: { success: true, data: [...] }
            const rawData = response.data
            if (rawData && typeof rawData === 'object' && 'data' in rawData && Array.isArray(rawData.data)) {
                skills.value = rawData.data
            } else if (Array.isArray(rawData)) {
                skills.value = rawData
            } else {
                skills.value = []
                console.warn('[useSkills] Unexpected response format:', rawData)
            }
        } catch (err: any) {
            error.value = err.message || 'Failed to load skills'
            skills.value = []
        } finally {
            loading.value = false
        }
    }

    const getSkill = async (name: string) => {
        loading.value = true
        error.value = null
        try {
            const response = await api.get(`/skills/${encodeURIComponent(name)}`)
            // API returns ApiResponse<Skill> format: { success: true, data: {...} }
            const rawData = response.data
            let skill: Skill | null = null
            if (rawData && typeof rawData === 'object' && 'data' in rawData && rawData.data) {
                skill = rawData.data as Skill
            } else if (rawData && 'name' in rawData) {
                skill = rawData as Skill
            }
            currentSkill.value = skill
            return skill
        } catch (err: any) {
            error.value = err.message || 'Failed to load skill'
            throw err
        } finally {
            loading.value = false
        }
    }

    const addSkill = async (req: CreateSkillRequest) => {
        loading.value = true
        error.value = null
        try {
            await api.post('/skills', req)
            await listSkills()
        } catch (err: any) {
            error.value = err.message || 'Failed to add skill'
            throw err
        } finally {
            loading.value = false
        }
    }

    const updateSkill = async (name: string, req: UpdateSkillRequest) => {
        loading.value = true
        error.value = null
        try {
            await api.put(`/skills/${encodeURIComponent(name)}`, req)
            await listSkills()
        } catch (err: any) {
            error.value = err.message || 'Failed to update skill'
            throw err
        } finally {
            loading.value = false
        }
    }

    const deleteSkill = async (name: string) => {
        loading.value = true
        error.value = null
        try {
            await api.delete(`/skills/${encodeURIComponent(name)}`)
            await listSkills()
        } catch (err: any) {
            error.value = err.message || 'Failed to delete skill'
            throw err
        } finally {
            loading.value = false
        }
    }

    return {
        skills,
        currentSkill,
        loading,
        error,
        listSkills,
        getSkill,
        addSkill,
        updateSkill,
        deleteSkill
    }
}
