import { describe, expect, it } from 'vitest'
import { antigravityRouteLoaders, geminiRouteLoaders } from '@/features/gemini/routeLoaders'

describe('gemini route loaders', () => {
  it('exports antigravity catalog ids', () => {
    expect(geminiRouteLoaders).toBe(antigravityRouteLoaders)
    const ids = Object.keys(geminiRouteLoaders)
    expect(ids).toEqual(
      expect.arrayContaining([
        'antigravity',
        'gemini-slash-commands',
        'gemini-mcp',
        'gemini-agents',
        'gemini-plugins',
        'gemini-system-prompts',
        'agent-detail',
        'agents',
      ]),
    )
  })

  it('resolves each loader to a Component', async () => {
    for (const loader of Object.values(geminiRouteLoaders)) {
      const mod = await loader()
      expect(typeof mod.Component).toBe('function')
    }
  })
})
