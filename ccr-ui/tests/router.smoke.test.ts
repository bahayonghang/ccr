import { describe, expect, it } from 'vitest'
import router from '@/router'

describe('router smoke', () => {
  it('keeps critical named routes registered', () => {
    const requiredRoutes = [
      'home',
      'settings',
      'codex',
      'gemini-mcp',
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

  it('redirects skills legacy entrypoints to the migration bridge', () => {
    const skillsRoute = router.getRoutes().find(route => route.path === '/skills')
    const marketRoute = router.getRoutes().find(route => route.path === '/market')
    const skillsAddRoute = router.getRoutes().find(route => route.path === '/skills/add')
    const skillsHubRoute = router.getRoutes().find(route => route.path === '/skills/hub')
    const skillsDetailRoute = router.getRoutes().find(route => route.path === '/skills/:platform/:name')
    const skillsManagerRoute = router.getRoutes().find(route => route.path === '/skills-manager')
    const skillportManagerRoute = router.getRoutes().find(route => route.path === '/skillport-manager')
    const opencodeSkillsRoute = router.getRoutes().find(route => route.path === '/opencode/skills')

    expect(skillsRoute?.redirect).toBeUndefined()
    expect(marketRoute?.redirect).toBe('/skills')
    expect(skillsAddRoute?.redirect).toBe('/skills')
    expect(skillsHubRoute?.redirect).toBe('/skills')
    expect(skillsDetailRoute?.redirect).toBe('/skills')
    expect(skillsManagerRoute?.redirect).toBe('/skills')
    expect(skillportManagerRoute?.redirect).toBe('/skills')
    expect(opencodeSkillsRoute?.redirect).toBe('/skills')
  })

  it('preserves generated platform child route paths', () => {
    const routePaths = router.getRoutes().map((route) => route.path)

    expect(routePaths).toContain('/gemini-cli/mcp')
    expect(routePaths).not.toContain('/droid')
    expect(routePaths).not.toContain('/droid/mcp')
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
      'sync',
      'configs',
      'usage',
      'opencode',
      'opencode-providers',
      'opencode-mcp',
      'opencode-agents',
      'opencode-commands',
      'opencode-plugins',
      'opencode-settings',
    ]

    for (const routeName of routeNames) {
      const route = router.getRoutes().find((candidate) => candidate.name === routeName)
      expect(route?.meta.hideGlobalBackground).toBe(true)
    }
  })
})
