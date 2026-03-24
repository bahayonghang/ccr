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

type UnknownRecord = Record<string, unknown>

function asRecord(value: unknown): UnknownRecord {
  return typeof value === 'object' && value !== null ? (value as UnknownRecord) : {}
}

function asArray(value: unknown): unknown[] {
  return Array.isArray(value) ? value : []
}

function pickArray(value: unknown, key: string): unknown[] {
  return asArray(asRecord(value)[key])
}

function toStringArray(value: unknown): string[] {
  return asArray(value).filter((item): item is string => typeof item === 'string')
}

function toBackendSkillItem(raw: unknown): BackendSkillItem {
  const source = asRecord(raw)
  return {
    name: String(source.name ?? ''),
    description: typeof source.description === 'string' ? source.description : undefined,
    skill_dir: String(source.skill_dir ?? source.skillDir ?? ''),
    platform: String(source.platform ?? ''),
    platform_name: String(source.platform_name ?? source.platformName ?? source.platform ?? ''),
    category: typeof source.category === 'string' ? source.category : undefined,
    tags: toStringArray(source.tags),
    version: typeof source.version === 'string' ? source.version : undefined,
    author: typeof source.author === 'string' ? source.author : undefined,
    source: typeof source.source === 'string' ? source.source : undefined,
    source_url: typeof source.source_url === 'string' ? source.source_url : undefined,
    install_date: typeof source.install_date === 'number' ? source.install_date : undefined,
    commit_hash: typeof source.commit_hash === 'string' ? source.commit_hash : undefined,
  }
}

export function useUnifiedSkills() {
  const store = useSkillsStore()
  const {
    skills,
    platforms,
    marketplaceItems,
    marketplaceLoaded,
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
    if (normalized === 'qoder') return 'qoder'
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

  function transformPlatform(raw: unknown): PlatformSummary {
    const source = asRecord(raw)
    const id = normalizePlatformId(String(source.id ?? source.platform ?? ''))
    return {
      id,
      display_name: String(source.display_name ?? source.displayName ?? source.name ?? id),
      global_skills_dir: String(source.global_skills_dir ?? source.globalSkillsDir ?? ''),
      detected: Boolean(source.detected ?? true),
      installed_count: Number(source.installed_count ?? source.installedCount ?? 0),
    }
  }

  function toOperationResponse(raw: unknown): OperationResponse {
    const source = asRecord(raw)
    const rawResults = source.results
    if (Array.isArray(rawResults)) return raw as OperationResponse
    if (Array.isArray(raw)) {
      return {
        results: raw.map((item) => {
          const row = asRecord(item)
          return {
            agent: String(row.agent ?? row.platform ?? 'unknown'),
            ok: Boolean(row.ok ?? row.success ?? true),
            message: typeof row.message === 'string' ? row.message : undefined,
          }
        }),
      }
    }
    return {
      results: [
        {
          agent: 'unknown',
          ok: Boolean(source.ok ?? source.success ?? true),
          message: typeof source.message === 'string' ? source.message : undefined,
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
      const response = await getSkillHubUnified<UnknownRecord>()
      const platformList = pickArray(response, 'platforms')
      const normalizedPlatforms = platformList.map(transformPlatform)
      store.setPlatforms(normalizedPlatforms)

      const skillsRaw = pickArray(response, 'skills')
      const allSkills: UnifiedSkill[] = skillsRaw.map((s) => transformSkill(toBackendSkillItem(s)))
      store.setSkills(allSkills)
    } catch (err) {
      // unified 命令失败时回退到 N+1 模式
      logger.warn('[useUnifiedSkills] unified 查询失败，回退到逐平台查询', err)
      try {
        const platformRows = await getSkillHubAgents<unknown>()
        const platformList = Array.isArray(platformRows)
          ? platformRows
          : (pickArray(platformRows, 'platforms').length > 0 ? pickArray(platformRows, 'platforms') : pickArray(platformRows, 'agents'))
        const normalizedPlatforms = platformList.map(transformPlatform)
        store.setPlatforms(normalizedPlatforms)

        const allSkills: UnifiedSkill[] = []
        for (const p of normalizedPlatforms) {
          try {
            const rows = await getSkillHubAgentSkills<unknown>(p.id)
            const skillsRaw = Array.isArray(rows)
              ? rows
              : (pickArray(rows, 'skills').length > 0 ? pickArray(rows, 'skills') : pickArray(asRecord(rows).data, 'skills'))
            allSkills.push(...skillsRaw.map((s) => transformSkill(toBackendSkillItem(s))))
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
      const rows = await getSkillHubAgentSkills<unknown>(platform)
      const skillsRaw = Array.isArray(rows)
        ? rows
        : (pickArray(rows, 'skills').length > 0 ? pickArray(rows, 'skills') : pickArray(asRecord(rows).data, 'skills'))

      const otherSkills = skills.value.filter((s) => s.platform !== platform)
      const newSkills = skillsRaw.map((s) => transformSkill(toBackendSkillItem(s)))
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
      const response = await getSkillHubTrending<unknown>()
      // TODO: 当前 Tauri API 不支持 limit/page，先使用后端默认分页
      void limit
      void page
      const data: BackendMarketplaceResponse = Array.isArray(response)
        ? { items: response as BackendMarketplaceItem[] }
        : {
          items: pickArray(response, 'items') as BackendMarketplaceItem[],
          total: Number(asRecord(response).total ?? 0),
          cached: Boolean(asRecord(response).cached ?? false),
        }

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
      const response = await searchSkillHubMarketplace<unknown>(query)
      // TODO: 当前 Tauri API 不支持 limit/page，先使用后端默认分页
      void limit
      void page
      const data: BackendMarketplaceResponse = Array.isArray(response)
        ? { items: response as BackendMarketplaceItem[] }
        : {
          items: pickArray(response, 'items') as BackendMarketplaceItem[],
          total: Number(asRecord(response).total ?? 0),
          cached: Boolean(asRecord(response).cached ?? false),
        }

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
      const response = await getSkillHubAgents<unknown>()
      const platformList = Array.isArray(response)
        ? response
        : (pickArray(response, 'platforms').length > 0 ? pickArray(response, 'platforms') : pickArray(response, 'agents'))
      store.setPlatforms(platformList.map(transformPlatform))
    } catch (err) {
      logger.error('Failed to fetch platforms', err)
    }
  }

  let initPromise: Promise<void> | null = null
  async function initialize(preloadMarketplace = false) {
    if (isSkillsCacheValid.value && (!preloadMarketplace || isMarketplaceCacheValid.value)) return
    if (initPromise) return initPromise
    const tasks: Array<Promise<unknown>> = [fetchAllSkills()]
    if (preloadMarketplace) {
      tasks.push(fetchMarketplaceTrending())
    }
    initPromise = Promise.all(tasks).then(() => {})
    return initPromise
  }

  async function refresh(includeMarketplace = false) {
    initPromise = null
    const tasks: Array<Promise<unknown>> = [fetchAllSkills(true)]
    if (includeMarketplace) {
      tasks.push(fetchMarketplaceTrending(30, 1, true))
    }
    await Promise.all(tasks)
  }

  async function refreshMarketplaceCache() {
    // TODO: @/api 暂无 refresh marketplace cache 命令，回退为强制刷新
    await fetchMarketplaceTrending(30, 1, true)
  }

  async function fetchSkillContent(skillDir: string): Promise<SkillContent> {
    const result = await getSkillHubSkillContent<unknown>(skillDir)
    const source = asRecord(result)
    return {
      name: String(source.name ?? ''),
      description: typeof source.description === 'string' ? source.description : undefined,
      category: typeof source.category === 'string' ? source.category : undefined,
      tags: asArray(source.tags) as string[],
      content: String(source.content ?? ''),
      raw: String(source.raw ?? ''),
      skillDir: String(source.skill_dir ?? skillDir),
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
    const result = await importSkillViaNpx<unknown>(request.package, request.agents, request.global)
    const source = asRecord(result)
    const resultRows = asArray(source.results)
    await scheduleMutationRefresh()
    return {
      success: Boolean(source.success),
      method: String(source.method ?? 'npx'),
      stdout: typeof source.stdout === 'string' ? source.stdout : undefined,
      stderr: typeof source.stderr === 'string' ? source.stderr : undefined,
      results: resultRows.map((r) => {
        const row = asRecord(r)
        return {
          agent: String(row.agent ?? 'unknown'),
          ok: Boolean(row.ok ?? row.success),
          message: typeof row.message === 'string' ? row.message : undefined,
        }
      }),
    }
  }

  async function batchInstall(request: BatchInstallRequest): Promise<BatchInstallResponse> {
    const result = await batchInstallSkills<unknown>(request.packages, request.agents, request.force)
    const source = asRecord(result)
    const resultRows = asArray(source.results)
    await scheduleMutationRefresh()
    return {
      total: Number(source.total ?? 0),
      successCount: Number(source.success_count ?? 0),
      failCount: Number(source.fail_count ?? 0),
      results: resultRows.map((r) => {
        const row = asRecord(r)
        return {
          package: String(row.package ?? ''),
          ok: Boolean(row.ok),
          message: typeof row.message === 'string' ? row.message : undefined,
        }
      }),
    }
  }

  async function checkNpxStatus(): Promise<NpxStatus> {
    try {
      const result = await checkNpxAvailability<unknown>()
      const source = asRecord(result)
      const status: NpxStatus = {
        available: Boolean(source.available),
        version: typeof source.version === 'string' ? source.version : undefined,
        path: typeof source.path === 'string' ? source.path : undefined,
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
      const result = await browseForFolder<unknown>()
      const source = asRecord(result)
      return typeof source.path === 'string' ? source.path : null
    } catch {
      return null
    }
  }

  return {
    skills,
    platforms,
    marketplaceItems,
    marketplaceLoaded,
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
    setFilters: store.setFilters,
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
