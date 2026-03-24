/**
 * Unified Skills Store
 * Pinia 状态管理 for Skills Hub
 */
import { defineStore } from 'pinia'
import { ref, computed, shallowRef, triggerRef } from 'vue'
import { useCachedFetch } from '@/composables/useCachedFetch'
import type {
  SkillFilters,
  UnifiedSkill,
  PlatformSummary,
  MarketplaceItem,
  ContentTab,
  SkillsStats,
  NpxStatus,
  InstallProgress
} from '@/types/skills'

export const useSkillsStore = defineStore('skills', () => {
  const skillsCache = useCachedFetch<UnifiedSkill[]>({
    ttlMs: 5 * 60 * 1000,
    initialValue: [],
    isEmpty: (value) => value.length === 0,
  })
  const marketplaceCache = useCachedFetch<MarketplaceItem[]>({
    ttlMs: 5 * 60 * 1000,
    initialValue: [],
    isEmpty: (value) => value.length === 0,
  })

  // State
  const skills = skillsCache.data
  const platforms = ref<PlatformSummary[]>([])
  const marketplaceItems = marketplaceCache.data
  const isLoading = ref(false)
  const isMarketplaceLoading = ref(false)
  const error = ref<string | null>(null)
  const marketplaceError = ref<string | null>(null)
  const marketplaceCached = ref(false)
  const marketplaceLoaded = ref(false)

  // 缓存时间戳（0 表示从未加载）
  const skillsLastFetchedAt = skillsCache.lastFetchedAt
  const marketplaceLastFetchedAt = marketplaceCache.lastFetchedAt

  // === 新增状态 ===
  // 安装状态
  const isInstalling = ref(false)
  const installProgress = ref<InstallProgress | null>(null)

  // 批量模式
  const batchMode = ref(false)
  const batchSelection = shallowRef<Set<string>>(new Set())

  // npx 状态
  const npxStatus = ref<NpxStatus | null>(null)

  // 市场分页
  const marketplacePage = ref(1)
  const marketplaceTotal = ref(0)
  const marketplacePageSize = ref(20)

  // Filters
  const filters = ref<SkillFilters>({
    search: '',
    source: 'all',
    category: null,
    tags: [],
    platform: 'all'
  })

  // Active content tab
  const activeTab = ref<ContentTab>('installed')

  // Computed: filtered skills
  const filteredSkills = computed(() => {
    let result = [...skills.value]

    // Filter by platform
    if (filters.value.platform !== 'all') {
      result = result.filter(s => s.platform === filters.value.platform)
    }

    // Filter by search query
    if (filters.value.search.trim()) {
      const query = filters.value.search.trim().toLowerCase()
      result = result.filter(s =>
        s.name.toLowerCase().includes(query) ||
        s.description?.toLowerCase().includes(query) ||
        s.tags.some(t => t.toLowerCase().includes(query)) ||
        s.category?.toLowerCase().includes(query)
      )
    }

    // Filter by category
    if (filters.value.category) {
      result = result.filter(s => s.category === filters.value.category)
    }

    // Filter by tags
    if (filters.value.tags.length > 0) {
      result = result.filter(s =>
        filters.value.tags.some(tag => s.tags.includes(tag))
      )
    }

    return result
  })

  // Computed: available categories (filtered by current platform)
  const availableCategories = computed(() => {
    const categories = new Set<string>()
    // Filter by platform first
    const baseSkills = filters.value.platform === 'all'
      ? skills.value
      : skills.value.filter(s => s.platform === filters.value.platform)

    baseSkills.forEach(s => {
      if (s.category) categories.add(s.category)
    })
    return Array.from(categories).sort()
  })

  // Computed: available tags (filtered by current platform)
  const availableTags = computed(() => {
    const tags = new Set<string>()
    // Filter by platform first
    const baseSkills = filters.value.platform === 'all'
      ? skills.value
      : skills.value.filter(s => s.platform === filters.value.platform)

    baseSkills.forEach(s => {
      s.tags.forEach(t => tags.add(t))
    })
    return Array.from(tags).sort()
  })

  // Computed: stats
  const stats = computed<SkillsStats>(() => {
    const activePlatforms = platforms.value.filter(p => p.detected && p.installed_count > 0).length
    return {
      installed: skills.value.length,
      available: marketplaceItems.value.length,
      activePlatforms,
      totalPlatforms: platforms.value.length
    }
  })

  // Computed: skills 缓存是否有效（5分钟 TTL）
  const isSkillsCacheValid = skillsCache.isCacheValid

  // Computed: marketplace 缓存是否有效（5分钟 TTL）
  const isMarketplaceCacheValid = marketplaceCache.isCacheValid

  // Computed: skills grouped by platform
  const skillsByPlatform = computed(() => {
    const grouped = new Map<string, UnifiedSkill[]>()
    skills.value.forEach(skill => {
      const existing = grouped.get(skill.platform) || []
      existing.push(skill)
      grouped.set(skill.platform, existing)
    })
    return grouped
  })

  function getFacetOptionsForPlatform(platform: SkillFilters['platform']) {
    const baseSkills = platform === 'all'
      ? skills.value
      : skills.value.filter((skill) => skill.platform === platform)

    const categories = new Set<string>()
    const tags = new Set<string>()

    for (const skill of baseSkills) {
      if (skill.category) categories.add(skill.category)
      for (const tag of skill.tags) tags.add(tag)
    }

    return {
      categories,
      tags,
    }
  }

  function normalizeFilters(nextFilters: SkillFilters): SkillFilters {
    const normalized: SkillFilters = {
      search: nextFilters.search,
      source: nextFilters.source,
      category: nextFilters.category,
      tags: [...nextFilters.tags],
      platform: nextFilters.platform,
    }

    const { categories, tags } = getFacetOptionsForPlatform(normalized.platform)

    if (normalized.category && !categories.has(normalized.category)) {
      normalized.category = null
    }

    if (normalized.tags.length > 0) {
      normalized.tags = normalized.tags.filter((tag) => tags.has(tag))
    }

    return normalized
  }

  // Actions
  function setFilter<K extends keyof SkillFilters>(key: K, value: SkillFilters[K]) {
    setFilters({
      ...filters.value,
      [key]: value
    })
  }

  function setFilters(nextFilters: SkillFilters) {
    filters.value = normalizeFilters(nextFilters)
  }

  function resetFilters() {
    setFilters({
      search: '',
      source: 'all',
      category: null,
      tags: [],
      platform: 'all'
    })
  }

  function setActiveTab(tab: ContentTab) {
    activeTab.value = tab
  }

  function setSkills(newSkills: UnifiedSkill[]) {
    skillsCache.setData(newSkills)
  }

  function setPlatforms(newPlatforms: PlatformSummary[]) {
    platforms.value = newPlatforms
  }

  function setMarketplaceItems(items: MarketplaceItem[], cached: boolean) {
    marketplaceCache.setData(items)
    marketplaceCached.value = cached
    marketplaceLoaded.value = true
  }

  function setLoading(loading: boolean) {
    isLoading.value = loading
  }

  function setMarketplaceLoading(loading: boolean) {
    isMarketplaceLoading.value = loading
  }

  function setError(err: string | null) {
    error.value = err
  }

  function setMarketplaceError(err: string | null) {
    marketplaceError.value = err
  }

  // === 新增 Actions ===
  function toggleBatchMode() {
    batchMode.value = !batchMode.value
    if (!batchMode.value) {
      batchSelection.value.clear()
      triggerRef(batchSelection)
    }
  }

  function toggleBatchSelection(packageId: string) {
    if (batchSelection.value.has(packageId)) {
      batchSelection.value.delete(packageId)
    } else {
      batchSelection.value.add(packageId)
    }
    triggerRef(batchSelection)
  }

  function selectAllBatch(packageIds: string[]) {
    batchSelection.value = new Set(packageIds)
    triggerRef(batchSelection)
  }

  function clearBatchSelection() {
    batchSelection.value.clear()
    triggerRef(batchSelection)
  }

  function setMarketplacePage(page: number) {
    marketplacePage.value = page
  }

  function setMarketplaceTotal(total: number) {
    marketplaceTotal.value = total
  }

  function setNpxStatus(status: NpxStatus | null) {
    npxStatus.value = status
  }

  function setInstalling(installing: boolean) {
    isInstalling.value = installing
  }

  function setInstallProgress(progress: InstallProgress | null) {
    installProgress.value = progress
  }

  return {
    // State
    skills,
    platforms,
    marketplaceItems,
    isLoading,
    isMarketplaceLoading,
    error,
    marketplaceError,
    marketplaceCached,
    marketplaceLoaded,
    filters,
    activeTab,
    // 新增状态
    isInstalling,
    installProgress,
    batchMode,
    batchSelection,
    npxStatus,
    marketplacePage,
    marketplaceTotal,
    marketplacePageSize,

    // 缓存时间戳与有效性
    skillsLastFetchedAt,
    marketplaceLastFetchedAt,
    isSkillsCacheValid,
    isMarketplaceCacheValid,

    // Computed
    filteredSkills,
    availableCategories,
    availableTags,
    stats,
    skillsByPlatform,

    // Actions
    setFilter,
    setFilters,
    resetFilters,
    setActiveTab,
    setSkills,
    setPlatforms,
    setMarketplaceItems,
    setLoading,
    setMarketplaceLoading,
    setError,
    setMarketplaceError,
    // 新增 Actions
    toggleBatchMode,
    toggleBatchSelection,
    selectAllBatch,
    clearBatchSelection,
    setMarketplacePage,
    setMarketplaceTotal,
    setNpxStatus,
    setInstalling,
    setInstallProgress
  }
})
