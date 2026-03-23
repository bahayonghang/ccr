import { getIcon } from '@iconify/vue'
import { describe, expect, it } from 'vitest'
import { iconMap } from '@/config/icons'
import { registerAppIcons } from '@/config/iconRegistry'

const solarPrefix = 'solar:'

const configuredSolarIconNames = [...new Set(
  Object.values(iconMap)
    .filter((iconId): iconId is string => iconId.startsWith(solarPrefix))
    .map((iconId) => iconId.slice(solarPrefix.length))
)]

describe('icon registry smoke', () => {
  it('registers every configured solar icon in the local Iconify cache', async () => {
    await registerAppIcons()

    const missingIcons = configuredSolarIconNames.filter((iconName) => !getIcon(`solar:${iconName}`))

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
})
