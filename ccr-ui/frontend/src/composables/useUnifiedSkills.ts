/**
 * Unified Skills Composable
 * 统一 Skills 管理组合式 API
 */
import { storeToRefs } from 'pinia'
import { useSkillsStore } from '@/stores/skills'
import {
  getSkillHubTrending,
  searchSkillHubMarketplace,
  getSkillHubAgents,
  getSkillHubAgentSkills,
  installSkillHubSkill,
  removeSkillHubSkill,
  getSkillHubUnified,
  getSkillHubSkillContent,
  saveSkillHubSkillContent,
  importSkillFromGithub,
  importSkillFromLocal,
  importSkillViaNpx,
  batchInstallSkills,
  checkNpxAvailability,
  browseForFolder,
} from '@/api'
import { logger } from '@/utils/logger'
import type {
  Platform,
  UnifiedSkill,
  InstallRequest,
  RemoveRequest,
  OperationResponse,
  PlatformSummary,
  SkillContent,
  ImportGithubRequest,
  ImportLocalRequest,
  NpxInstallRequest,
  NpxInstallResponse,
  BatchInstallRequest,
  BatchInstallResponse,
  NpxStatus,
} from '@/types/skills'

// Backend response types (snake_case)
interface BackendSkillItem {
  name: string
  description?: string
  skill_dir: string
  platform: string
  platform_name: string
  category?: string
  tags?: string[]
  version?: string
  author?: string
  source?: string
  source_url?: string
  install_date?: number
  commit_hash?: string
}

interface BackendMarketplaceItem {
  package: string
  owner: string
  repo: string
  skill?: string
  skills_sh_url?: string
  description?: string
  author_avatar?: string
  stars?: number
}

interface BackendMarketplaceResponse {
  items: BackendMarketplaceItem[]
  total?: number
  cached?: boolean
}

export function useUnifiedSkills() {
  const store = useSkillsStore()
  const {
    skills,
    platforms,
    marketplaceItems,
    isLoading,
    isMarketplaceLoading,
    error,
    marketplaceError,
    marketplaceCached,
    filters,
    activeTab,
    filteredSkills,
    availableCategories,
    availableTags,
    stats,
    skillsByPlatform,
    isInstalling,
    installProgress,
    batchMode,
    batchSelection,
    npxStatus,
    marketplacePage,
    marketplaceTotal,
    marketplacePageSize,
    isSkillsCacheValid,
    isMarketplaceCacheValid,
  } = storeToRefs(store)

  function normalizePlatformId(value: string): Platform {
    const normalized = value.toLowerCase()
    if (normalized === 'claude' || normalized === 'claude-code') return 'claude-code'
    if (normalized === 'codex') return 'codex'
    if (normalized === 'gemini') return 'gemini'
    if (normalized === 'qwen') return 'qwen'
    if (normalized === 'iflow') return 'iflow'
    return 'droid'
  }

  function transformSkill(skill: BackendSkillItem): UnifiedSkill {
    return {
      name: skill.name,
      description: skill.description,
      skillDir: skill.skill_dir,
      platform: normalizePlatformId(skill.platform),
      platformName: skill.platform_name,
      category: skill.category,
      tags: skill.tags || [],
      version: skill.version,
      author: skill.author,
      source: skill.source,
      sourceUrl: skill.source_url,
      installDate: skill.install_date,
      commitHash: skill.commit_hash,
    }
  }

  function transformPlatform(raw: any): PlatformSummary {
    const id = normalizePlatformId(raw?.id || raw?.platform || '')
    return {
      id,
      display_name: raw?.display_name || raw?.displayName || raw?.name || id,
      global_skills_dir: raw?.global_skills_dir || raw?.globalSkillsDir || '',
      detected: Boolean(raw?.detected ?? true),
      installed_count: Number(raw?.installed_count ?? raw?.installedCount ?? 0),
    }
  }

  function toOperationResponse(raw: any): OperationResponse {
    if (Array.isArray(raw?.results)) return raw as OperationResponse
    if (Array.isArray(raw)) {
      return {
        results: raw.map((item: any) => ({
          agent: item?.agent || item?.platform || 'unknown',
          ok: Boolean(item?.ok ?? item?.success ?? true),
          message: item?.message,
        })),
      }
    }
    return {
      results: [
        {
          agent: 'unknown',
          ok: Boolean(raw?.ok ?? raw?.success ?? true),
          message: raw?.message,
        },
      ],
    }
  }

  let mutationRefreshTimer: ReturnType<typeof setTimeout> | null = null
  let mutationRefreshPromise: Promise<void> | null = null
  let mutationRefreshResolve: (() => void) | null = null

  function scheduleMutationRefresh(delayMs: number = 200): Promise<void> {
    if (!mutationRefreshPromise) {
      mutationRefreshPromise = new Promise<void>((resolve) => {
        mutationRefreshResolve = resolve
      })
    }

    if (mutationRefreshTimer) clearTimeout(mutationRefreshTimer)
    mutationRefreshTimer = setTimeout(async () => {
      mutationRefreshTimer = null
      await fetchAllSkills()
      mutationRefreshResolve?.()
      mutationRefreshPromise = null
      mutationRefreshResolve = null
    }, delayMs)

    return mutationRefreshPromise
  }

  async function fetchAllSkills(force = false) {
    if (!force && isSkillsCacheValid.value) return

    store.setLoading(true)
    store.setError(null)

    try {
      // 使用 unified 命令一次性获取所有平台数据，避免 N+1 查询
      const response = await getSkillHubUnified()
      const platformList = Array.isArray(response?.platforms)
        ? response.platforms
        : []
      const normalizedPlatforms = platformList.map(transformPlatform)
      store.setPlatforms(normalizedPlatforms)

      const skillsRaw = Array.isArray(response?.skills)
        ? response.skills
        : []
      const allSkills: UnifiedSkill[] = skillsRaw.map((s: BackendSkillItem) => transformSkill(s))
      store.setSkills(allSkills)
    } catch (err) {
      // unified 命令失败时回退到 N+1 模式
      logger.warn('[useUnifiedSkills] unified 查询失败，回退到逐平台查询', err)
      try {
        const platformRows = await getSkillHubAgents()
        const platformList = Array.isArray(platformRows)
          ? platformRows
          : (platformRows?.platforms || platformRows?.agents || [])
        const normalizedPlatforms = platformList.map(transformPlatform)
        store.setPlatforms(normalizedPlatforms)

        const allSkills: UnifiedSkill[] = []
        for (const p of normalizedPlatforms) {
          try {
            const rows = await getSkillHubAgentSkills(p.id)
            const skillsRaw = Array.isArray(rows)
              ? rows
              : (rows?.skills || rows?.data?.skills || [])
            allSkills.push(...skillsRaw.map((s: BackendSkillItem) => transformSkill(s)))
          } catch (innerErr) {
            logger.warn(`[useUnifiedSkills] 拉取平台 ${p.id} 的 skills 失败`, innerErr)
          }
        }
        store.setSkills(allSkills)
      } catch (fallbackErr) {
        const errorMessage = fallbackErr instanceof Error ? fallbackErr.message : 'Failed to fetch skills'
        store.setError(errorMessage)
        logger.error('Failed to fetch unified skills', fallbackErr)
      }
    } finally {
      store.setLoading(false)
    }
  }

  async function fetchSkillsByPlatform(platform: Platform) {
    store.setLoading(true)
    store.setError(null)

    try {
      const rows = await getSkillHubAgentSkills(platform)
      const skillsRaw = Array.isArray(rows)
        ? rows
        : (rows?.skills || rows?.data?.skills || [])

      const otherSkills = skills.value.filter((s) => s.platform !== platform)
      const newSkills = skillsRaw.map((s: BackendSkillItem) => transformSkill(s))
      store.setSkills([...otherSkills, ...newSkills])
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : `Failed to fetch skills for ${platform}`
      store.setError(errorMessage)
      logger.error(`platform`, err)
    } finally {
      store.setLoading(false)
    }
  }

  async function fetchMarketplaceTrending(limit: number = 30, page: number = 1, force = false) {
    if (!force && isMarketplaceCacheValid.value) return

    store.setMarketplaceLoading(true)
    store.setMarketplaceError(null)

    try {
      const response = await getSkillHubTrending()
      // TODO: 当前 Tauri API 不支持 limit/page，先使用后端默认分页
      void limit
      void page
      const data: BackendMarketplaceResponse = Array.isArray(response)
        ? { items: response as BackendMarketplaceItem[] }
        : response

      store.setMarketplaceItems(
        (data.items || []).map((item: BackendMarketplaceItem) => ({
          package: item.package,
          owner: item.owner,
          repo: item.repo,
          skill: item.skill,
          skillsShUrl: item.skills_sh_url || '',
          description: item.description,
          authorAvatar:
            item.author_avatar || `https://avatars.githubusercontent.com/${item.owner}?s=64`,
          stars: item.stars,
        })),
        Boolean(data.cached)
      )
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to fetch marketplace'
      store.setMarketplaceError(errorMessage)
      logger.error('Failed to fetch marketplace', err)
    } finally {
      store.setMarketplaceLoading(false)
    }
  }

  async function searchMarketplace(query: string, limit: number = 30, page: number = 1) {
    store.setMarketplaceLoading(true)
    store.setMarketplaceError(null)

    try {
      const response = await searchSkillHubMarketplace(query)
      // TODO: 当前 Tauri API 不支持 limit/page，先使用后端默认分页
      void limit
      void page
      const data: BackendMarketplaceResponse = Array.isArray(response)
        ? { items: response as BackendMarketplaceItem[] }
        : response

      store.setMarketplaceItems(
        (data.items || []).map((item: BackendMarketplaceItem) => ({
          package: item.package,
          owner: item.owner,
          repo: item.repo,
          skill: item.skill,
          skillsShUrl: item.skills_sh_url || '',
          description: item.description,
          authorAvatar:
            item.author_avatar || `https://avatars.githubusercontent.com/${item.owner}?s=64`,
          stars: item.stars,
        })),
        Boolean(data.cached)
      )
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to search marketplace'
      store.setMarketplaceError(errorMessage)
      logger.error('Failed to search marketplace', err)
    } finally {
      store.setMarketplaceLoading(false)
    }
  }

  async function installSkill(request: InstallRequest): Promise<OperationResponse> {
    try {
      const result = await installSkillHubSkill(request)
      await scheduleMutationRefresh()
      return toOperationResponse(result)
    } catch (err) {
      logger.error('Failed to install skill', err)
      throw err
    }
  }

  async function removeSkill(request: RemoveRequest): Promise<OperationResponse> {
    try {
      const result = await removeSkillHubSkill(request.skill)
      await scheduleMutationRefresh()
      return toOperationResponse(result)
    } catch (err) {
      logger.error('Failed to remove skill', err)
      throw err
    }
  }

  async function fetchPlatforms() {
    try {
      const response = await getSkillHubAgents()
      const platformList = Array.isArray(response)
        ? response
        : (response?.platforms || response?.agents || [])
      store.setPlatforms(platformList.map(transformPlatform))
    } catch (err) {
      logger.error('Failed to fetch platforms', err)
    }
  }

  let initPromise: Promise<void> | null = null
  async function initialize() {
    if (isSkillsCacheValid.value && isMarketplaceCacheValid.value) return
    if (initPromise) return initPromise
    initPromise = Promise.all([fetchAllSkills(), fetchMarketplaceTrending()]).then(() => {})
    return initPromise
  }

  async function refresh() {
    initPromise = null
    await Promise.all([fetchAllSkills(true), fetchMarketplaceTrending(30, 1, true)])
  }

  async function refreshMarketplaceCache() {
    // TODO: @/api 暂无 refresh marketplace cache 命令，回退为强制刷新
    await fetchMarketplaceTrending(30, 1, true)
  }

  async function fetchSkillContent(skillDir: string): Promise<SkillContent> {
    const result = await getSkillHubSkillContent(skillDir)
    return {
      name: result?.name || '',
      description: result?.description,
      category: result?.category,
      tags: result?.tags || [],
      content: result?.content || '',
      raw: result?.raw || '',
      skillDir: result?.skill_dir || skillDir,
    }
  }

  async function saveSkillContent(skillDir: string, content: string): Promise<void> {
    await saveSkillHubSkillContent(skillDir, content)
  }

  async function importFromGithub(request: ImportGithubRequest): Promise<OperationResponse> {
    const result = await importSkillFromGithub(request.url, request.agents, request.force)
    const response = toOperationResponse(result)
    await scheduleMutationRefresh()
    return response
  }

  async function importFromLocal(request: ImportLocalRequest): Promise<OperationResponse> {
    const result = await importSkillFromLocal(request.sourcePath, request.agents, request.skillName)
    const response = toOperationResponse(result)
    await scheduleMutationRefresh()
    return response
  }

  async function importViaNpx(request: NpxInstallRequest): Promise<NpxInstallResponse> {
    const result = await importSkillViaNpx(request.package, request.agents, request.global)
    await scheduleMutationRefresh()
    return {
      success: Boolean(result?.success),
      method: result?.method || 'npx',
      stdout: result?.stdout,
      stderr: result?.stderr,
      results: (result?.results || []).map((r: any) => ({
        agent: r?.agent || 'unknown',
        ok: Boolean(r?.ok ?? r?.success),
        message: r?.message,
      })),
    }
  }

  async function batchInstall(request: BatchInstallRequest): Promise<BatchInstallResponse> {
    const result = await batchInstallSkills(request.packages, request.agents, request.force)
    await scheduleMutationRefresh()
    return {
      total: result?.total || 0,
      successCount: result?.success_count || 0,
      failCount: result?.fail_count || 0,
      results: (result?.results || []).map((r: any) => ({
        package: r?.package || '',
        ok: Boolean(r?.ok),
        message: r?.message,
      })),
    }
  }

  async function checkNpxStatus(): Promise<NpxStatus> {
    try {
      const result = await checkNpxAvailability()
      const status: NpxStatus = {
        available: Boolean(result?.available),
        version: result?.version,
        path: result?.path,
      }
      store.setNpxStatus(status)
      return status
    } catch {
      const status: NpxStatus = { available: false }
      store.setNpxStatus(status)
      return status
    }
  }

  async function browseFolder(): Promise<string | null> {
    try {
      const result = await browseForFolder()
      return result?.path ?? null
    } catch {
      return null
    }
  }

  return {
    skills,
    platforms,
    marketplaceItems,
    isLoading,
    isMarketplaceLoading,
    error,
    marketplaceError,
    marketplaceCached,
    filters,
    activeTab,
    isInstalling,
    installProgress,
    batchMode,
    batchSelection,
    npxStatus,
    marketplacePage,
    marketplaceTotal,
    marketplacePageSize,

    filteredSkills,
    availableCategories,
    availableTags,
    stats,
    skillsByPlatform,

    setFilter: store.setFilter,
    resetFilters: store.resetFilters,
    setActiveTab: store.setActiveTab,
    toggleBatchMode: store.toggleBatchMode,
    toggleBatchSelection: store.toggleBatchSelection,
    selectAllBatch: store.selectAllBatch,
    clearBatchSelection: store.clearBatchSelection,
    setMarketplacePage: store.setMarketplacePage,
    setInstalling: store.setInstalling,
    setInstallProgress: store.setInstallProgress,

    fetchAllSkills,
    fetchSkillsByPlatform,
    fetchMarketplaceTrending,
    searchMarketplace,
    installSkill,
    removeSkill,
    fetchPlatforms,
    fetchSkillContent,
    saveSkillContent,
    initialize,
    refresh,
    refreshMarketplaceCache,
    importFromGithub,
    importFromLocal,
    importViaNpx,
    batchInstall,
    checkNpxStatus,
    browseFolder,
  }
}
