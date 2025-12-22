// 技能数据缓存 Composable
// 提供内存缓存和持久化缓存功能

import { ref, computed } from 'vue'
import { listSkills, listSkillRepositories, type Skill, type SkillRepository } from '@/api/client'

// 缓存配置
const CACHE_TTL = 5 * 60 * 1000 // 5 分钟
const CACHE_KEY = 'ccr_skills_cache'

// 缓存数据结构
interface CacheData {
    skills: Skill[]
    repositories: SkillRepository[]
    timestamp: number
}

// 内存缓存
const memoryCache = ref<CacheData | null>(null)
const isLoading = ref(false)
const lastError = ref<string | null>(null)

/**
 * 检查缓存是否过期
 */
const isCacheExpired = (timestamp: number): boolean => {
    return Date.now() - timestamp > CACHE_TTL
}

/**
 * 从 localStorage 读取缓存
 */
const loadFromStorage = (): CacheData | null => {
    try {
        const stored = localStorage.getItem(CACHE_KEY)
        if (!stored) return null
        const data = JSON.parse(stored) as CacheData
        if (isCacheExpired(data.timestamp)) {
            localStorage.removeItem(CACHE_KEY)
            return null
        }
        return data
    } catch {
        return null
    }
}

/**
 * 保存到 localStorage
 */
const saveToStorage = (data: CacheData) => {
    try {
        localStorage.setItem(CACHE_KEY, JSON.stringify(data))
    } catch (e) {
        console.warn('Failed to save cache to localStorage:', e)
    }
}

/**
 * 清除所有缓存
 */
const clearCache = () => {
    memoryCache.value = null
    localStorage.removeItem(CACHE_KEY)
}

/**
 * 从 API 刷新数据
 */
const refreshFromApi = async (): Promise<CacheData> => {
    const [skills, repositories] = await Promise.all([
        listSkills(),
        listSkillRepositories()
    ])

    const data: CacheData = {
        skills,
        repositories,
        timestamp: Date.now()
    }

    memoryCache.value = data
    saveToStorage(data)
    return data
}

/**
 * 获取缓存数据（带自动刷新）
 */
const getCachedData = async (forceRefresh = false): Promise<CacheData> => {
    // 强制刷新
    if (forceRefresh) {
        return refreshFromApi()
    }

    // 检查内存缓存
    if (memoryCache.value && !isCacheExpired(memoryCache.value.timestamp)) {
        return memoryCache.value
    }

    // 检查 localStorage 缓存
    const storedData = loadFromStorage()
    if (storedData) {
        memoryCache.value = storedData
        return storedData
    }

    // 从 API 获取
    return refreshFromApi()
}

/**
 * 技能缓存 Composable
 */
export function useSkillsCache() {
    const skills = computed(() => memoryCache.value?.skills || [])
    const repositories = computed(() => memoryCache.value?.repositories || [])
    const cacheAge = computed(() => {
        if (!memoryCache.value) return null
        return Math.round((Date.now() - memoryCache.value.timestamp) / 1000)
    })

    /**
     * 加载数据（优先使用缓存）
     */
    const load = async (forceRefresh = false) => {
        if (isLoading.value) return

        try {
            isLoading.value = true
            lastError.value = null
            await getCachedData(forceRefresh)
        } catch (error) {
            lastError.value = error instanceof Error ? error.message : 'Unknown error'
            console.error('Failed to load skills cache:', error)
        } finally {
            isLoading.value = false
        }
    }

    /**
     * 强制刷新
     */
    const refresh = () => load(true)

    /**
     * 按标签筛选技能
     */
    const filterByTags = (tags: string[]): Skill[] => {
        if (tags.length === 0) return skills.value
        return skills.value.filter(skill => {
            const skillTags = skill.metadata?.tags || []
            return tags.some(t => skillTags.includes(t))
        })
    }

    /**
     * 按仓库筛选技能
     */
    const filterByRepository = (repoName: string | null): Skill[] => {
        if (repoName === null) return skills.value
        if (repoName === '') {
            // 本地技能
            return skills.value.filter(s => !s.is_remote)
        }
        return skills.value.filter(s => s.repository === repoName)
    }

    /**
     * 搜索技能
     */
    const search = (query: string): Skill[] => {
        if (!query.trim()) return skills.value
        const q = query.toLowerCase()
        return skills.value.filter(skill => {
            return (
                skill.name.toLowerCase().includes(q) ||
                skill.description?.toLowerCase().includes(q) ||
                skill.metadata?.author?.toLowerCase().includes(q) ||
                skill.metadata?.tags?.some(t => t.toLowerCase().includes(q))
            )
        })
    }

    /**
     * 获取所有可用标签
     */
    const getAllTags = (): string[] => {
        const tags = new Set<string>()
        skills.value.forEach(s => {
            s.metadata?.tags?.forEach(tag => tags.add(tag))
        })
        return Array.from(tags).sort()
    }

    /**
     * 获取所有仓库名称
     */
    const getAllRepositories = (): string[] => {
        const repos = new Set<string>()
        skills.value.forEach(s => {
            if (s.repository) repos.add(s.repository)
        })
        return Array.from(repos).sort()
    }

    return {
        // 状态
        skills,
        repositories,
        isLoading,
        lastError,
        cacheAge,

        // 方法
        load,
        refresh,
        clearCache,

        // 筛选
        filterByTags,
        filterByRepository,
        search,
        getAllTags,
        getAllRepositories
    }
}
