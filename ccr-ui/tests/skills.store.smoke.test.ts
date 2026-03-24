import { setActivePinia, createPinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'
import { useSkillsStore } from '@/stores/skills'
import type { PlatformSummary, UnifiedSkill } from '@/types/skills'
import { toInstalledPackageSet } from '@/utils/skills'

const samplePlatforms: PlatformSummary[] = [
  {
    id: 'codex',
    display_name: 'Codex',
    global_skills_dir: '/tmp/codex',
    detected: true,
    installed_count: 2
  },
  {
    id: 'gemini',
    display_name: 'Gemini CLI',
    global_skills_dir: '/tmp/gemini',
    detected: false,
    installed_count: 0
  }
]

const sampleSkills: UnifiedSkill[] = [
  {
    name: 'Skill Alpha',
    description: 'Alpha description',
    skillDir: '/tmp/skills/alpha',
    platform: 'codex',
    platformName: 'Codex',
    category: 'ops',
    tags: ['sync', 'shell']
  },
  {
    name: 'Skill Beta',
    description: 'Beta description',
    skillDir: '/tmp/skills/beta',
    platform: 'gemini',
    platformName: 'Gemini CLI',
    category: 'analysis',
    tags: ['report']
  }
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

    store.setMarketplaceItems([
      {
        package: 'owner/skill-alpha',
        owner: 'owner',
        repo: 'skill-alpha',
        skillsShUrl: 'https://skills.sh/owner/skill-alpha',
      },
    ], false)

    expect(store.marketplaceLoaded).toBe(true)
    expect(store.stats.available).toBe(1)
  })
})
