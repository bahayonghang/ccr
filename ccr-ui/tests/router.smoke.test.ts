import { describe, expect, it } from 'vitest'
import { matchRoutes } from 'react-router'
import {
  mainLayoutGroupTitleMap,
  mainLayoutNavSections,
  mainLayoutRouteTitleMap,
} from '@/config/mainLayoutShell'
import { getModuleSubnavItems } from '@/config/moduleSubnav'
import { flattenCatalog } from '@/shell/routeCatalog'
import { appRoutes } from '@/shell/router'
import { assertHandleKeys, ROUTE_HANDLE_KEYS } from '@/shell/routeHandle'

const flat = flattenCatalog()
const byId = (id: string) => flat.find((route) => route.id === id)
const byPath = (path: string) => flat.find((route) => route.path === path)

/** 75 条路径门：顺序与 `flattenCatalog()` / shell-port `route-inventory.md` 对齐。 */
const EXPECTED_FLAT_PATHS = [
  '/tray/codex',
  '/',
  '/',
  '/settings',
  '/claude-code',
  '/claude-code/settings',
  '/claude-code/system-prompts',
  '/claude-code/profiles',
  '/claude-code/auth',
  '/codex',
  '/grok',
  '/grok/auth',
  '/grok/profiles',
  '/grok/settings',
  '/antigravity',
  '/gemini-cli',
  '/ccr-control',
  '/commands/:client?',
  '/converter',
  '/sync',
  '/configs',
  '/stats',
  '/budget',
  '/pricing',
  '/usage',
  '/monitoring',
  '/sessions',
  '/mcp',
  '/mcp/unified',
  '/mcp-manager',
  '/slash-commands',
  '/agents',
  '/agents/:name',
  '/skills',
  '/skills-manager',
  '/skillport-manager',
  '/skills/add',
  '/skills/hub',
  '/skills/:platform/:name',
  '/market',
  '/plugins',
  '/hooks',
  '/output-styles',
  '/statusline',
  '/checkin/manage/:accountId',
  '/checkin',
  '/codex/mcp',
  '/codex/profiles',
  '/codex/agents',
  '/codex/sessions',
  '/codex/slash-commands',
  '/codex/auth',
  '/codex/settings',
  '/codex/system-prompts',
  '/antigravity/slash-commands',
  '/gemini-cli/slash-commands',
  '/gemini-cli/mcp',
  '/gemini-cli/agents',
  '/gemini-cli/plugins',
  '/antigravity/system-prompts',
  '/gemini-cli/system-prompts',
  '/opencode',
  '/opencode/providers',
  '/opencode/mcp',
  '/opencode/agents',
  '/opencode/commands',
  '/opencode/skills',
  '/opencode/plugins',
  '/opencode/settings',
  '/opencode/system-prompts',
  '/wsl',
  '/ssh',
  '/antigravity/mcp',
  '/antigravity/agents',
  '/antigravity/plugins',
] as const

describe('router smoke', () => {
  it('keeps 75 route records matching the route inventory', () => {
    expect(EXPECTED_FLAT_PATHS).toHaveLength(75)
    expect(flat).toHaveLength(75)
    expect(flat.map((route) => route.path)).toEqual([...EXPECTED_FLAT_PATHS])
    const recordKeys = flat.map(
      (route) => `${route.path}::${route.id ?? ''}::${route.redirect ?? ''}`,
    )
    expect(new Set(recordKeys).size).toBe(75)
  })

  it('keeps handle keys inside the allowlist', () => {
    const violations = flat.flatMap((route) =>
      assertHandleKeys(route.handle, route.id ?? route.path),
    )
    expect(violations).toEqual([])
    expect(ROUTE_HANDLE_KEYS).toContain('cache')
  })

  it('keeps critical named routes registered', () => {
    const required = [
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
    for (const id of required) {
      expect(byId(id), id).toBeTruthy()
    }
  })

  it('keeps /stats redirected to /usage', () => {
    expect(byPath('/stats')?.redirect).toBe('/usage')
  })

  it('redirects the legacy CCR control entrypoint to the unified command center', () => {
    expect(byId('ccr-control')?.redirect).toBe('/commands/ccr')
    expect(mainLayoutRouteTitleMap['ccr-control']).toBe('nav.commands')
  })

  it('shows only one CCR command entry in the tools navigation', () => {
    const toolsSection = mainLayoutNavSections.find((section) => section.id === 'tools')
    const commandLikeItems = toolsSection?.items.filter((item) =>
      [item.to, item.labelKey].some((value) => /ccr-control|nav\.ccrControl|\/commands/.test(value)),
    )
    expect(commandLikeItems).toEqual([
      expect.objectContaining({ to: '/commands', labelKey: 'nav.commands' }),
    ])
  })

  it('keeps the root route registered as the cached dashboard', () => {
    const dashboard = byId('dashboard')
    expect(dashboard?.path).toBe('/')
    expect(dashboard?.handle?.cacheKey).toBe('DashboardView')
  })

  it('registers the cached Grok home and both management child routes', () => {
    expect(byId('grok')).toMatchObject({
      path: '/grok',
      handle: { cache: true, cacheKey: 'GrokView', depth: 1, group: 'grok' },
    })
    expect(byId('grok-auth')).toMatchObject({ path: '/grok/auth', handle: { depth: 2, group: 'grok' } })
    expect(byId('grok-profiles')).toMatchObject({
      path: '/grok/profiles',
      handle: { depth: 2, group: 'grok' },
    })
    expect(byId('grok-settings')).toMatchObject({
      path: '/grok/settings',
      handle: { depth: 2, group: 'grok' },
    })
    const modulesSection = mainLayoutNavSections.find((section) => section.id === 'modules')
    expect(modulesSection?.items).toContainEqual(
      expect.objectContaining({ to: '/grok', labelKey: 'nav.grok', icon: 'Zap' }),
    )
    expect(mainLayoutRouteTitleMap.grok).toBe('nav.grok')
    expect(mainLayoutGroupTitleMap.grok).toBe('nav.grok')
    expect(getModuleSubnavItems('grok').map((item) => item.href)).toEqual([
      '/grok',
      '/grok/auth',
      '/grok/profiles',
      '/grok/settings',
    ])
  })

  it('keeps legacy Gemini CLI routes redirected to Antigravity', () => {
    expect(byPath('/gemini-cli')?.redirect).toBe('/antigravity')
    expect(byPath('/gemini-cli/mcp')?.redirect).toBe('/antigravity/mcp')
    expect(byPath('/gemini-cli/slash-commands')?.redirect).toBe('/antigravity/slash-commands')
    expect(byPath('/gemini-cli/agents')?.redirect).toBe('/antigravity/agents')
    expect(byPath('/gemini-cli/plugins')?.redirect).toBe('/antigravity/plugins')
    expect(byPath('/gemini-cli/system-prompts')?.redirect).toBe('/antigravity/system-prompts')
  })

  it('redirects skills legacy entrypoints to the migration bridge', () => {
    expect(byPath('/skills')?.redirect).toBeUndefined()
    expect(byPath('/market')?.redirect).toBe('/skills')
    expect(byPath('/skills/add')?.redirect).toBe('/skills')
    expect(byPath('/skills/hub')?.redirect).toBe('/skills')
    expect(byPath('/skills/:platform/:name')?.redirect).toBe('/skills')
    expect(byPath('/skills-manager')?.redirect).toBe('/skills')
    expect(byPath('/skillport-manager')?.redirect).toBe('/skills')
    expect(byPath('/opencode/skills')?.redirect).toBe('/skills')
  })

  it('preserves generated platform child route paths', () => {
    const paths = flat.map((route) => route.path)
    expect(paths).toContain('/antigravity/mcp')
    expect(paths).toContain('/antigravity/agents')
    expect(paths).toContain('/antigravity/slash-commands')
    expect(paths).toContain('/antigravity/plugins')
    expect(paths).toContain('/antigravity/system-prompts')
    expect(paths).not.toContain('/droid')
  })

  it('registers the global settings route as its own navigation group', () => {
    expect(byId('settings')?.path).toBe('/settings')
    expect(byId('settings')?.handle?.group).toBe('settings')
  })

  it('hides the global background on routes with page-level decorative backgrounds', () => {
    const ids = [
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
    for (const id of ids) {
      expect(byId(id)?.handle?.hideGlobalBackground, id).toBe(true)
    }
  })

  it('matches /commands and /commands/claude on the same route with optional client', () => {
    const bare = matchRoutes(appRoutes, '/commands')
    const withClient = matchRoutes(appRoutes, '/commands/claude')
    const bareLeaf = bare?.[bare.length - 1]
    const clientLeaf = withClient?.[withClient.length - 1]
    expect(bareLeaf?.route.id).toBe('commands')
    expect(clientLeaf?.route.id).toBe('commands')
    expect(bareLeaf?.params.client).toBeUndefined()
    expect(clientLeaf?.params.client).toBe('claude')
  })
})
