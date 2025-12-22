import { ref } from 'vue'
import { api } from '@/api/client'

export interface Skill {
    name: string
    description?: string
    path: string
    instruction: string
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
    const loading = ref(false)
    const error = ref<string | null>(null)

    const listSkills = async () => {
        loading.value = true
        error.value = null
        try {
            const response = await api.get('/skills')
            skills.value = response.data
        } catch (err: any) {
            error.value = err.message || 'Failed to load skills'
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
        loading,
        error,
        listSkills,
        addSkill,
        updateSkill,
        deleteSkill
    }
}
