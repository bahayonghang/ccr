import { describe, expect, it } from 'vitest'
import { opencodeRouteLoaders } from '@/features/opencode/routeLoaders'

describe('opencode route loaders', () => {
  it('exports catalog ids', () => {
    const ids = Object.keys(opencodeRouteLoaders)
    expect(ids).toEqual(
      expect.arrayContaining([
        'opencode',
        'opencode-providers',
        'opencode-mcp',
        'opencode-agents',
        'opencode-commands',
        'opencode-plugins',
        'opencode-settings',
        'opencode-system-prompts',
      ]),
    )
  })

  it('resolves each loader to a Component', async () => {
    for (const loader of Object.values(opencodeRouteLoaders)) {
      const mod = await loader()
      expect(typeof mod.Component).toBe('function')
    }
  })
})
