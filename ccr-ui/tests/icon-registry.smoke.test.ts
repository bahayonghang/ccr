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
})
