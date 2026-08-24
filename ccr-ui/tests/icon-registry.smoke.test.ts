import { createElement } from 'react'
import { readFileSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { render } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { iconMap } from '@/config/icons'
import { registerAppIcons } from '@/config/iconRegistry'
import { solarIconSubset } from '@/config/solarIconSubset'
import { SIcon } from '@/ui/s-icon'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')

const solarPrefix = 'solar:'

const configuredSolarIconNames = [
  ...new Set(
    Object.values(iconMap)
      .filter((iconId) => iconId.startsWith(solarPrefix))
      .map((iconId) => iconId.slice(solarPrefix.length)),
  ),
]

describe('icon registry smoke', () => {
  it('registers every configured solar icon in the local Iconify cache', async () => {
    await registerAppIcons()
    const missingIcons = configuredSolarIconNames.filter(
      (iconName) => !solarIconSubset.icons[iconName],
    )
    expect(missingIcons).toEqual([])
  })

  it('keeps configured solar icons on the expected 24x24 canvas', async () => {
    await registerAppIcons()
    expect(solarIconSubset.width).toBe(24)
    expect(solarIconSubset.height).toBe(24)
  })

  it('renders a registered icon from the offline Iconify entry', async () => {
    await registerAppIcons()
    const view = render(createElement(SIcon, { name: 'Home' }))
    expect(view.container.querySelector('svg')).toBeTruthy()
    view.unmount()
  })

  it('registers into the offline Iconify entry before the React tree mounts', () => {
    const registry = readFileSync(path.join(root, 'src/config/iconRegistry.ts'), 'utf8')
    const icons = readFileSync(path.join(root, 'src/ui/s-icon.tsx'), 'utf8')
    const main = readFileSync(path.join(root, 'src/main.tsx'), 'utf8')
    expect(registry).toContain("from '@iconify/react/offline'")
    expect(icons).toContain("from '@iconify/react/offline'")
    expect(registry).not.toContain("from '@iconify/vue'")
    expect(main).toContain('registerShellIcons()')
  })
})
