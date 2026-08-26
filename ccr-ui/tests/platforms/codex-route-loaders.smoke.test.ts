import { describe, expect, it } from 'vitest'
import { codexRouteLoaders } from '@/features/codex/routeLoaders'

describe('codex route loaders', () => {
  it('exports catalog ids and task shorthand ids', () => {
    const ids = Object.keys(codexRouteLoaders)
    expect(ids).toEqual(
      expect.arrayContaining([
        'codex',
        'codex-mcp',
        'profiles',
        'codex-profiles',
        'agents',
        'codex-agents',
        'sessions',
        'codex-sessions',
        'slash-commands',
        'codex-slash-commands',
        'auth',
        'codex-auth',
        'settings',
        'codex-settings',
        'system-prompts',
        'codex-system-prompts',
      ]),
    )
    expect(ids).not.toContain('codex-tray-panel')
  })

  it('resolves each loader to a Component', async () => {
    const entries = Object.entries(codexRouteLoaders)
    for (const [, loader] of entries) {
      const mod = await loader()
      expect(typeof mod.Component).toBe('function')
    }
  })
})
