/* eslint-disable no-console -- Development debugging, console output acceptable */
import { ref } from 'vue'
import {
  addSkill as addSkillApi,
  deleteSkill as deleteSkillApi,
  listSkills as listSkillsApi,
  getSkillDetail,
  updateSkillContent,
} from '@/api'

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
            const result = await listSkillsApi()
            if (Array.isArray(result)) {
                skills.value = result
            } else {
                skills.value = []
                console.warn('[useSkills] Unexpected response format:', result)
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
            const result = await getSkillDetail(name)
            currentSkill.value = result as Skill
            return result as Skill
        } catch (err: any) {
            error.value = err.message || 'Failed to load skill'
            currentSkill.value = null
            throw err
        } finally {
            loading.value = false
        }
    }

    const addSkill = async (req: CreateSkillRequest) => {
        loading.value = true
        error.value = null
        try {
            await addSkillApi(req)
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
            await updateSkillContent(name, req.instruction)
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
            await deleteSkillApi(name)
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
