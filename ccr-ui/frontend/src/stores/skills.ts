/**
 * Unified Skills Store
 * Pinia 状态管理 for Skills Hub
 */
import { defineStore } from 'pinia'
import { ref, computed, watch, shallowRef, triggerRef } from 'vue'
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
  // State
  const skills = ref<UnifiedSkill[]>([])
  const platforms = ref<PlatformSummary[]>([])
  const marketplaceItems = ref<MarketplaceItem[]>([])
  const isLoading = ref(false)
  const isMarketplaceLoading = ref(false)
  const error = ref<string | null>(null)
  const marketplaceError = ref<string | null>(null)
  const marketplaceCached = ref(false)

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

  // 防抖搜索值（250ms 延迟）
  const debouncedSearch = ref('')
  let searchTimer: ReturnType<typeof setTimeout> | null = null
  watch(() => filters.value.search, (val) => {
    if (searchTimer) clearTimeout(searchTimer)
    searchTimer = setTimeout(() => { debouncedSearch.value = val }, 250)
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

    // Filter by search query (debounced)
    if (debouncedSearch.value) {
      const query = debouncedSearch.value.toLowerCase()
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

  // Actions
  function setFilter<K extends keyof SkillFilters>(key: K, value: SkillFilters[K]) {
    filters.value[key] = value

    // When platform changes, clear category and tags that may not exist in the new platform
    if (key === 'platform') {
      // Clear category if it no longer exists in the new platform's categories
      if (filters.value.category && !availableCategories.value.includes(filters.value.category)) {
        filters.value.category = null
      }
      // Clear tags that no longer exist in the new platform's tags
      if (filters.value.tags.length > 0) {
        filters.value.tags = filters.value.tags.filter(t => availableTags.value.includes(t))
      }
    }
  }

  function resetFilters() {
    filters.value = {
      search: '',
      source: 'all',
      category: null,
      tags: [],
      platform: 'all'
    }
  }

  function setActiveTab(tab: ContentTab) {
    activeTab.value = tab
  }

  function setSkills(newSkills: UnifiedSkill[]) {
    skills.value = newSkills
  }

  function setPlatforms(newPlatforms: PlatformSummary[]) {
    platforms.value = newPlatforms
  }

  function setMarketplaceItems(items: MarketplaceItem[], cached: boolean) {
    marketplaceItems.value = items
    marketplaceCached.value = cached
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

    // Computed
    filteredSkills,
    availableCategories,
    availableTags,
    stats,
    skillsByPlatform,

    // Actions
    setFilter,
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
