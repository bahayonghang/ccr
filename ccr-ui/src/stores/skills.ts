import { defineStore } from 'pinia'
import { computed, ref, shallowRef, triggerRef } from 'vue'
import { useCachedFetch } from '@/composables/useCachedFetch'
import {
  checkNpxAvailability,
  getSkillDetail,
  getSkillHubSkillContent,
  getSkillHubTrending,
  getSkillHubUnified,
  searchSkillHubMarketplace,
  saveSkillHubSkillContent,
  skillsInstall,
  skillsRemoveInstallation,
  skillsRemoveSkill,
  skillsSourceAddGit,
  skillsSourceAddLocal,
  skillsSourceRemove,
  skillsSourceSync,
  skillsSourcesList,
  skillsSync,
} from '@/api'
import type {
  MarketplaceResponse,
  NpxStatus,
  Platform,
  SkillContent,
  SkillFilters,
  SkillLogEntry,
  SkillOperationResponse,
  SkillRecord,
  SkillSourceRecord,
  SkillsInstallRequest,
  SkillsInventoryResponse,
  SkillsRouteState,
  SkillsSyncRequest,
  SkillPlatformSummary,
  UnifiedSkill,
} from '@/types/skills'

type UnknownRecord = Record<string, unknown>
const MARKETPLACE_PAGE_SIZE = 20

const DEFAULT_ROUTE_STATE: SkillsRouteState = {
  tab: 'inventory',
  selected: null,
  mode: 'view',
  platform: 'all',
  origin: 'all',
  q: '',
  page: 1,
  source: null,
}

function asRecord(value: unknown): UnknownRecord {
  return typeof value === 'object' && value !== null ? (value as UnknownRecord) : {}
}

function asArray(value: unknown): unknown[] {
  return Array.isArray(value) ? value : []
}

function normalizePlatform(value: string): Platform {
  const normalized = value.toLowerCase()
  if (normalized === 'claude' || normalized === 'claude-code') return 'claude-code'
  if (normalized === 'codex') return 'codex'
  if (normalized === 'gemini') return 'gemini'
  if (normalized === 'qwen') return 'qwen'
  if (normalized === 'qoder') return 'qoder'
  return 'droid'
}

function toStringArray(value: unknown): string[] {
  return asArray(value).filter((item): item is string => typeof item === 'string')
}

function transformInstallation(raw: unknown) {
  const source = asRecord(raw)
  return {
    id: String(source.id ?? ''),
    platformId: normalizePlatform(String(source.platform_id ?? source.platformId ?? 'codex')),
    platformName: String(source.platform_name ?? source.platformName ?? ''),
    installPath: String(source.install_path ?? source.installPath ?? ''),
    installMode: 'copy' as const,
    installedAt: typeof source.installed_at === 'number' ? source.installed_at : undefined,
    isPrimary: Boolean(source.is_primary ?? source.isPrimary),
  }
}

function transformSkill(raw: unknown): SkillRecord {
  const source = asRecord(raw)
  return {
    id: String(source.id ?? ''),
    name: String(source.name ?? ''),
    description: typeof source.description === 'string' ? source.description : undefined,
    category: typeof source.category === 'string' ? source.category : undefined,
    tags: toStringArray(source.tags),
    version: typeof source.version === 'string' ? source.version : undefined,
    author: typeof source.author === 'string' ? source.author : undefined,
    origin: String(source.origin ?? 'unknown') as SkillRecord['origin'],
    sourceLabel: typeof source.source_label === 'string' ? source.source_label : undefined,
    sourceRef: typeof source.source_ref === 'string' ? source.source_ref : undefined,
    installCount: Number(source.install_count ?? source.installCount ?? 0),
    installations: asArray(source.installations).map(transformInstallation),
    editableInstallations: toStringArray(source.editable_installations ?? source.editableInstallations),
  }
}

function transformPlatform(raw: unknown): SkillPlatformSummary {
  const source = asRecord(raw)
  return {
    id: normalizePlatform(String(source.id ?? 'codex')),
    displayName: String(source.display_name ?? source.displayName ?? ''),
    globalSkillsDir: String(source.global_skills_dir ?? source.globalSkillsDir ?? ''),
    detected: Boolean(source.detected),
    installedCount: Number(source.installed_count ?? source.installedCount ?? 0),
  }
}

function transformInventory(raw: unknown): SkillsInventoryResponse {
  const source = asRecord(raw)
  return {
    skills: asArray(source.skills).map(transformSkill),
    platforms: asArray(source.platforms).map(transformPlatform),
    total: Number(source.total ?? 0),
  }
}

function transformSourceSkill(raw: unknown) {
  const source = asRecord(raw)
  return {
    id: String(source.id ?? ''),
    name: String(source.name ?? ''),
    description: typeof source.description === 'string' ? source.description : undefined,
    category: typeof source.category === 'string' ? source.category : undefined,
    tags: toStringArray(source.tags),
    installRef: String(source.install_ref ?? source.installRef ?? ''),
  }
}

function transformSource(raw: unknown): SkillSourceRecord {
  const source = asRecord(raw)
  return {
    id: String(source.id ?? ''),
    type: String(source.type ?? source.source_type ?? 'local') as SkillSourceRecord['type'],
    name: String(source.name ?? ''),
    description: typeof source.description === 'string' ? source.description : undefined,
    location: String(source.location ?? ''),
    skillsRoot: String(source.skills_root ?? source.skillsRoot ?? ''),
    skillCount: Number(source.skill_count ?? source.skillCount ?? 0),
    lastSyncedAt: typeof source.last_synced_at === 'string' ? source.last_synced_at : undefined,
    health: String(source.health ?? 'ok') as SkillSourceRecord['health'],
    skills: asArray(source.skills).map(transformSourceSkill),
  }
}

function transformMarketplace(raw: unknown): MarketplaceResponse {
  const source = asRecord(raw)
  return {
    items: asArray(source.items).map((item) => {
      const row = asRecord(item)
      return {
        package: String(row.package ?? ''),
        owner: String(row.owner ?? ''),
        repo: String(row.repo ?? ''),
        skill: typeof row.skill === 'string' ? row.skill : undefined,
        skillsShUrl: String(row.skills_sh_url ?? row.skillsShUrl ?? `https://skills.sh/${String(row.package ?? '')}`),
        description: typeof row.description === 'string' ? row.description : undefined,
        authorAvatar: typeof row.author_avatar === 'string'
          ? row.author_avatar
          : typeof row.authorAvatar === 'string'
            ? row.authorAvatar
            : undefined,
        stars: typeof row.stars === 'number' ? row.stars : undefined,
      }
    }),
    total: Number(source.total ?? 0),
    page: Number(source.page ?? 1),
    pageSize: Number(source.page_size ?? source.pageSize ?? 20),
    cached: Boolean(source.cached),
  }
}

function transformContent(raw: unknown): SkillContent {
  const source = asRecord(raw)
  return {
    skillId: String(source.skill_id ?? source.skillId ?? ''),
    installationId: String(source.installation_id ?? source.installationId ?? ''),
    name: String(source.name ?? ''),
    description: typeof source.description === 'string' ? source.description : undefined,
    category: typeof source.category === 'string' ? source.category : undefined,
    tags: toStringArray(source.tags),
    raw: String(source.raw ?? ''),
    content: String(source.content ?? ''),
    skillDir: String(source.skill_dir ?? source.skillDir ?? ''),
  }
}

function transformOperation(raw: unknown): SkillOperationResponse {
  const source = asRecord(raw)
  return {
    results: asArray(source.results).map((item) => {
      const row = asRecord(item)
      return {
        agent: String(row.agent ?? ''),
        ok: Boolean(row.ok),
        message: typeof row.message === 'string' ? row.message : undefined,
      }
    }),
  }
}

function fromUnifiedSkill(skill: UnifiedSkill): SkillRecord {
  return {
    id: `${skill.platform}:${skill.skillDir}`,
    name: skill.name,
    description: skill.description,
    category: skill.category,
    tags: skill.tags,
    version: skill.version,
    author: skill.author,
    origin: skill.source ?? 'unknown',
    sourceLabel: undefined,
    sourceRef: skill.sourceUrl,
    installCount: 1,
    editableInstallations: [`${skill.platform}:${skill.skillDir}`],
    installations: [
      {
        id: `${skill.platform}:${skill.skillDir}`,
        platformId: skill.platform,
        platformName: skill.platformName,
        installPath: skill.skillDir,
        installMode: 'copy',
        installedAt: skill.installDate,
        isPrimary: true,
      },
    ],
  }
}

export const useSkillsStore = defineStore('skills', () => {
  const inventoryCache = useCachedFetch<SkillsInventoryResponse>({
    ttlMs: 60_000,
    initialValue: { skills: [], platforms: [], total: 0 },
    isEmpty: (value) => value.total === 0 && value.platforms.length === 0,
  })
  const sourceCache = useCachedFetch<SkillSourceRecord[]>({
    ttlMs: 60_000,
    initialValue: [],
    isEmpty: (value) => value.length === 0,
  })
  const marketplaceCache = useCachedFetch<MarketplaceResponse>({
    ttlMs: 60_000,
    initialValue: { items: [], total: 0, page: 1, pageSize: 20, cached: false },
    isEmpty: (value) => value.items.length === 0,
  })

  const routeState = ref<SkillsRouteState>({ ...DEFAULT_ROUTE_STATE })
  const filters = ref<SkillFilters>({
    search: '',
    platform: 'all',
    origin: 'all',
    category: null,
    tags: [],
    source: 'all',
  })
  const selectedSkillId = ref<string | null>(null)
  const selectedInstallationId = ref<string | null>(null)
  const operationLog = ref<SkillLogEntry[]>([])
  const npxStatus = ref<NpxStatus | null>(null)
  const marketplaceLoaded = ref(false)
  const detailCache = shallowRef(new Map<string, SkillRecord>())
  const contentCache = shallowRef(new Map<string, SkillContent>())
  const detailLoading = ref(false)
  const contentLoading = ref(false)
  const mutationLoading = ref(false)

  const inventory = computed(() => inventoryCache.data.value)
  const skills = computed(() => inventory.value.skills)
  const platforms = computed(() => inventory.value.platforms)
  const sources = computed(() => sourceCache.data.value)
  const marketplace = computed(() => marketplaceCache.data.value)
  const facetScopedSkills = computed(() => {
    return skills.value.filter((skill) => {
      if (filters.value.platform !== 'all' && !skill.installations.some((item) => item.platformId === filters.value.platform)) {
        return false
      }
      if (filters.value.origin !== 'all' && skill.origin !== filters.value.origin) {
        return false
      }
      if (filters.value.source !== 'all'
        && skill.sourceRef !== filters.value.source
        && skill.sourceLabel !== filters.value.source) {
        return false
      }
      return true
    })
  })
  const selectedSkill = computed(() => {
    const skillId = selectedSkillId.value || routeState.value.selected
    if (!skillId) return null
    return detailCache.value.get(skillId) ?? skills.value.find((skill) => skill.id === skillId) ?? null
  })
  const selectedInstallation = computed(() => {
    const skill = selectedSkill.value
    if (!skill) return null
    const installationId = selectedInstallationId.value
    return skill.installations.find((installation) => installation.id === installationId)
      ?? skill.installations.find((installation) => installation.isPrimary)
      ?? skill.installations[0]
      ?? null
  })
  const filteredSkills = computed(() => {
    return facetScopedSkills.value.filter((skill) => {
      if (filters.value.category && skill.category !== filters.value.category) {
        return false
      }
      if (filters.value.tags.length > 0 && !filters.value.tags.every((tag) => skill.tags.includes(tag))) {
        return false
      }
      const q = filters.value.search.trim().toLowerCase()
      if (!q) return true
      return skill.name.toLowerCase().includes(q)
        || skill.description?.toLowerCase().includes(q)
        || skill.category?.toLowerCase().includes(q)
        || skill.author?.toLowerCase().includes(q)
        || skill.tags.some((tag) => tag.toLowerCase().includes(q))
    })
  })
  const categories = computed(() => {
    return Array.from(new Set(skills.value.map((skill) => skill.category).filter(Boolean) as string[])).sort()
  })
  const tags = computed(() => {
    return Array.from(new Set(skills.value.flatMap((skill) => skill.tags))).sort()
  })
  const availableCategories = computed(() => {
    return Array.from(new Set(facetScopedSkills.value.map((skill) => skill.category).filter(Boolean) as string[])).sort()
  })
  const availableTags = computed(() => {
    return Array.from(new Set(facetScopedSkills.value.flatMap((skill) => skill.tags))).sort()
  })
  const stats = computed(() => {
    const activePlatforms = platforms.value.filter((platform) => platform.detected && platform.installedCount > 0).length
    return {
      logicalSkills: skills.value.length,
      installations: skills.value.reduce((sum, skill) => sum + skill.installCount, 0),
      sources: sources.value.length,
      activePlatforms,
      marketplace: marketplace.value.total,
      installed: skills.value.length,
      available: marketplace.value.total,
      totalPlatforms: platforms.value.length,
    }
  })

  function pushLog(entry: Omit<SkillLogEntry, 'id' | 'timestamp'>) {
    operationLog.value.unshift({
      id: crypto.randomUUID(),
      timestamp: Date.now(),
      ...entry,
    })
    if (operationLog.value.length > 200) {
      operationLog.value = operationLog.value.slice(0, 200)
    }
  }

  function syncFiltersFromRoute() {
    filters.value = {
      search: routeState.value.q,
      platform: routeState.value.platform,
      origin: routeState.value.origin,
      category: filters.value.category,
      tags: filters.value.tags,
      source: (routeState.value.source ?? 'all') as SkillFilters['source'],
    }
    selectedSkillId.value = routeState.value.selected
  }

  async function loadInventory(force = false) {
    const response = await inventoryCache.fetch(async () => {
      return transformInventory(await getSkillHubUnified())
    }, force)
    response.skills.forEach((skill) => {
      detailCache.value.set(skill.id, skill)
    })
    triggerRef(detailCache)
    return response
  }

  async function loadSources(force = false) {
    return sourceCache.fetch(async () => {
      const response = await skillsSourcesList()
      return asArray(response).map(transformSource)
    }, force)
  }

  async function loadMarketplace(force = false) {
    return marketplaceCache.fetch(async () => {
      const response = routeState.value.q
        ? await searchSkillHubMarketplace(routeState.value.q, routeState.value.page, MARKETPLACE_PAGE_SIZE)
        : await getSkillHubTrending(routeState.value.page, MARKETPLACE_PAGE_SIZE)
      marketplaceLoaded.value = true
      return transformMarketplace(response)
    }, force)
  }

  async function loadNpxStatus(force = false) {
    if (!force && npxStatus.value) return npxStatus.value
    const response = asRecord(await checkNpxAvailability())
    npxStatus.value = {
      available: Boolean(response.available),
      version: typeof response.version === 'string' ? response.version : undefined,
      path: typeof response.path === 'string' ? response.path : undefined,
    }
    return npxStatus.value
  }

  async function ensureDetail(skillId: string, force = false) {
    if (!force && detailCache.value.has(skillId)) {
      return detailCache.value.get(skillId) ?? null
    }
    detailLoading.value = true
    try {
      const detail = transformSkill(await getSkillDetail(skillId))
      detailCache.value.set(skillId, detail)
      triggerRef(detailCache)
      return detail
    } finally {
      detailLoading.value = false
    }
  }

  async function ensureContent(skillId: string, installationId?: string | null, force = false) {
    const cacheKey = `${skillId}:${installationId ?? 'primary'}`
    if (!force && contentCache.value.has(cacheKey)) {
      return contentCache.value.get(cacheKey) ?? null
    }
    contentLoading.value = true
    try {
      const content = transformContent(await getSkillHubSkillContent(skillId, installationId ?? null))
      contentCache.value.set(cacheKey, content)
      triggerRef(contentCache)
      return content
    } finally {
      contentLoading.value = false
    }
  }

  async function refreshAll() {
    await Promise.all([loadInventory(true), loadSources(true)])
  }

  function clearCaches() {
    inventoryCache.invalidate()
    sourceCache.invalidate()
    marketplaceCache.invalidate()
    detailCache.value.clear()
    contentCache.value.clear()
    triggerRef(detailCache)
    triggerRef(contentCache)
  }

  function setSkills(nextSkills: UnifiedSkill[] | SkillRecord[]) {
    const normalized = nextSkills.map((skill) => ('installations' in skill ? skill : fromUnifiedSkill(skill)))
    inventoryCache.setData({
      ...inventory.value,
      skills: normalized,
      total: normalized.length,
    })
  }

  function setPlatforms(nextPlatforms: SkillPlatformSummary[]) {
    const normalized = nextPlatforms.map(transformPlatform)
    inventoryCache.setData({
      ...inventory.value,
      platforms: normalized,
      total: inventory.value.skills.length,
    })
  }

  function setMarketplaceItems(items: MarketplaceResponse['items'], cached: boolean) {
    marketplaceLoaded.value = true
    marketplaceCache.setData({
      items,
      total: items.length,
      page: 1,
      pageSize: marketplace.value.pageSize || 20,
      cached,
    })
  }

  function normalizeFilters(nextFilters: SkillFilters): SkillFilters {
    const normalized: SkillFilters = {
      search: nextFilters.search,
      source: nextFilters.source ?? 'all',
      platform: nextFilters.platform,
      origin: nextFilters.origin ?? 'all',
      category: nextFilters.category,
      tags: [...nextFilters.tags],
    }

    const scopedSkills = normalized.platform === 'all'
      ? skills.value
      : skills.value.filter((skill) => skill.installations.some((item) => item.platformId === normalized.platform))
    const scopedCategories = new Set(scopedSkills.map((skill) => skill.category).filter(Boolean) as string[])
    const scopedTags = new Set(scopedSkills.flatMap((skill) => skill.tags))

    if (normalized.category && !scopedCategories.has(normalized.category)) {
      normalized.category = null
    }
    if (normalized.tags.length > 0) {
      normalized.tags = normalized.tags.filter((tag) => scopedTags.has(tag))
    }

    return normalized
  }

  function setFilters(nextFilters: SkillFilters) {
    filters.value = normalizeFilters(nextFilters)
  }

  function setFilter<K extends keyof SkillFilters>(key: K, value: SkillFilters[K]) {
    setFilters({
      ...filters.value,
      [key]: value,
    })
  }

  function resetFilters() {
    setFilters({
      search: '',
      source: 'all',
      platform: 'all',
      origin: 'all',
      category: null,
      tags: [],
    })
  }

  function setRouteState(nextState: Partial<SkillsRouteState>) {
    routeState.value = {
      ...routeState.value,
      ...nextState,
    }
    syncFiltersFromRoute()
  }

  function selectSkill(skillId: string | null, installationId?: string | null) {
    selectedSkillId.value = skillId
    selectedInstallationId.value = installationId ?? null
    routeState.value.selected = skillId
  }

  async function saveContent(skillId: string, installationId: string, raw: string) {
    mutationLoading.value = true
    pushLog({ action: 'save', target: skillId, status: 'pending' })
    try {
      const saved = transformContent(await saveSkillHubSkillContent(skillId, installationId, raw))
      contentCache.value.set(`${skillId}:${installationId}`, saved)
      triggerRef(contentCache)
      await loadInventory(true)
      await ensureDetail(skillId, true)
      pushLog({ action: 'save', target: skillId, status: 'success' })
      return saved
    } catch (error) {
      pushLog({ action: 'save', target: skillId, status: 'error', detail: error instanceof Error ? error.message : String(error) })
      throw error
    } finally {
      mutationLoading.value = false
    }
  }

  async function runOperation(action: string, target: string, request: Promise<unknown>) {
    mutationLoading.value = true
    pushLog({ action, target, status: 'pending' })
    try {
      const response = transformOperation(await request)
      response.results.forEach((result) => {
        pushLog({
          action,
          target: `${target}:${result.agent}`,
          status: result.ok ? 'success' : 'error',
          detail: result.message,
        })
      })
      await refreshAll()
      return response
    } finally {
      mutationLoading.value = false
    }
  }

  async function install(request: SkillsInstallRequest) {
    return runOperation('install', request.sourceRef, skillsInstall({
      source_kind: request.sourceKind,
      source_ref: request.sourceRef,
      source_skill_id: request.sourceSkillId ?? null,
      target_platforms: request.targetPlatforms,
      force: request.force ?? false,
    }))
  }

  async function syncSkill(request: SkillsSyncRequest) {
    return runOperation('sync', request.skillId, skillsSync({
      skill_id: request.skillId,
      installation_id: request.installationId ?? null,
      target_platforms: request.targetPlatforms,
      force: request.force ?? false,
    }))
  }

  async function removeInstallation(skillId: string, installationId: string) {
    return runOperation('remove-installation', installationId, skillsRemoveInstallation(skillId, installationId))
  }

  async function removeSkillRecord(skillId: string) {
    return runOperation('remove-skill', skillId, skillsRemoveSkill(skillId))
  }

  async function addGitSource(url: string) {
    const source = transformSource(await skillsSourceAddGit(url))
    await loadSources(true)
    return source
  }

  async function addLocalSource(path: string) {
    const source = transformSource(await skillsSourceAddLocal(path))
    await loadSources(true)
    return source
  }

  async function syncSource(sourceId: string) {
    const source = transformSource(await skillsSourceSync(sourceId))
    await refreshAll()
    return source
  }

  async function removeSource(sourceId: string) {
    await skillsSourceRemove(sourceId)
    await loadSources(true)
  }

  return {
    inventory,
    skills,
    platforms,
    sources,
    marketplace,
    routeState,
    filters,
    selectedSkillId,
    selectedInstallationId,
    selectedSkill,
    selectedInstallation,
    operationLog,
    npxStatus,
    detailLoading,
    contentLoading,
    mutationLoading,
    categories,
    tags,
    availableCategories,
    availableTags,
    filteredSkills,
    stats,
    marketplaceLoaded,
    inventoryLoading: inventoryCache.loading,
    sourcesLoading: sourceCache.loading,
    marketplaceLoading: marketplaceCache.loading,
    inventoryError: inventoryCache.error,
    sourcesError: sourceCache.error,
    marketplaceError: marketplaceCache.error,
    loadInventory,
    loadSources,
    loadMarketplace,
    loadNpxStatus,
    ensureDetail,
    ensureContent,
    refreshAll,
    clearCaches,
    setSkills,
    setPlatforms,
    setMarketplaceItems,
    setFilters,
    setFilter,
    resetFilters,
    setRouteState,
    selectSkill,
    saveContent,
    install,
    syncSkill,
    removeInstallation,
    removeSkillRecord,
    addGitSource,
    addLocalSource,
    syncSource,
    removeSource,
  }
})
