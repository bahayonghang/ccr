import { setActivePinia, createPinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'
import { useSkillsStore } from '@/stores/skills'
import type { PlatformSummary, SkillRecord, UnifiedSkill } from '@/types/skills'
import { toInstalledPackageSet } from '@/utils/skills'

const samplePlatforms: PlatformSummary[] = [
  {
    id: 'codex',
    display_name: 'Codex',
    global_skills_dir: '/tmp/codex',
    detected: true,
    installed_count: 2,
  },
  {
    id: 'gemini',
    display_name: 'Gemini CLI',
    global_skills_dir: '/tmp/gemini',
    detected: false,
    installed_count: 0,
  },
]

const sampleSkills: UnifiedSkill[] = [
  {
    name: 'Skill Alpha',
    description: 'Alpha description',
    skillDir: '/tmp/skills/alpha',
    platform: 'codex',
    platformName: 'Codex',
    category: 'ops',
    tags: ['sync', 'shell'],
  },
  {
    name: 'Skill Beta',
    description: 'Beta description',
    skillDir: '/tmp/skills/beta',
    platform: 'gemini',
    platformName: 'Gemini CLI',
    category: 'analysis',
    tags: ['report'],
  },
]

const sampleSkillRecords: SkillRecord[] = [
  {
    id: 'sg_skill_alpha',
    name: 'Skill Alpha',
    description: 'Alpha description',
    category: 'ops',
    tags: ['sync', 'shell'],
    origin: 'repo',
    sourceRef: 'src_alpha',
    sourceLabel: 'Repo Alpha',
    installCount: 1,
    editableInstallations: ['ins_skill_alpha'],
    installations: [
      {
        id: 'ins_skill_alpha',
        platformId: 'codex',
        platformName: 'Codex',
        installPath: '/tmp/skills/alpha',
        installMode: 'copy',
        isPrimary: true,
      },
    ],
    targets: [
      {
        id: 'ins_skill_alpha',
        platformId: 'codex',
        platformName: 'Codex',
        targetPath: '/tmp/skills/alpha',
        syncMode: 'copy',
        status: 'ok',
        isPrimary: true,
      },
    ],
    lifecycle: {
      sourceRef: 'src_alpha',
      sourceLabel: 'Repo Alpha',
      hasErrors: false,
      targetCount: 1,
      healthyTargetCount: 1,
    },
  },
  {
    id: 'sg_skill_beta',
    name: 'Skill Beta',
    description: 'Beta description',
    category: 'analysis',
    tags: ['report'],
    origin: 'marketplace',
    sourceRef: 'owner/skill-beta',
    sourceLabel: 'owner/skill-beta',
    installCount: 1,
    editableInstallations: ['ins_skill_beta'],
    installations: [
      {
        id: 'ins_skill_beta',
        platformId: 'gemini',
        platformName: 'Gemini CLI',
        installPath: '/tmp/skills/beta',
        installMode: 'copy',
        isPrimary: true,
      },
    ],
    targets: [
      {
        id: 'ins_skill_beta',
        platformId: 'gemini',
        platformName: 'Gemini CLI',
        targetPath: '/tmp/skills/beta',
        syncMode: 'copy',
        status: 'ok',
        isPrimary: true,
      },
    ],
    lifecycle: {
      sourceRef: 'owner/skill-beta',
      sourceLabel: 'owner/skill-beta',
      hasErrors: false,
      targetCount: 1,
      healthyTargetCount: 1,
    },
  },
]

describe('skills store smoke', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('filters and resets installed skills predictably', () => {
    const store = useSkillsStore()
    store.setPlatforms(samplePlatforms)
    store.setSkills(sampleSkills)

    store.setFilters({
      search: '',
      source: 'all',
      category: null,
      tags: [],
      platform: 'codex',
    })
    expect(store.filteredSkills).toHaveLength(1)
    expect(store.filteredSkills[0]?.name).toBe('Skill Alpha')
    expect(store.availableCategories).toEqual(['ops'])

    store.setFilter('category', 'ops')
    store.setFilter('tags', ['sync'])
    expect(store.filteredSkills).toHaveLength(1)

    store.resetFilters()
    expect(store.filters.platform).toBe('all')
    expect(store.filters.category).toBeNull()
    expect(store.filters.tags).toEqual([])
    expect(store.filteredSkills).toHaveLength(2)
  })

  it('drops stale category and tags when the platform changes', () => {
    const store = useSkillsStore()
    store.setPlatforms(samplePlatforms)
    store.setSkills(sampleSkills)

    store.setFilter('category', 'ops')
    store.setFilter('tags', ['sync'])
    store.setFilter('platform', 'gemini')

    expect(store.filters.category).toBeNull()
    expect(store.filters.tags).toEqual([])
    expect(store.availableCategories).toEqual(['analysis'])
  })

  it('keeps marketplace install detection stable when installed filters narrow the list', () => {
    const store = useSkillsStore()
    store.setPlatforms(samplePlatforms)
    store.setSkills(sampleSkills)

    store.setFilter('platform', 'codex')

    const installedPackages = toInstalledPackageSet(store.skills)

    expect(store.filteredSkills).toHaveLength(1)
    expect(installedPackages.has('Skill Alpha')).toBe(true)
    expect(installedPackages.has('Skill Beta')).toBe(true)
  })

  it('tracks marketplace load state separately from the available count', () => {
    const store = useSkillsStore()

    expect(store.marketplaceLoaded).toBe(false)
    expect(store.stats.available).toBe(0)

    store.setMarketplaceItems(
      [
        {
          package: 'owner/skill-alpha',
          owner: 'owner',
          repo: 'skill-alpha',
          skillsShUrl: 'https://skills.sh/owner/skill-alpha',
        },
      ],
      false
    )

    expect(store.marketplaceLoaded).toBe(true)
    expect(store.stats.available).toBe(1)
  })

  it('derives lifecycle metadata from installed skills', () => {
    const store = useSkillsStore()
    store.setPlatforms(samplePlatforms)
    store.setSkills(sampleSkillRecords)

    const alpha = store.skills.find((skill: SkillRecord) => skill.id === 'sg_skill_alpha')

    expect(alpha?.targets).toHaveLength(1)
    expect(alpha?.targets[0]?.status).toBe('ok')
    expect(alpha?.lifecycle.targetCount).toBe(1)
    expect(alpha?.lifecycle.healthyTargetCount).toBe(1)
    expect(alpha?.lifecycle.hasErrors).toBe(false)
    expect(alpha?.lifecycle.sourceRef).toBe('src_alpha')
  })

  it('preserves unhealthy targets when normalizing provided skill records', () => {
    const store = useSkillsStore()
    store.setPlatforms(samplePlatforms)
    store.setSkills([
      {
        ...sampleSkillRecords[0],
        id: 'sg_skill_gamma',
        name: 'Skill Gamma',
        sourceRef: 'src_gamma',
        sourceLabel: 'Repo Gamma',
        installations: [
          {
            id: 'ins_skill_gamma',
            platformId: 'codex',
            platformName: 'Codex',
            installPath: '/tmp/skills/gamma',
            installMode: 'copy',
            isPrimary: true,
          },
        ],
        targets: [
          {
            id: 'ins_skill_gamma',
            platformId: 'codex',
            platformName: 'Codex',
            targetPath: '/tmp/skills/gamma',
            syncMode: 'copy',
            status: 'stale',
            syncedAt: 42,
            isPrimary: true,
          },
        ],
        lifecycle: {
          hasErrors: false,
          targetCount: 0,
          healthyTargetCount: 0,
        },
        editableInstallations: ['ins_skill_gamma'],
      },
    ])

    const gamma = store.skills.find((skill: SkillRecord) => skill.id === 'sg_skill_gamma')

    expect(gamma?.targets).toHaveLength(1)
    expect(gamma?.targets[0]?.status).toBe('stale')
    expect(gamma?.lifecycle.targetCount).toBe(1)
    expect(gamma?.lifecycle.healthyTargetCount).toBe(0)
    expect(gamma?.lifecycle.hasErrors).toBe(true)
    expect(gamma?.lifecycle.lastSyncedAt).toBe(42)
  })

  it('tracks workflow state for install-like operations', async () => {
    const store = useSkillsStore()
    store.setPlatforms(samplePlatforms)
    store.setSkills(sampleSkillRecords)

    expect(store.workflowState.status).toBe('idle')

    await expect(store.removeSkillRecord('sg_skill_alpha')).rejects.toBeDefined()

    expect(store.workflowState.action).toBe('remove-skill')
    expect(store.workflowState.target).toBe('sg_skill_alpha')
    expect(store.workflowState.status).toBe('error')
  })
})
