/**
 * Unified Skills Store
 * Pinia 状态管理 for Skills Hub
 */
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type {
  SkillFilters,
  UnifiedSkill,
  PlatformSummary,
  MarketplaceItem,
  ContentTab,
  SkillsStats
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
    if (filters.value.search) {
      const query = filters.value.search.toLowerCase()
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
    setMarketplaceError
  }
})
