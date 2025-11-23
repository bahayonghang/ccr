import { ref } from 'vue'
import axios from 'axios'

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
            const response = await axios.get('/api/skills')
            if (response.data.success) {
                skills.value = response.data.data
            } else {
                error.value = response.data.error
            }
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
            const response = await axios.post('/api/skills', req)
            if (!response.data.success) {
                throw new Error(response.data.error)
            }
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
            const response = await axios.put(`/api/skills/${encodeURIComponent(name)}`, req)
            if (!response.data.success) {
                throw new Error(response.data.error)
            }
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
            const response = await axios.delete(`/api/skills/${encodeURIComponent(name)}`)
            if (!response.data.success) {
                throw new Error(response.data.error)
            }
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
