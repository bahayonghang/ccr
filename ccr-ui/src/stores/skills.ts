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
  skillsNpxCapabilities,
  skillsFileGet,
  skillsFilesList,
  skillsInstall,
  skillsPrepareInstall,
  skillsOnboardingCandidates,
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
  NpxPlatformSupport,
  OnboardingCandidate,
  Platform,
  SkillContent,
  SkillFileContent,
  SkillFileEntry,
  SkillInstallCommandPreview,
  SkillInstallReviewResponse,
  SkillInstallReviewTarget,
  SkillFilters,
  SkillLogEntry,
  SkillOperationResponse,
  SkillRecord,
  SkillSourceRecord,
  SkillsInstallRequest,
  SkillsNpxCapabilities,
  SkillsInventoryResponse,
  SkillsRouteState,
  SkillsSyncRequest,
  SkillPlatformSummary,
  SkillTargetRecord,
  SkillWorkflowState,
  UnifiedSkill,
} from '@/types/skills'

type UnknownRecord = Record<string, unknown>
const MARKETPLACE_PAGE_SIZE = 20

const DEFAULT_ROUTE_STATE: SkillsRouteState = {
  tab: 'library',
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

function isNonNull<T>(value: T | null): value is T {
  return value !== null
}

function normalizePlatform(value: string): Platform | null {
  const normalized = value.trim().toLowerCase()
  if (!normalized) return null
  if (normalized === 'claude') return 'claude-code'
  return normalized
}

function toStringArray(value: unknown): string[] {
  return asArray(value).filter((item): item is string => typeof item === 'string')
}

function transformInstallation(raw: unknown) {
  const source = asRecord(raw)
  const platformId = normalizePlatform(String(source.platform_id ?? source.platformId ?? 'codex'))
  if (!platformId) return null

  return {
    id: String(source.id ?? ''),
    platformId,
    platformName: String(source.platform_name ?? source.platformName ?? ''),
    installPath: String(source.install_path ?? source.installPath ?? ''),
    installMode: 'copy' as const,
    installedAt: typeof source.installed_at === 'number' ? source.installed_at : undefined,
    isPrimary: Boolean(source.is_primary ?? source.isPrimary),
  }
}

function toTargets(installations: SkillRecord['installations'], source?: UnknownRecord) {
  const rawTargets = asArray(source?.targets)
  if (rawTargets.length > 0) {
    return rawTargets
      .map((raw) => {
        const row = asRecord(raw)
        const platformId = normalizePlatform(String(row.platform_id ?? row.platformId ?? ''))
        if (!platformId) return null
        return {
          id: String(row.id ?? ''),
          platformId,
          platformName: String(row.platform_name ?? row.platformName ?? ''),
          targetPath: String(row.target_path ?? row.targetPath ?? ''),
          syncMode: 'copy' as const,
          status: String(row.status ?? 'unknown') as SkillTargetRecord['status'],
          syncedAt: typeof row.synced_at === 'number' ? row.synced_at : undefined,
          lastError: typeof row.last_error === 'string' ? row.last_error : undefined,
          isPrimary: Boolean(row.is_primary ?? row.isPrimary),
        }
      })
      .filter(isNonNull)
  }

  return installations.filter(isNonNull).map(
    (installation): SkillTargetRecord => ({
      id: installation.id,
      platformId: installation.platformId,
      platformName: installation.platformName,
      targetPath: installation.installPath,
      syncMode: installation.installMode,
      status: 'ok',
      syncedAt: installation.installedAt,
      lastError: undefined,
      isPrimary: installation.isPrimary,
    })
  )
}

function toLifecycle(
  installations: SkillRecord['installations'],
  source: UnknownRecord,
  sourceRef?: string,
  sourceLabel?: string,
  fallbackTargets?: SkillTargetRecord[]
) {
  const rawLifecycle = asRecord(source.lifecycle)
  if (Object.keys(rawLifecycle).length > 0) {
    return {
      sourceRef: typeof rawLifecycle.source_ref === 'string' ? rawLifecycle.source_ref : sourceRef,
      sourceLabel:
        typeof rawLifecycle.source_label === 'string' ? rawLifecycle.source_label : sourceLabel,
      sourceRevision:
        typeof rawLifecycle.source_revision === 'string' ? rawLifecycle.source_revision : undefined,
      contentHash:
        typeof rawLifecycle.content_hash === 'string' ? rawLifecycle.content_hash : undefined,
      lastSyncedAt:
        typeof rawLifecycle.last_synced_at === 'number' ? rawLifecycle.last_synced_at : undefined,
      hasErrors: Boolean(rawLifecycle.has_errors),
      targetCount: Number(rawLifecycle.target_count ?? installations.length),
      healthyTargetCount: Number(rawLifecycle.healthy_target_count ?? installations.length),
    }
  }

  const targets = fallbackTargets?.length ? fallbackTargets : toTargets(installations, source)
  const healthyTargetCount = targets.filter((target) => target.status === 'ok').length
  const syncedAt = targets
    .map((target) => target.syncedAt ?? 0)
    .reduce((latest, value) => Math.max(latest, value), 0)

  return {
    sourceRef,
    sourceLabel,
    sourceRevision: typeof source.source_revision === 'string' ? source.source_revision : undefined,
    contentHash: typeof source.content_hash === 'string' ? source.content_hash : undefined,
    lastSyncedAt: syncedAt > 0 ? syncedAt : undefined,
    hasErrors: healthyTargetCount !== targets.length,
    targetCount: targets.length,
    healthyTargetCount,
  }
}

function transformSkill(raw: unknown): SkillRecord | null {
  const source = asRecord(raw)
  const installations = asArray(source.installations).map(transformInstallation).filter(isNonNull)

  if (installations.length === 0) {
    return null
  }

  const sourceLabel = typeof source.source_label === 'string' ? source.source_label : undefined
  const sourceRef = typeof source.source_ref === 'string' ? source.source_ref : undefined

  return {
    id: String(source.id ?? ''),
    name: String(source.name ?? ''),
    description: typeof source.description === 'string' ? source.description : undefined,
    category: typeof source.category === 'string' ? source.category : undefined,
    tags: toStringArray(source.tags),
    version: typeof source.version === 'string' ? source.version : undefined,
    author: typeof source.author === 'string' ? source.author : undefined,
    origin: String(source.origin ?? 'unknown') as SkillRecord['origin'],
    sourceLabel,
    sourceRef,
    installCount: installations.length,
    installations,
    targets: toTargets(installations, source),
    lifecycle: toLifecycle(installations, source, sourceRef, sourceLabel),
    editableInstallations: toStringArray(
      source.editable_installations ?? source.editableInstallations
    ),
  }
}

function transformPlatform(raw: unknown): SkillPlatformSummary | null {
  const source = asRecord(raw)
  const id = normalizePlatform(String(source.id ?? 'codex'))
  if (!id) return null

  return {
    id,
    displayName: String(source.display_name ?? source.displayName ?? ''),
    globalSkillsDir: String(source.global_skills_dir ?? source.globalSkillsDir ?? ''),
    detected: Boolean(source.detected),
    installedCount: Number(source.installed_count ?? source.installedCount ?? 0),
    sharedDirGroup:
      typeof source.shared_dir_group === 'string'
        ? source.shared_dir_group
        : typeof source.sharedDirGroup === 'string'
          ? source.sharedDirGroup
          : undefined,
    installStrategy:
      typeof source.install_strategy === 'string'
        ? (source.install_strategy as SkillPlatformSummary['installStrategy'])
        : typeof source.installStrategy === 'string'
          ? (source.installStrategy as SkillPlatformSummary['installStrategy'])
          : undefined,
    npxAgentKey:
      typeof source.npx_agent_key === 'string'
        ? source.npx_agent_key
        : typeof source.npxAgentKey === 'string'
          ? source.npxAgentKey
          : undefined,
    category: typeof source.category === 'string' ? source.category : undefined,
    capabilities: toStringArray(source.capabilities),
    sortOrder: Number(source.sort_order ?? source.sortOrder ?? 0),
  }
}

function transformInventory(raw: unknown): SkillsInventoryResponse {
  const source = asRecord(raw)
  const skills = asArray(source.skills).map(transformSkill).filter(isNonNull)
  const platforms = asArray(source.platforms)
    .map(transformPlatform)
    .filter(isNonNull)
    .sort((left, right) => (left.sortOrder ?? 0) - (right.sortOrder ?? 0))

  return {
    skills,
    platforms,
    total: skills.length,
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
        skillsShUrl: String(
          row.skills_sh_url ?? row.skillsShUrl ?? `https://skills.sh/${String(row.package ?? '')}`
        ),
        description: typeof row.description === 'string' ? row.description : undefined,
        authorAvatar:
          typeof row.author_avatar === 'string'
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

function transformNpxCapabilities(raw: unknown): SkillsNpxCapabilities {
  const source = asRecord(raw)
  return {
    available: Boolean(source.available ?? asRecord(source.status).available),
    version:
      typeof source.version === 'string'
        ? source.version
        : typeof asRecord(source.status).version === 'string'
          ? String(asRecord(source.status).version)
          : undefined,
    path:
      typeof source.path === 'string'
        ? source.path
        : typeof asRecord(source.status).path === 'string'
          ? String(asRecord(source.status).path)
          : undefined,
    packageManager: String(source.package_manager ?? source.packageManager ?? 'npx'),
    supportedFlags: toStringArray(source.supported_flags ?? source.supportedFlags),
    supportedPlatforms: asArray(source.supported_platforms ?? source.supportedPlatforms).map((row) => {
      const item = asRecord(row)
      return {
        platformId: String(item.platform_id ?? item.platformId ?? ''),
        platformName: String(item.platform_name ?? item.platformName ?? ''),
        supported: Boolean(item.supported),
        agentKey:
          typeof item.agent_key === 'string'
            ? item.agent_key
            : typeof item.agentKey === 'string'
              ? item.agentKey
              : undefined,
        reason: typeof item.reason === 'string' ? item.reason : undefined,
      } satisfies NpxPlatformSupport
    }),
  }
}

function transformInstallReview(raw: unknown): SkillInstallReviewResponse {
  const source = asRecord(raw)
  const reviewSource = asRecord(source.source)
  return {
    source: {
      sourceKind: String(reviewSource.source_kind ?? reviewSource.sourceKind ?? ''),
      sourceRef: String(reviewSource.source_ref ?? reviewSource.sourceRef ?? ''),
      sourceSkillId:
        typeof reviewSource.source_skill_id === 'string'
          ? reviewSource.source_skill_id
          : typeof reviewSource.sourceSkillId === 'string'
            ? reviewSource.sourceSkillId
            : undefined,
      resolvedName: String(reviewSource.resolved_name ?? reviewSource.resolvedName ?? ''),
      resolvedDirName: String(reviewSource.resolved_dir_name ?? reviewSource.resolvedDirName ?? ''),
      origin: String(reviewSource.origin ?? 'unknown') as SkillInstallReviewResponse['source']['origin'],
      description:
        typeof reviewSource.description === 'string' ? reviewSource.description : undefined,
    },
    targets: asArray(source.targets).map((row) => {
      const item = asRecord(row)
      return {
        platformId: String(item.platform_id ?? item.platformId ?? ''),
        platformName: String(item.platform_name ?? item.platformName ?? ''),
        detected: Boolean(item.detected),
        targetPath: String(item.target_path ?? item.targetPath ?? ''),
        sharedDirGroup:
          typeof item.shared_dir_group === 'string'
            ? item.shared_dir_group
            : typeof item.sharedDirGroup === 'string'
              ? item.sharedDirGroup
              : undefined,
        installStrategy:
          typeof item.install_strategy === 'string'
            ? (item.install_strategy as SkillInstallReviewTarget['installStrategy'])
            : typeof item.installStrategy === 'string'
              ? (item.installStrategy as SkillInstallReviewTarget['installStrategy'])
              : undefined,
        directNpxSupported: Boolean(item.direct_npx_supported ?? item.directNpxSupported),
        npxAgentKey:
          typeof item.npx_agent_key === 'string'
            ? item.npx_agent_key
            : typeof item.npxAgentKey === 'string'
              ? item.npxAgentKey
              : undefined,
      } satisfies SkillInstallReviewTarget
    }),
    warnings: toStringArray(source.warnings),
    commandPreviews: asArray(source.command_previews ?? source.commandPreviews).map((row) => {
      const item = asRecord(row)
      return {
        kind: String(item.kind ?? ''),
        label: String(item.label ?? ''),
        command: String(item.command ?? ''),
        platforms: toStringArray(item.platforms),
      } satisfies SkillInstallCommandPreview
    }),
    npx: source.npx ? transformNpxCapabilities(source.npx) : undefined,
  }
}

function normalizeSkillRecord(skill: SkillRecord): SkillRecord {
  const targets = skill.targets?.length ? skill.targets : toTargets(skill.installations)
  const lifecycle = skill.lifecycle?.targetCount
    ? skill.lifecycle
    : toLifecycle(
        skill.installations,
        {
          source_ref: skill.sourceRef,
          source_label: skill.sourceLabel,
        },
        skill.sourceRef,
        skill.sourceLabel,
        targets
      )

  return {
    ...skill,
    targets,
    lifecycle,
  }
}

function fromUnifiedSkill(skill: UnifiedSkill): SkillRecord {
  const installations = [
    {
      id: `${skill.platform}:${skill.skillDir}`,
      platformId: skill.platform,
      platformName: skill.platformName,
      installPath: skill.skillDir,
      installMode: 'copy' as const,
      installedAt: skill.installDate,
      isPrimary: true,
    },
  ]

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
    installations,
    targets: toTargets(installations),
    lifecycle: {
      sourceRef: skill.sourceUrl,
      sourceLabel: undefined,
      sourceRevision: skill.commitHash,
      contentHash: undefined,
      lastSyncedAt: skill.installDate,
      hasErrors: false,
      targetCount: 1,
      healthyTargetCount: 1,
    },
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
  const workflowState = ref<SkillWorkflowState>({
    action: 'idle',
    target: '',
    status: 'idle',
  })
  const onboardingCandidates = ref<OnboardingCandidate[]>([])
  const filesCache = shallowRef(new Map<string, SkillFileEntry[]>())
  const fileContentCache = shallowRef(new Map<string, SkillFileContent>())
  const npxStatus = ref<NpxStatus | null>(null)
  const npxCapabilities = ref<SkillsNpxCapabilities | null>(null)
  const installReview = ref<SkillInstallReviewResponse | null>(null)
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
      if (
        filters.value.platform !== 'all' &&
        !skill.installations.some((item) => item.platformId === filters.value.platform)
      ) {
        return false
      }
      if (filters.value.origin !== 'all' && skill.origin !== filters.value.origin) {
        return false
      }
      if (
        filters.value.source !== 'all' &&
        skill.sourceRef !== filters.value.source &&
        skill.sourceLabel !== filters.value.source
      ) {
        return false
      }
      return true
    })
  })
  const selectedSkill = computed(() => {
    const skillId = selectedSkillId.value || routeState.value.selected
    if (!skillId) return null
    return (
      detailCache.value.get(skillId) ?? skills.value.find((skill) => skill.id === skillId) ?? null
    )
  })
  const selectedInstallation = computed(() => {
    const skill = selectedSkill.value
    if (!skill) return null
    const installationId = selectedInstallationId.value
    return (
      skill.installations.find((installation) => installation.id === installationId) ??
      skill.installations.find((installation) => installation.isPrimary) ??
      skill.installations[0] ??
      null
    )
  })
  const filteredSkills = computed(() => {
    return facetScopedSkills.value.filter((skill) => {
      if (filters.value.category && skill.category !== filters.value.category) {
        return false
      }
      if (
        filters.value.tags.length > 0 &&
        !filters.value.tags.every((tag) => skill.tags.includes(tag))
      ) {
        return false
      }
      const q = filters.value.search.trim().toLowerCase()
      if (!q) return true
      return (
        skill.name.toLowerCase().includes(q) ||
        skill.description?.toLowerCase().includes(q) ||
        skill.category?.toLowerCase().includes(q) ||
        skill.author?.toLowerCase().includes(q) ||
        skill.tags.some((tag) => tag.toLowerCase().includes(q))
      )
    })
  })
  const categories = computed(() => {
    return Array.from(
      new Set(skills.value.map((skill) => skill.category).filter(Boolean) as string[])
    ).sort()
  })
  const tags = computed(() => {
    return Array.from(new Set(skills.value.flatMap((skill) => skill.tags))).sort()
  })
  const availableCategories = computed(() => {
    return Array.from(
      new Set(facetScopedSkills.value.map((skill) => skill.category).filter(Boolean) as string[])
    ).sort()
  })
  const availableTags = computed(() => {
    return Array.from(new Set(facetScopedSkills.value.flatMap((skill) => skill.tags))).sort()
  })
  const stats = computed(() => {
    const activePlatforms = platforms.value.filter(
      (platform) => platform.detected && platform.installedCount > 0
    ).length
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
        ? await searchSkillHubMarketplace(
            routeState.value.q,
            routeState.value.page,
            MARKETPLACE_PAGE_SIZE
          )
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

  async function loadNpxCapabilities(force = false) {
    if (!force && npxCapabilities.value) return npxCapabilities.value
    npxCapabilities.value = transformNpxCapabilities(await skillsNpxCapabilities())
    return npxCapabilities.value
  }

  async function prepareInstall(request: SkillsInstallRequest) {
    installReview.value = transformInstallReview(
      await skillsPrepareInstall({
        source_kind: request.sourceKind,
        source_ref: request.sourceRef,
        source_skill_id: request.sourceSkillId ?? null,
        selected_skills: request.selectedSkills ?? [],
        target_platforms: request.targetPlatforms,
        force: request.force ?? false,
        scope: request.scope ?? 'global',
        copy_mode: request.copyMode ?? true,
        all_mode: request.allMode ?? false,
      })
    )
    return installReview.value
  }

  async function ensureDetail(skillId: string, force = false) {
    if (!force && detailCache.value.has(skillId)) {
      return detailCache.value.get(skillId) ?? null
    }
    detailLoading.value = true
    try {
      const detail = transformSkill(await getSkillDetail(skillId))
      if (!detail) {
        return null
      }
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
      const content = transformContent(
        await getSkillHubSkillContent(skillId, installationId ?? null)
      )
      contentCache.value.set(cacheKey, content)
      triggerRef(contentCache)
      return content
    } finally {
      contentLoading.value = false
    }
  }

  async function ensureFiles(skillId: string, installationId?: string | null, force = false) {
    const cacheKey = `${skillId}:${installationId ?? 'primary'}`
    if (!force && filesCache.value.has(cacheKey)) {
      return filesCache.value.get(cacheKey) ?? []
    }

    const files = asArray(await skillsFilesList(skillId, installationId ?? null)).map((raw) => {
      const row = asRecord(raw)
      return {
        path: String(row.path ?? ''),
        size: Number(row.size ?? 0),
        isDir: Boolean(row.is_dir ?? row.isDir),
      } satisfies SkillFileEntry
    })

    filesCache.value.set(cacheKey, files)
    triggerRef(filesCache)
    return files
  }

  async function ensureFileContent(
    skillId: string,
    path: string,
    installationId?: string | null,
    force = false
  ) {
    const cacheKey = `${skillId}:${installationId ?? 'primary'}:${path}`
    if (!force && fileContentCache.value.has(cacheKey)) {
      return fileContentCache.value.get(cacheKey) ?? null
    }

    const row = asRecord(await skillsFileGet(skillId, path, installationId ?? null))
    const content = {
      skillId: String(row.skill_id ?? row.skillId ?? skillId),
      installationId: String(
        row.installation_id ?? row.installationId ?? installationId ?? 'primary'
      ),
      path: String(row.path ?? path),
      content: String(row.content ?? ''),
    } satisfies SkillFileContent

    fileContentCache.value.set(cacheKey, content)
    triggerRef(fileContentCache)
    return content
  }

  async function loadOnboardingCandidates(force = false) {
    if (!force && onboardingCandidates.value.length > 0) {
      return onboardingCandidates.value
    }

    onboardingCandidates.value = asArray(await skillsOnboardingCandidates()).map((raw) => {
      const row = asRecord(raw)
      return {
        skillId: String(row.skill_id ?? row.skillId ?? ''),
        name: String(row.name ?? ''),
        platformIds: toStringArray(row.platform_ids ?? row.platformIds)
          .map((value) => normalizePlatform(value))
          .filter(isNonNull),
        installationIds: toStringArray(row.installation_ids ?? row.installationIds),
        installationPaths: toStringArray(row.installation_paths ?? row.installationPaths),
        reason: String(row.reason ?? 'unknown_origin') as OnboardingCandidate['reason'],
      }
    })
    return onboardingCandidates.value
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
    const normalized = nextSkills.map((skill) => {
      if ('installations' in skill) {
        return normalizeSkillRecord(skill)
      }
      return fromUnifiedSkill(skill)
    })
    inventoryCache.setData({
      ...inventory.value,
      skills: normalized,
      total: normalized.length,
    })
  }

  function setPlatforms(nextPlatforms: SkillPlatformSummary[]) {
    const normalized = nextPlatforms.map(transformPlatform).filter(isNonNull)
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

    const scopedSkills =
      normalized.platform === 'all'
        ? skills.value
        : skills.value.filter((skill) =>
            skill.installations.some((item) => item.platformId === normalized.platform)
          )
    const scopedCategories = new Set(
      scopedSkills.map((skill) => skill.category).filter(Boolean) as string[]
    )
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
    workflowState.value = {
      action: 'save',
      target: skillId,
      status: 'pending',
    }
    pushLog({ action: 'save', target: skillId, status: 'pending' })
    try {
      const saved = transformContent(await saveSkillHubSkillContent(skillId, installationId, raw))
      contentCache.value.set(`${skillId}:${installationId}`, saved)
      triggerRef(contentCache)
      detailCache.value.delete(skillId)
      triggerRef(detailCache)
      await ensureDetail(skillId, true)
      workflowState.value = {
        action: 'save',
        target: skillId,
        status: 'success',
      }
      pushLog({ action: 'save', target: skillId, status: 'success' })
      return saved
    } catch (error) {
      workflowState.value = {
        action: 'save',
        target: skillId,
        status: 'error',
        detail: error instanceof Error ? error.message : String(error),
      }
      pushLog({
        action: 'save',
        target: skillId,
        status: 'error',
        detail: error instanceof Error ? error.message : String(error),
      })
      throw error
    } finally {
      mutationLoading.value = false
    }
  }

  async function runOperation(
    action: string,
    target: string,
    request: Promise<unknown>,
    targetPlatforms?: Platform[]
  ) {
    mutationLoading.value = true
    workflowState.value = {
      action,
      target,
      status: 'pending',
      targetPlatforms,
    }
    pushLog({ action, target, status: 'pending' })
    try {
      const response = transformOperation(await request)
      workflowState.value = {
        action,
        target,
        status: response.results.every((result) => result.ok) ? 'success' : 'error',
        targetPlatforms,
        results: response.results,
      }
      response.results.forEach((result) => {
        pushLog({
          action,
          target: `${target}:${result.agent}`,
          status: result.ok ? 'success' : 'error',
          detail: result.message,
        })
      })

      if (
        action === 'install' ||
        action === 'sync' ||
        action === 'remove-installation' ||
        action === 'remove-skill'
      ) {
        inventoryCache.invalidate()
        detailCache.value.delete(target)
        triggerRef(detailCache)
        await loadInventory(true)
      } else if (action === 'source-sync') {
        sourceCache.invalidate()
        inventoryCache.invalidate()
        await Promise.all([loadSources(true), loadInventory(true)])
      }

      return response
    } catch (error) {
      workflowState.value = {
        action,
        target,
        status: 'error',
        targetPlatforms,
        detail: error instanceof Error ? error.message : String(error),
      }
      throw error
    } finally {
      mutationLoading.value = false
    }
  }

  async function install(request: SkillsInstallRequest) {
    return runOperation(
      'install',
      request.sourceRef,
      skillsInstall({
        source_kind: request.sourceKind,
        source_ref: request.sourceRef,
        source_skill_id: request.sourceSkillId ?? null,
        selected_skills: request.selectedSkills ?? [],
        target_platforms: request.targetPlatforms,
        force: request.force ?? false,
        scope: request.scope ?? 'global',
        copy_mode: request.copyMode ?? true,
        all_mode: request.allMode ?? false,
      }),
      request.targetPlatforms
    )
  }

  async function syncSkill(request: SkillsSyncRequest) {
    return runOperation(
      'sync',
      request.skillId,
      skillsSync({
        skill_id: request.skillId,
        installation_id: request.installationId ?? null,
        target_platforms: request.targetPlatforms,
        force: request.force ?? false,
      }),
      request.targetPlatforms
    )
  }

  async function removeInstallation(skillId: string, installationId: string) {
    return runOperation(
      'remove-installation',
      installationId,
      skillsRemoveInstallation(skillId, installationId)
    )
  }

  async function removeSkillRecord(skillId: string) {
    return runOperation('remove-skill', skillId, skillsRemoveSkill(skillId))
  }

  async function addGitSource(url: string) {
    mutationLoading.value = true
    workflowState.value = {
      action: 'source-add-git',
      target: url,
      status: 'pending',
    }
    try {
      const source = transformSource(await skillsSourceAddGit(url))
      sourceCache.invalidate()
      await loadSources(true)
      workflowState.value = {
        action: 'source-add-git',
        target: url,
        status: 'success',
      }
      return source
    } catch (error) {
      workflowState.value = {
        action: 'source-add-git',
        target: url,
        status: 'error',
        detail: error instanceof Error ? error.message : String(error),
      }
      throw error
    } finally {
      mutationLoading.value = false
    }
  }

  async function addLocalSource(path: string) {
    mutationLoading.value = true
    workflowState.value = {
      action: 'source-add-local',
      target: path,
      status: 'pending',
    }
    try {
      const source = transformSource(await skillsSourceAddLocal(path))
      sourceCache.invalidate()
      await loadSources(true)
      workflowState.value = {
        action: 'source-add-local',
        target: path,
        status: 'success',
      }
      return source
    } catch (error) {
      workflowState.value = {
        action: 'source-add-local',
        target: path,
        status: 'error',
        detail: error instanceof Error ? error.message : String(error),
      }
      throw error
    } finally {
      mutationLoading.value = false
    }
  }

  async function syncSource(sourceId: string) {
    const source = transformSource(await skillsSourceSync(sourceId))
    workflowState.value = {
      action: 'source-sync',
      target: sourceId,
      status: 'success',
    }
    sourceCache.invalidate()
    inventoryCache.invalidate()
    await Promise.all([loadSources(true), loadInventory(true)])
    return source
  }

  async function removeSource(sourceId: string) {
    mutationLoading.value = true
    workflowState.value = {
      action: 'source-remove',
      target: sourceId,
      status: 'pending',
    }
    try {
      await skillsSourceRemove(sourceId)
      sourceCache.invalidate()
      inventoryCache.invalidate()
      await Promise.all([loadSources(true), loadInventory(true)])
      workflowState.value = {
        action: 'source-remove',
        target: sourceId,
        status: 'success',
      }
    } catch (error) {
      workflowState.value = {
        action: 'source-remove',
        target: sourceId,
        status: 'error',
        detail: error instanceof Error ? error.message : String(error),
      }
      throw error
    } finally {
      mutationLoading.value = false
    }
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
    onboardingCandidates,
    workflowState,
    npxStatus,
    npxCapabilities,
    installReview,
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
    loadNpxCapabilities,
    prepareInstall,
    ensureDetail,
    ensureContent,
    ensureFiles,
    ensureFileContent,
    loadOnboardingCandidates,
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
