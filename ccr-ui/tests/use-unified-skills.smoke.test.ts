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
  skillsSourcesList: vi.fn(async () => []),
  getSkillHubAgents: vi.fn(async () => ({ platforms: [] })),
  getSkillHubAgentSkills: vi.fn(async () => ({ skills: [] })),
  installSkillHubSkill: vi.fn(),
  removeSkillHubSkill: vi.fn(),
  getSkillDetail: vi.fn(async () => ({
    id: 'sg_skill_alpha',
    name: 'skill-alpha',
    origin: 'marketplace',
    tags: ['sync'],
    install_count: 1,
    editable_installations: ['ins_skill_alpha'],
    installations: [
      {
        id: 'ins_skill_alpha',
        platform_id: 'codex',
        platform_name: 'Codex',
        install_path: '/tmp/skill-alpha',
        is_primary: true,
      },
    ],
  })),
  getSkillHubUnified: vi.fn(async () => ({
    skills: [
      {
        id: 'sg_skill_alpha',
        name: 'skill-alpha',
        origin: 'marketplace',
        tags: ['sync'],
        install_count: 1,
        editable_installations: ['ins_skill_alpha'],
        installations: [
          {
            id: 'ins_skill_alpha',
            platform_id: 'codex',
            platform_name: 'Codex',
            install_path: '/tmp/skill-alpha',
            is_primary: true,
          },
        ],
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
  getSkillHubSkillContent: vi.fn(async (skillId: string, installationId?: string | null) => ({
    skill_id: skillId,
    installation_id: installationId ?? 'ins_skill_alpha',
    name: 'skill-alpha',
    tags: ['sync'],
    raw: 'name: skill-alpha',
    content: 'name: skill-alpha',
    skill_dir: '/tmp/skill-alpha',
  })),
  saveSkillHubSkillContent: vi.fn(async (skillId: string, installationId: string, raw: string) => ({
    skill_id: skillId,
    installation_id: installationId,
    name: 'skill-alpha',
    tags: ['sync'],
    raw,
    content: raw,
    skill_dir: '/tmp/skill-alpha',
  })),
  importSkillFromGithub: vi.fn(),
  importSkillFromLocal: vi.fn(),
  importSkillViaNpx: vi.fn(),
  batchInstallSkills: vi.fn(),
  skillsOnboardingCandidates: vi.fn(async (): Promise<unknown[]> => []),
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
    expect(apiMocks.skillsSourcesList).toHaveBeenCalledTimes(1)
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

  it('loads onboarding candidates on demand', async () => {
    apiMocks.skillsOnboardingCandidates.mockImplementationOnce(async () => [
      {
        skill_id: 'sg_skill_alpha',
        name: 'Skill Alpha',
        platform_ids: ['codex'],
        installation_ids: ['ins_skill_alpha'],
        reason: 'missing_source',
      },
    ])

    const { useUnifiedSkills } = await import('@/composables/useUnifiedSkills')
    const skillsApi = useUnifiedSkills()

    const candidates = await skillsApi.loadOnboardingCandidates(true)

    expect(apiMocks.skillsOnboardingCandidates).toHaveBeenCalledTimes(1)
    expect(candidates).toHaveLength(1)
    expect(candidates[0]?.skillId).toBe('sg_skill_alpha')
    expect(skillsApi.onboardingCandidates.value).toHaveLength(1)
  })

  it('preserves backend target statuses when loading skill detail', async () => {
    apiMocks.getSkillDetail.mockImplementationOnce(async () => ({
      id: 'sg_skill_alpha',
      name: 'skill-alpha',
      origin: 'repo',
      source_ref: 'src_alpha',
      source_label: 'Repo Alpha',
      tags: ['sync'],
      install_count: 1,
      editable_installations: ['ins_skill_alpha'],
      installations: [
        {
          id: 'ins_skill_alpha',
          platform_id: 'codex',
          platform_name: 'Codex',
          install_path: '/tmp/skill-alpha',
          is_primary: true,
        },
      ],
      targets: [
        {
          id: 'ins_skill_alpha',
          platform_id: 'codex',
          platform_name: 'Codex',
          target_path: '/tmp/skill-alpha',
          status: 'pending',
          synced_at: 123,
          is_primary: true,
        },
      ],
    }))

    const { useUnifiedSkills } = await import('@/composables/useUnifiedSkills')
    const skillsApi = useUnifiedSkills()

    const detail = await skillsApi.ensureDetail('sg_skill_alpha', true)

    expect(detail?.targets).toHaveLength(1)
    expect(detail?.targets[0]?.status).toBe('pending')
    expect(detail?.lifecycle.targetCount).toBe(1)
    expect(detail?.lifecycle.healthyTargetCount).toBe(0)
    expect(detail?.lifecycle.hasErrors).toBe(true)
  })
})
