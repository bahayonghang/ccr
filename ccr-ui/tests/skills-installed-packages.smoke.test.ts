import { describe, expect, it } from 'vitest'
import { isMarketplaceItemInstalled, toInstalledPackageSet } from '@/utils/skills'

describe('skills installed package set smoke', () => {
  it('derives installed packages from the full installed skill list', () => {
    const installedPackages = toInstalledPackageSet([
      { name: 'Skill Alpha' },
      { name: 'Skill Beta' },
    ])

    expect(installedPackages.has('Skill Alpha')).toBe(true)
    expect(installedPackages.has('Skill Beta')).toBe(true)
    expect(installedPackages.size).toBe(2)
  })

  it('matches marketplace installs through source refs instead of list filtering side effects', () => {
    const installed = isMarketplaceItemInstalled(
      {
        package: 'owner/skill-alpha',
        owner: 'owner',
        repo: 'skill-alpha',
        skill: 'Skill Alpha',
        skillsShUrl: 'https://skills.sh/owner/skill-alpha',
      },
      [
        {
          name: 'Skill Alpha',
          origin: 'marketplace',
          sourceRef: 'owner/skill-alpha',
          sourceLabel: 'owner/skill-alpha',
        },
      ],
    )

    expect(installed).toBe(true)
  })
})
