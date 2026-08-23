import { describe, expect, it } from 'vitest'
import { configsRouteLoaders } from '@/features/configs/routeLoaders'

describe('configs route loaders', () => {
  it('exports configs, settings, and converter ids', () => {
    expect(Object.keys(configsRouteLoaders)).toEqual(['configs', 'settings', 'converter'])
  })

  it('resolves each loader to a Component', async () => {
    for (const loader of Object.values(configsRouteLoaders)) {
      const mod = await loader()
      expect(typeof mod.Component).toBe('function')
    }
  })
})
