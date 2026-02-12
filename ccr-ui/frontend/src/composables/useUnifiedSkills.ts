/**
 * Unified Skills Composable
 * 统一 Skills 管理组合式 API
 */
import { storeToRefs } from 'pinia'
import { useSkillsStore } from '@/stores/skills'
import api from '@/api/core'
import type {
  Platform,
  UnifiedSkill,
  UnifiedSkillsResponse,
  InstallRequest,
  RemoveRequest,
  OperationResponse,
  PlatformSummary,
  SkillContent
} from '@/types/skills'

// API endpoints
const SKILL_HUB_BASE = '/skill_hub'

// Backend response types (snake_case)
interface BackendMarketplaceItem {
  package: string
  owner: string
  repo: string
  skill?: string
  skills_sh_url: string
}

interface BackendMarketplaceResponse {
  items: BackendMarketplaceItem[]
  total: number
  cached: boolean
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
    skillsByPlatform
  } = storeToRefs(store)

  // Transform backend response to frontend format
  function transformSkill(skill: any): UnifiedSkill {
    return {
      name: skill.name,
      description: skill.description,
      skillDir: skill.skill_dir,
      platform: skill.platform as Platform,
      platformName: skill.platform_name,
      category: skill.category,
      tags: skill.tags || [],
      // Metadata fields (snake_case → camelCase)
      version: skill.version,
      author: skill.author,
      source: skill.source,
      sourceUrl: skill.source_url,
      installDate: skill.install_date,
      commitHash: skill.commit_hash,
    }
  }

  function transformPlatform(platform: any): PlatformSummary {
    return {
      id: platform.id,
      display_name: platform.display_name,
      global_skills_dir: platform.global_skills_dir,
      detected: platform.detected,
      installed_count: platform.installed_count
    }
  }

  // Fetch all skills from all platforms
  async function fetchAllSkills() {
    store.setLoading(true)
    store.setError(null)

    try {
      const response = await api.get<{ data: UnifiedSkillsResponse }>(`${SKILL_HUB_BASE}/unified`)
      const data = response.data.data

      store.setSkills(data.skills.map(transformSkill))
      store.setPlatforms(data.platforms.map(transformPlatform))
    } catch (err: any) {
      store.setError(err.message || 'Failed to fetch skills')
      console.error('Failed to fetch unified skills:', err)
    } finally {
      store.setLoading(false)
    }
  }

  // Fetch skills for a specific platform
  async function fetchSkillsByPlatform(platform: Platform) {
    store.setLoading(true)
    store.setError(null)

    try {
      const response = await api.get<{ data: UnifiedSkillsResponse }>(`${SKILL_HUB_BASE}/unified/${platform}`)
      const data = response.data.data

      // Merge with existing skills (replace platform-specific skills)
      const otherSkills = skills.value.filter(s => s.platform !== platform)
      const newSkills = data.skills.map(transformSkill)
      store.setSkills([...otherSkills, ...newSkills])

      // Update platform info
      if (data.platforms.length > 0) {
        const updatedPlatforms = platforms.value.map(p =>
          p.id === platform ? transformPlatform(data.platforms[0]) : p
        )
        store.setPlatforms(updatedPlatforms)
      }
    } catch (err: any) {
      store.setError(err.message || `Failed to fetch skills for ${platform}`)
      console.error(`Failed to fetch skills for ${platform}:`, err)
    } finally {
      store.setLoading(false)
    }
  }

  // Fetch marketplace trending
  async function fetchMarketplaceTrending(limit: number = 30, page: number = 1) {
    store.setMarketplaceLoading(true)
    store.setMarketplaceError(null)

    try {
      const response = await api.get<{ data: BackendMarketplaceResponse }>(
        `${SKILL_HUB_BASE}/marketplace/trending`,
        { params: { limit, page } }
      )
      const data = response.data.data

      store.setMarketplaceItems(
        data.items.map((item: BackendMarketplaceItem) => ({
          package: item.package,
          owner: item.owner,
          repo: item.repo,
          skill: item.skill,
          skillsShUrl: item.skills_sh_url
        })),
        data.cached
      )
    } catch (err: any) {
      store.setMarketplaceError(err.message || 'Failed to fetch marketplace')
      console.error('Failed to fetch marketplace:', err)
    } finally {
      store.setMarketplaceLoading(false)
    }
  }

  // Search marketplace
  async function searchMarketplace(query: string, limit: number = 30, page: number = 1) {
    store.setMarketplaceLoading(true)
    store.setMarketplaceError(null)

    try {
      const response = await api.get<{ data: BackendMarketplaceResponse }>(
        `${SKILL_HUB_BASE}/marketplace/search`,
        { params: { q: query, limit, page } }
      )
      const data = response.data.data

      store.setMarketplaceItems(
        data.items.map((item: BackendMarketplaceItem) => ({
          package: item.package,
          owner: item.owner,
          repo: item.repo,
          skill: item.skill,
          skillsShUrl: item.skills_sh_url
        })),
        data.cached
      )
    } catch (err: any) {
      store.setMarketplaceError(err.message || 'Failed to search marketplace')
      console.error('Failed to search marketplace:', err)
    } finally {
      store.setMarketplaceLoading(false)
    }
  }

  // Install skill to platforms
  async function installSkill(request: InstallRequest): Promise<OperationResponse> {
    try {
      const response = await api.post<{ data: OperationResponse }>(
        `${SKILL_HUB_BASE}/install`,
        request
      )

      // Refresh skills after install
      await fetchAllSkills()

      return response.data.data
    } catch (err: any) {
      console.error('Failed to install skill:', err)
      throw err
    }
  }

  // Remove skill from platforms
  async function removeSkill(request: RemoveRequest): Promise<OperationResponse> {
    try {
      const response = await api.post<{ data: OperationResponse }>(
        `${SKILL_HUB_BASE}/remove`,
        request
      )

      // Refresh skills after removal
      await fetchAllSkills()

      return response.data.data
    } catch (err: any) {
      console.error('Failed to remove skill:', err)
      throw err
    }
  }

  // Fetch platforms list
  async function fetchPlatforms() {
    try {
      const response = await api.get<{ data: PlatformSummary[] }>(`${SKILL_HUB_BASE}/agents`)
      store.setPlatforms(response.data.data.map(transformPlatform))
    } catch (err: any) {
      console.error('Failed to fetch platforms:', err)
    }
  }

  // Initialize - fetch all data
  async function initialize() {
    await Promise.all([
      fetchAllSkills(),
      fetchMarketplaceTrending()
    ])
  }

  // Refresh all data
  async function refresh() {
    await initialize()
  }

  // Fetch skill content (full SKILL.md with frontmatter + body)
  async function fetchSkillContent(skillDir: string): Promise<SkillContent> {
    const response = await api.get<{ data: any }>(`${SKILL_HUB_BASE}/skill/content`, {
      params: { skill_dir: skillDir }
    })
    const d = response.data.data
    return {
      name: d.name,
      description: d.description,
      category: d.category,
      tags: d.tags || [],
      content: d.content,
      raw: d.raw,
      skillDir: d.skill_dir
    }
  }

  // Save skill content (write full SKILL.md)
  async function saveSkillContent(skillDir: string, content: string): Promise<void> {
    await api.post(`${SKILL_HUB_BASE}/skill/content`, {
      skill_dir: skillDir,
      content
    })
    // Refresh skills list after save to reflect any metadata changes
    await fetchAllSkills()
  }

  return {
    // State (reactive refs from store)
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

    // Actions from store
    setFilter: store.setFilter,
    resetFilters: store.resetFilters,
    setActiveTab: store.setActiveTab,

    // API actions
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
    refresh
  }
}
