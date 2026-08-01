import { describe, expect, it } from 'vitest'
import {
  mainLayoutGroupTitleMap,
  mainLayoutNavSections,
  mainLayoutRouteTitleMap,
} from '@/config/mainLayoutShell'
import { getModuleSubnavItems } from '@/config/moduleSubnav'
import router from '@/router'

describe('router smoke', () => {
  it('keeps critical named routes registered', () => {
    const requiredRoutes = [
      'dashboard',
      'settings',
      'codex',
      'grok',
      'grok-profiles',
      'grok-settings',
      'antigravity',
      'gemini-mcp',
      'gemini-agents',
      'gemini-slash-commands',
      'gemini-plugins',
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
      'claude-system-prompts',
      'codex-system-prompts',
      'gemini-system-prompts',
      'opencode-system-prompts',
    ]

    for (const routeName of requiredRoutes) {
      expect(router.hasRoute(routeName)).toBe(true)
    }
  })

  it('keeps /stats redirected to /usage', () => {
    const statsRoute = router.getRoutes().find((route) => route.path === '/stats')

    expect(statsRoute?.redirect).toBe('/usage')
  })

  it('redirects the legacy CCR control entrypoint to the unified command center', () => {
    const ccrControlRoute = router.getRoutes().find((route) => route.path === '/ccr-control')

    expect(ccrControlRoute?.redirect).toBe('/commands/ccr')
    expect(ccrControlRoute?.components).toBeUndefined()
    expect(mainLayoutRouteTitleMap['ccr-control']).toBe('nav.commands')
  })

  it('shows only one CCR command entry in the tools navigation', () => {
    const toolsSection = mainLayoutNavSections.find((section) => section.id === 'tools')
    const commandLikeItems = toolsSection?.items.filter((item) =>
      [item.to, item.labelKey].some((value) => /ccr-control|nav\.ccrControl|\/commands/.test(value))
    )

    expect(commandLikeItems).toEqual([
      expect.objectContaining({ to: '/commands', labelKey: 'nav.commands' }),
    ])
  })

  it('keeps the root route registered as the cached dashboard', () => {
    const dashboardRoute = router.getRoutes().find((route) => route.name === 'dashboard')

    expect(dashboardRoute?.path).toBe('/')
    expect(dashboardRoute?.meta.cacheKey).toBe('DashboardView')
  })

  it('registers the cached Grok home and both placeholder child routes', () => {
    const homeRoute = router.getRoutes().find((route) => route.name === 'grok')
    const profilesRoute = router.getRoutes().find((route) => route.name === 'grok-profiles')
    const settingsRoute = router.getRoutes().find((route) => route.name === 'grok-settings')

    expect(homeRoute).toMatchObject({
      path: '/grok',
      meta: { cache: true, cacheKey: 'GrokView', depth: 1, group: 'grok' },
    })
    expect(profilesRoute).toMatchObject({
      path: '/grok/profiles',
      meta: { depth: 2, group: 'grok' },
    })
    expect(settingsRoute).toMatchObject({
      path: '/grok/settings',
      meta: { depth: 2, group: 'grok' },
    })

    const modulesSection = mainLayoutNavSections.find((section) => section.id === 'modules')
    expect(modulesSection?.items).toContainEqual(expect.objectContaining({
      to: '/grok',
      labelKey: 'nav.grok',
      icon: 'Zap',
    }))
    expect(mainLayoutRouteTitleMap.grok).toBe('nav.grok')
    expect(mainLayoutRouteTitleMap['grok-profiles']).toBe('nav.profiles')
    expect(mainLayoutRouteTitleMap['grok-settings']).toBe('common.settings')
    expect(mainLayoutGroupTitleMap.grok).toBe('nav.grok')
    expect(getModuleSubnavItems('grok').map((item) => item.href)).toEqual([
      '/grok',
      '/grok/profiles',
      '/grok/settings',
    ])
  })

  it('keeps legacy Gemini CLI routes redirected to Antigravity', () => {
    const homeRoute = router.getRoutes().find((route) => route.path === '/gemini-cli')
    const mcpRoute = router.getRoutes().find((route) => route.path === '/gemini-cli/mcp')
    const commandsRoute = router
      .getRoutes()
      .find((route) => route.path === '/gemini-cli/slash-commands')
    const agentsRoute = router.getRoutes().find((route) => route.path === '/gemini-cli/agents')
    const pluginsRoute = router.getRoutes().find((route) => route.path === '/gemini-cli/plugins')
    const promptsRoute = router
      .getRoutes()
      .find((route) => route.path === '/gemini-cli/system-prompts')

    expect(homeRoute?.redirect).toBe('/antigravity')
    expect(mcpRoute?.redirect).toBe('/antigravity/mcp')
    expect(commandsRoute?.redirect).toBe('/antigravity/slash-commands')
    expect(agentsRoute?.redirect).toBe('/antigravity/agents')
    expect(pluginsRoute?.redirect).toBe('/antigravity/plugins')
    expect(promptsRoute?.redirect).toBe('/antigravity/system-prompts')
  })

  it('redirects skills legacy entrypoints to the migration bridge', () => {
    const skillsRoute = router.getRoutes().find((route) => route.path === '/skills')
    const marketRoute = router.getRoutes().find((route) => route.path === '/market')
    const skillsAddRoute = router.getRoutes().find((route) => route.path === '/skills/add')
    const skillsHubRoute = router.getRoutes().find((route) => route.path === '/skills/hub')
    const skillsDetailRoute = router
      .getRoutes()
      .find((route) => route.path === '/skills/:platform/:name')
    const skillsManagerRoute = router.getRoutes().find((route) => route.path === '/skills-manager')
    const skillportManagerRoute = router
      .getRoutes()
      .find((route) => route.path === '/skillport-manager')
    const opencodeSkillsRoute = router
      .getRoutes()
      .find((route) => route.path === '/opencode/skills')

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

    expect(routePaths).toContain('/antigravity/mcp')
    expect(routePaths).toContain('/antigravity/agents')
    expect(routePaths).toContain('/antigravity/slash-commands')
    expect(routePaths).toContain('/antigravity/plugins')
    expect(routePaths).toContain('/antigravity/system-prompts')
    expect(routePaths).toContain('/claude-code/system-prompts')
    expect(routePaths).toContain('/codex/system-prompts')
    expect(routePaths).toContain('/opencode/system-prompts')
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
      'antigravity',
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
