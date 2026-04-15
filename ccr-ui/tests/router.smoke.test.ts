import { describe, expect, it } from 'vitest'
import router from '@/router'

describe('router smoke', () => {
  it('keeps critical named routes registered', () => {
    const requiredRoutes = [
      'home',
      'settings',
      'codex',
      'gemini-mcp',
      'droid',
      'droid-mcp',
      'usage',
      'monitoring',
      'mcp',
      'skills',
      'sessions',
      'wsl-management',
      'ssh-management',
      'opencode',
      'opencode-agents',
      'opencode-commands',
      'opencode-skills',
      'opencode-settings',
    ]

    for (const routeName of requiredRoutes) {
      expect(router.hasRoute(routeName)).toBe(true)
    }
  })

  it('keeps /stats redirected to /usage', () => {
    const statsRoute = router.getRoutes().find(route => route.path === '/stats')

    expect(statsRoute?.redirect).toBe('/usage')
  })

  it('redirects /market to the unified skills manager', () => {
    const marketRoute = router.getRoutes().find(route => route.path === '/market')

    expect(marketRoute?.redirect).toBe('/skills-manager')
  })

  it('preserves generated platform child route paths', () => {
    const routePaths = router.getRoutes().map((route) => route.path)

    expect(routePaths).toContain('/gemini-cli/mcp')
    expect(routePaths).toContain('/droid/mcp')
  })

  it('registers the global settings route as its own navigation group', () => {
    const settingsRoute = router.getRoutes().find((route) => route.name === 'settings')

    expect(settingsRoute?.path).toBe('/settings')
    expect(settingsRoute?.meta.group).toBe('settings')
  })

  it('hides the global background on routes with page-level decorative backgrounds', () => {
    const routeNames = [
      'claude-code',
      'gemini-cli',
      'droid',
      'sync',
      'configs',
      'usage',
      'opencode',
      'opencode-providers',
      'opencode-mcp',
      'opencode-agents',
      'opencode-commands',
      'opencode-skills',
      'opencode-plugins',
      'opencode-settings',
    ]

    for (const routeName of routeNames) {
      const route = router.getRoutes().find((candidate) => candidate.name === routeName)
      expect(route?.meta.hideGlobalBackground).toBe(true)
    }
  })
})
