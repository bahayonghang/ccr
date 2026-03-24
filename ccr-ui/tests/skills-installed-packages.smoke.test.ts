import { describe, expect, it } from 'vitest'
import { toInstalledPackageSet } from '@/utils/skills'

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
})
