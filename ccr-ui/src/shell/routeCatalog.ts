import { genericPlatformDescriptorList } from '@/config/platformDescriptors'
import type { RouteHandle } from './routeHandle'

/** 路由表条目。redirect 与页面互斥。 */
export interface CatalogEntry {
  path: string
  id?: string
  redirect?: string
  handle?: RouteHandle
}

const dash = (handle: RouteHandle): RouteHandle => handle

const children: CatalogEntry[] = [
  { path: '', id: 'dashboard', handle: dash({ cache: true, cacheKey: 'DashboardView', depth: 0 }) },
  {
    path: 'settings',
    id: 'settings',
    handle: dash({ depth: 1, group: 'settings', deferLocaleHydration: true }),
  },
  {
    path: 'claude-code',
    id: 'claude-code',
    handle: dash({ depth: 1, group: 'claude-code', hideGlobalBackground: true }),
  },
  {
    path: 'claude-code/settings',
    id: 'claude-code-settings',
    handle: dash({ depth: 2, group: 'claude-code' }),
  },
  {
    path: 'claude-code/system-prompts',
    id: 'claude-system-prompts',
    handle: dash({ depth: 2, group: 'claude-code' }),
  },
  {
    path: 'claude-code/profiles',
    id: 'claude-code-profiles',
    handle: dash({ depth: 2, group: 'claude-code' }),
  },
  {
    path: 'claude-code/auth',
    id: 'claude-code-auth',
    handle: dash({ depth: 2, group: 'claude-code' }),
  },
  { path: 'codex', id: 'codex', handle: dash({ depth: 1, group: 'codex' }) },
  {
    path: 'grok',
    id: 'grok',
    handle: dash({ cache: true, cacheKey: 'GrokView', depth: 1, group: 'grok' }),
  },
  { path: 'grok/auth', id: 'grok-auth', handle: dash({ depth: 2, group: 'grok' }) },
  { path: 'grok/profiles', id: 'grok-profiles', handle: dash({ depth: 2, group: 'grok' }) },
  { path: 'grok/settings', id: 'grok-settings', handle: dash({ depth: 2, group: 'grok' }) },
  {
    path: 'antigravity',
    id: 'antigravity',
    handle: dash({ depth: 1, group: 'gemini', hideGlobalBackground: true }),
  },
  {
    path: 'gemini-cli',
    id: 'gemini-cli',
    redirect: '/antigravity',
    handle: dash({ depth: 1, group: 'gemini', hideGlobalBackground: true }),
  },
  {
    path: 'ccr-control',
    id: 'ccr-control',
    redirect: '/commands/ccr',
    handle: dash({ depth: 1, group: 'tools' }),
  },
  {
    path: 'commands/:client?',
    id: 'commands',
    handle: dash({
      cache: true,
      cacheKey: 'CommandsView',
      stream: true,
      depth: 1,
      group: 'tools',
    }),
  },
  { path: 'converter', id: 'converter', handle: dash({ depth: 1, group: 'tools' }) },
  {
    path: 'sync',
    id: 'sync',
    handle: dash({ depth: 1, group: 'tools', hideGlobalBackground: true }),
  },
  {
    path: 'configs',
    id: 'configs',
    handle: dash({
      cache: true,
      cacheKey: 'ConfigsView',
      depth: 1,
      group: 'config',
      hideGlobalBackground: true,
    }),
  },
  { path: 'stats', redirect: '/usage' },
  { path: 'budget', id: 'budget', handle: dash({ depth: 1, group: 'data' }) },
  { path: 'pricing', id: 'pricing', handle: dash({ depth: 1, group: 'data' }) },
  {
    path: 'usage',
    id: 'usage',
    handle: dash({
      cache: true,
      cacheKey: 'UsageDashboardView',
      depth: 1,
      group: 'data',
      hideGlobalBackground: true,
    }),
  },
  { path: 'monitoring', id: 'monitoring', handle: dash({ depth: 1, group: 'data' }) },
  { path: 'sessions', id: 'sessions', redirect: '/monitoring' },
  { path: 'mcp', id: 'mcp', redirect: '/mcp-manager' },
  { path: 'mcp/unified', id: 'mcp-unified', redirect: '/mcp-manager' },
  { path: 'mcp-manager', id: 'mcp-manager', handle: dash({ depth: 1, group: 'config' }) },
  { path: 'slash-commands', id: 'slash-commands', handle: dash({ depth: 1, group: 'config' }) },
  { path: 'agents', id: 'agents', handle: dash({ depth: 1, group: 'config' }) },
  { path: 'agents/:name', id: 'agent-detail', handle: dash({ depth: 2, group: 'config' }) },
  { path: 'skills', id: 'skills', handle: dash({ depth: 1, group: 'config' }) },
  { path: 'skills-manager', id: 'skills-manager', redirect: '/skills' },
  { path: 'skillport-manager', id: 'skillport-manager', redirect: '/skills' },
  { path: 'skills/add', id: 'skills-add', redirect: '/skills' },
  { path: 'skills/hub', redirect: '/skills' },
  { path: 'skills/:platform/:name', redirect: '/skills' },
  { path: 'market', id: 'market', redirect: '/skills' },
  { path: 'plugins', id: 'plugins', handle: dash({ depth: 1, group: 'config' }) },
  { path: 'hooks', id: 'hooks', handle: dash({ depth: 1, group: 'config' }) },
  { path: 'output-styles', id: 'output-styles', handle: dash({ depth: 1, group: 'config' }) },
  { path: 'statusline', id: 'statusline', handle: dash({ depth: 1, group: 'config' }) },
  {
    path: 'checkin/manage/:accountId',
    id: 'checkin-account-dashboard',
    handle: dash({ depth: 2, group: 'tools' }),
  },
  { path: 'checkin', id: 'checkin', handle: dash({ depth: 1, group: 'tools' }) },
  { path: 'codex/mcp', id: 'codex-mcp', handle: dash({ depth: 2, group: 'codex' }) },
  { path: 'codex/profiles', id: 'codex-profiles', handle: dash({ depth: 2, group: 'codex' }) },
  { path: 'codex/agents', id: 'codex-agents', handle: dash({ depth: 2, group: 'codex' }) },
  { path: 'codex/sessions', id: 'codex-sessions', handle: dash({ depth: 2, group: 'codex' }) },
  {
    path: 'codex/slash-commands',
    id: 'codex-slash-commands',
    handle: dash({ depth: 2, group: 'codex' }),
  },
  { path: 'codex/auth', id: 'codex-auth', handle: dash({ depth: 2, group: 'codex' }) },
  { path: 'codex/settings', id: 'codex-settings', handle: dash({ depth: 2, group: 'codex' }) },
  {
    path: 'codex/system-prompts',
    id: 'codex-system-prompts',
    handle: dash({ depth: 2, group: 'codex' }),
  },
  {
    path: 'antigravity/slash-commands',
    id: 'gemini-slash-commands',
    handle: dash({ depth: 2, group: 'gemini' }),
  },
  {
    path: 'gemini-cli/slash-commands',
    redirect: '/antigravity/slash-commands',
    handle: dash({ depth: 2, group: 'gemini' }),
  },
  {
    path: 'gemini-cli/mcp',
    redirect: '/antigravity/mcp',
    handle: dash({ depth: 2, group: 'gemini' }),
  },
  {
    path: 'gemini-cli/agents',
    redirect: '/antigravity/agents',
    handle: dash({ depth: 2, group: 'gemini' }),
  },
  {
    path: 'gemini-cli/plugins',
    redirect: '/antigravity/plugins',
    handle: dash({ depth: 2, group: 'gemini' }),
  },
  {
    path: 'antigravity/system-prompts',
    id: 'gemini-system-prompts',
    handle: dash({ depth: 2, group: 'gemini' }),
  },
  {
    path: 'gemini-cli/system-prompts',
    redirect: '/antigravity/system-prompts',
    handle: dash({ depth: 2, group: 'gemini' }),
  },
  {
    path: 'opencode',
    id: 'opencode',
    handle: dash({ depth: 1, group: 'opencode', hideGlobalBackground: true }),
  },
  {
    path: 'opencode/providers',
    id: 'opencode-providers',
    handle: dash({ depth: 2, group: 'opencode', hideGlobalBackground: true }),
  },
  {
    path: 'opencode/mcp',
    id: 'opencode-mcp',
    handle: dash({ depth: 2, group: 'opencode', hideGlobalBackground: true }),
  },
  {
    path: 'opencode/agents',
    id: 'opencode-agents',
    handle: dash({ depth: 2, group: 'opencode', hideGlobalBackground: true }),
  },
  {
    path: 'opencode/commands',
    id: 'opencode-commands',
    handle: dash({ depth: 2, group: 'opencode', hideGlobalBackground: true }),
  },
  {
    path: 'opencode/skills',
    id: 'opencode-skills',
    redirect: '/skills',
    handle: dash({ depth: 2, group: 'opencode', hideGlobalBackground: true }),
  },
  {
    path: 'opencode/plugins',
    id: 'opencode-plugins',
    handle: dash({ depth: 2, group: 'opencode', hideGlobalBackground: true }),
  },
  {
    path: 'opencode/settings',
    id: 'opencode-settings',
    handle: dash({ depth: 2, group: 'opencode', hideGlobalBackground: true }),
  },
  {
    path: 'opencode/system-prompts',
    id: 'opencode-system-prompts',
    handle: dash({ depth: 2, group: 'opencode', hideGlobalBackground: true }),
  },
  { path: 'wsl', id: 'wsl-management', handle: dash({ depth: 1, group: 'environment' }) },
  { path: 'ssh', id: 'ssh-management', handle: dash({ depth: 1, group: 'environment' }) },
]

const genericChildren: CatalogEntry[] = genericPlatformDescriptorList.flatMap((platform) => {
  const groupHandle = dash({ depth: 2, group: platform.id })
  const routes: CatalogEntry[] = [
    { path: `${platform.rootPath}/${platform.mcp.path}`, id: platform.mcp.name, handle: groupHandle },
    {
      path: `${platform.rootPath}/${platform.agents.path}`,
      id: platform.agents.name,
      handle: groupHandle,
    },
  ]
  if (platform.plugins) {
    routes.push({
      path: `${platform.rootPath}/${platform.plugins.path}`,
      id: platform.plugins.name,
      handle: groupHandle,
    })
  }
  return routes
})

export const layoutChildCatalog: CatalogEntry[] = [...children, ...genericChildren]

export const trayCatalog: CatalogEntry = {
  path: '/tray/codex',
  id: 'codex-tray-panel',
  handle: dash({ depth: 0, group: 'codex-tray', hideGlobalBackground: true }),
}

export const layoutCatalog: CatalogEntry = { path: '/' }

export interface FlatCatalogRoute {
  path: string
  id?: string
  redirect?: string
  handle?: RouteHandle
}

const joinPath = (parent: string, child: string): string => {
  if (child.startsWith('/')) return child
  if (!child) return parent || '/'
  if (!parent || parent === '/') return `/${child}`
  return `${parent.replace(/\/$/, '')}/${child}`
}

/** 扁平化后的 75 条路径记录（含布局父级与全部 children）。 */
export function flattenCatalog(): FlatCatalogRoute[] {
  const layoutPath = joinPath('', layoutCatalog.path)
  const rows: FlatCatalogRoute[] = [
    {
      path: joinPath('', trayCatalog.path),
      id: trayCatalog.id,
      handle: trayCatalog.handle,
    },
    { path: layoutPath, id: layoutCatalog.id, handle: layoutCatalog.handle },
  ]
  for (const entry of layoutChildCatalog) {
    rows.push({
      path: joinPath(layoutPath, entry.path),
      id: entry.id,
      redirect: entry.redirect,
      handle: entry.handle,
    })
  }
  return rows
}
