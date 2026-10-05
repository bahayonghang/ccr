import { describe, expect, it } from 'vitest'
import { grokRouteLoaders } from '@/features/grok/routeLoaders'

describe('grok route loaders', () => {
  it('exports catalog ids and shorthand ids', () => {
    const ids = Object.keys(grokRouteLoaders)
    expect(ids).toEqual(
      expect.arrayContaining([
        'grok',
        'grok-auth',
        'auth',
        'grok-profiles',
        'profiles',
        'grok-settings',
        'settings',
      ]),
    )
  })

  it('resolves each loader to a Component', async () => {
    for (const loader of Object.values(grokRouteLoaders)) {
      const mod = await loader()
      expect(typeof mod.Component).toBe('function')
    }
  })
})
