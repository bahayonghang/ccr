import { getIcon } from '@iconify/react'
import { readFileSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'
import { iconMap } from '@/config/icons'
import { registerAppIcons } from '@/config/iconRegistry'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')

const solarPrefix = 'solar:'

const configuredSolarIconNames = [
  ...new Set(
    Object.values(iconMap)
      .filter((iconId) => iconId.startsWith(solarPrefix))
      .map((iconId) => iconId.slice(solarPrefix.length))
  ),
]

describe('icon registry smoke', () => {
  it('registers every configured solar icon in the local Iconify cache', async () => {
    await registerAppIcons()

    const missingIcons = configuredSolarIconNames.filter(
      (iconName) => !getIcon(`solar:${iconName}`)
    )

    expect(missingIcons).toEqual([])
  })

  it('keeps configured solar icons on the expected 24x24 canvas', async () => {
    await registerAppIcons()

    const invalidCanvasIcons = configuredSolarIconNames.filter((iconName) => {
      const icon = getIcon(`solar:${iconName}`)
      return !icon || icon.width !== 24 || icon.height !== 24
    })

    expect(invalidCanvasIcons).toEqual([])
  })

  it('registers into @iconify/react before the React tree mounts', () => {
    const registry = readFileSync(path.join(root, 'src/config/iconRegistry.ts'), 'utf8')
    const main = readFileSync(path.join(root, 'src/main.tsx'), 'utf8')
    expect(registry).toContain("from '@iconify/react'")
    expect(registry).not.toContain("from '@iconify/vue'")
    expect(main).toContain('registerShellIcons()')
  })
})
