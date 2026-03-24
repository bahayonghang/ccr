import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

const apiMocks = vi.hoisted(() => ({
  getSkillHubTrending: vi.fn(async () => ({
    items: [
      {
        package: 'owner/skill-alpha',
        owner: 'owner',
        repo: 'skill-alpha',
        skills_sh_url: 'https://skills.sh/owner/skill-alpha',
      },
    ],
    total: 1,
    cached: false,
  })),
  searchSkillHubMarketplace: vi.fn(async () => ({
    items: [],
    total: 0,
    cached: false,
  })),
  getSkillHubAgents: vi.fn(async () => ({ platforms: [] })),
  getSkillHubAgentSkills: vi.fn(async () => ({ skills: [] })),
  installSkillHubSkill: vi.fn(),
  removeSkillHubSkill: vi.fn(),
  getSkillHubUnified: vi.fn(async () => ({
    skills: [
      {
        name: 'skill-alpha',
        skill_dir: '/tmp/skill-alpha',
        platform: 'codex',
        platform_name: 'Codex',
        tags: ['sync'],
      },
    ],
    platforms: [
      {
        id: 'codex',
        display_name: 'Codex',
        global_skills_dir: '/tmp/codex',
        detected: true,
        installed_count: 1,
      },
    ],
  })),
  getSkillHubSkillContent: vi.fn(),
  saveSkillHubSkillContent: vi.fn(),
  importSkillFromGithub: vi.fn(),
  importSkillFromLocal: vi.fn(),
  importSkillViaNpx: vi.fn(),
  batchInstallSkills: vi.fn(),
  checkNpxAvailability: vi.fn(),
  browseForFolder: vi.fn(),
}))

vi.mock('@/api', () => apiMocks)

describe('useUnifiedSkills smoke', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    Object.values(apiMocks).forEach((mock) => mock.mockClear())
  })

  it('loads installed skills without prefetching marketplace by default', async () => {
    const { useUnifiedSkills } = await import('@/composables/useUnifiedSkills')
    const skillsApi = useUnifiedSkills()

    await skillsApi.initialize()

    expect(apiMocks.getSkillHubUnified).toHaveBeenCalledTimes(1)
    expect(apiMocks.getSkillHubTrending).not.toHaveBeenCalled()
    expect(skillsApi.marketplaceLoaded.value).toBe(false)
    expect(skillsApi.stats.value.installed).toBe(1)
    expect(skillsApi.stats.value.available).toBe(0)
  })

  it('loads marketplace only when explicitly requested', async () => {
    const { useUnifiedSkills } = await import('@/composables/useUnifiedSkills')
    const skillsApi = useUnifiedSkills()

    await skillsApi.fetchMarketplaceTrending()

    expect(apiMocks.getSkillHubTrending).toHaveBeenCalledTimes(1)
    expect(skillsApi.marketplaceLoaded.value).toBe(true)
    expect(skillsApi.stats.value.available).toBe(1)
  })
})
