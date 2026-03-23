import { describe, expect, it } from 'vitest'
import router from '@/router'

describe('router smoke', () => {
  it('keeps critical named routes registered', () => {
    const requiredRoutes = [
      'home',
      'codex',
      'usage',
      'monitoring',
      'mcp',
      'skills',
      'sessions',
      'wsl-management',
      'ssh-management',
      'opencode'
    ]

    for (const routeName of requiredRoutes) {
      expect(router.hasRoute(routeName)).toBe(true)
    }
  })

  it('keeps /stats redirected to /usage', () => {
    const statsRoute = router.getRoutes().find(route => route.path === '/stats')

    expect(statsRoute?.redirect).toBe('/usage')
  })

  it('hides the global background on routes with page-level decorative backgrounds', () => {
    const routeNames = [
      'claude-code',
      'gemini-cli',
      'qwen',
      'iflow',
      'droid',
      'sync',
      'configs',
      'usage',
      'opencode',
      'opencode-providers',
      'opencode-mcp',
      'opencode-plugins',
    ]

    for (const routeName of routeNames) {
      const route = router.getRoutes().find((candidate) => candidate.name === routeName)
      expect(route?.meta.hideGlobalBackground).toBe(true)
    }
  })
})
