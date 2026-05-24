import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'
import { genericPlatformDescriptorList } from '@/config/platformDescriptors'
import { initPerfTelemetry, recordRouteTiming } from '@/utils/perfTelemetry'

// RouteMeta 类型扩展
declare module 'vue-router' {
  interface RouteMeta {
    cache?: boolean
    /**
     * keep-alive include 用的组件 name。
     *
     * Vue `<keep-alive :include>` 匹配的是组件自身的 name（`<script setup>`
     * 从文件名推导），而不是 route.name。路由名是 kebab-case，组件名通常是
     * PascalCase，两者不一致。此字段让 router 成为缓存列表的唯一事实源，
     * MainLayout 从 `meta.cache === true` 的路由派生 `cacheKey` 数组。
     *
     * 规则：只要 `meta.cache === true`，就必须同时给出 `cacheKey`。
     */
    cacheKey?: string
    hideGlobalBackground?: boolean
    stream?: boolean
    /** 路由层级深度: 0=首页, 1=顶级页面, 2=子页面 */
    depth?: number
    /** 路由分组: 用于判断是否是同一平台/模块内的导航 */
    group?: string
    /** 隐藏侧边栏，启用全宽布局模式 */
    hideSidebar?: boolean
  }
}

const genericPlatformRoutes: RouteRecordRaw[] = genericPlatformDescriptorList.flatMap(
  (platform) => {
    const routes: RouteRecordRaw[] = [
      {
        path: `${platform.rootPath}/${platform.mcp.path}`,
        name: platform.mcp.name,
        component: () => import('@/views/generic/PlatformMcpView.vue'),
        props: { platform: platform.id },
        meta: { depth: 2, group: platform.id },
      },
      {
        path: `${platform.rootPath}/${platform.agents.path}`,
        name: platform.agents.name,
        component: () => import('@/views/generic/AgentsView.vue'),
        props: { module: platform.agents.module },
        meta: { depth: 2, group: platform.id },
      },
    ]

    if (platform.plugins) {
      routes.push({
        path: `${platform.rootPath}/${platform.plugins.path}`,
        name: platform.plugins.name,
        component: () => import('@/views/generic/PlatformPluginsView.vue'),
        props: { platform: platform.id },
        meta: { depth: 2, group: platform.id },
      })
    }

    return routes
  }
)

const routes: RouteRecordRaw[] = [
  {
    path: '/tray/codex',
    name: 'codex-tray-panel',
    component: () => import('@/views/tray/CodexTrayPanelView.vue'),
    meta: { depth: 0, group: 'codex-tray', hideGlobalBackground: true },
  },
  {
    path: '/',
    component: () => import('@/components/MainLayout.vue'),
    children: [
      {
        path: '',
        name: 'dashboard',
        component: () => import('@/views/DashboardView.vue'),
        meta: { cache: true, cacheKey: 'DashboardView', depth: 0 },
      },
      {
        path: 'settings',
        name: 'settings',
        component: () => import('@/views/AppSettingsView.vue'),
        meta: { depth: 1, group: 'settings' },
      },
      // 主要模块 (depth: 1)
      {
        path: 'claude-code',
        name: 'claude-code',
        component: () => import('@/views/ClaudeCodeView.vue'),
        meta: { depth: 1, group: 'claude-code', hideGlobalBackground: true },
      },
      {
        path: 'claude-code/settings',
        name: 'claude-code-settings',
        component: () => import('@/views/ClaudeCodeSettingsView.vue'),
        meta: { depth: 2, group: 'claude-code' },
      },
      {
        path: 'claude-code/profiles',
        name: 'claude-code-profiles',
        component: () => import('@/views/ClaudeCodeProfilesView.vue'),
        meta: { depth: 2, group: 'claude-code' },
      },
      {
        path: 'claude-code/auth',
        name: 'claude-code-auth',
        component: () => import('@/views/ClaudeAuthView.vue'),
        meta: { depth: 2, group: 'claude-code' },
      },
      {
        path: 'codex',
        name: 'codex',
        component: () => import('@/views/CodexView.vue'),
        meta: { cache: true, cacheKey: 'CodexView', depth: 1, group: 'codex' },
      },
      {
        path: 'antigravity',
        name: 'antigravity',
        component: () => import('@/views/GeminiCliView.vue'),
        meta: { depth: 1, group: 'gemini', hideGlobalBackground: true },
      },
      {
        path: 'gemini-cli',
        name: 'gemini-cli',
        redirect: '/antigravity',
        meta: { depth: 1, group: 'gemini', hideGlobalBackground: true },
      },
      // 工具中心 (depth: 1, group: 'tools')
      {
        path: 'ccr-control',
        name: 'ccr-control',
        redirect: '/commands/ccr',
        meta: { depth: 1, group: 'tools' },
      },
      {
        path: 'commands/:client?',
        name: 'commands',
        component: () => import('@/views/CommandsView.vue'),
        meta: { cache: true, cacheKey: 'CommandsView', stream: true, depth: 1, group: 'tools' },
      },
      {
        path: 'converter',
        name: 'converter',
        component: () => import('@/views/ConverterView.vue'),
        meta: { depth: 1, group: 'tools' },
      },
      {
        path: 'sync',
        name: 'sync',
        component: () => import('@/views/SyncView.vue'),
        meta: { depth: 1, group: 'tools', hideGlobalBackground: true },
      },
      // 配置组 (depth: 1, group: 'config')
      {
        path: 'configs',
        name: 'configs',
        component: () => import('@/views/ConfigsView.vue'),
        meta: {
          cache: true,
          cacheKey: 'ConfigsView',
          depth: 1,
          group: 'config',
          hideGlobalBackground: true,
        },
      },
      {
        path: 'stats',
        redirect: '/usage',
      },
      // 数据组 (depth: 1, group: 'data')
      {
        path: 'budget',
        name: 'budget',
        component: () => import('@/views/BudgetView.vue'),
        meta: { depth: 1, group: 'data' },
      },
      {
        path: 'pricing',
        name: 'pricing',
        component: () => import('@/views/PricingView.vue'),
        meta: { depth: 1, group: 'data' },
      },
      {
        path: 'usage',
        name: 'usage',
        component: () => import('@/views/UsageDashboardView.vue'),
        meta: {
          cache: true,
          cacheKey: 'UsageDashboardView',
          depth: 1,
          group: 'data',
          hideGlobalBackground: true,
        },
      },
      {
        path: 'monitoring',
        name: 'monitoring',
        component: () => import('@/views/MonitoringView.vue'),
        meta: { depth: 1, group: 'data' },
      },
      {
        path: 'sessions',
        name: 'sessions',
        component: () => import('@/views/SessionsView.vue'),
        meta: { depth: 1, group: 'data' },
      },
      // MCP 管理
      {
        path: 'mcp',
        name: 'mcp',
        redirect: '/mcp-manager',
      },
      {
        path: 'mcp/unified',
        name: 'mcp-unified',
        redirect: '/mcp-manager',
      },
      {
        path: 'mcp-manager',
        name: 'mcp-manager',
        component: () => import('@/views/mcp/McpManagerView.vue'),
        meta: { cache: true, cacheKey: 'McpManagerView', depth: 1, group: 'config' },
      },
      {
        path: 'slash-commands',
        name: 'slash-commands',
        component: () => import('@/views/SlashCommandsView.vue'),
        meta: { depth: 1, group: 'config' },
      },
      {
        path: 'agents',
        name: 'agents',
        component: () => import('@/views/generic/AgentsView.vue'),
        props: { module: 'agents' },
        meta: { depth: 1, group: 'config' },
      },
      {
        path: 'agents/:name',
        name: 'agent-detail',
        component: () => import('@/views/generic/AgentDetailView.vue'),
        meta: { depth: 2, group: 'config' },
      },
      // Skills migration bridge
      {
        path: 'skills',
        name: 'skills',
        component: () => import('@/views/SkillsMigrationView.vue'),
        meta: { depth: 1, group: 'config' },
      },
      {
        path: 'skills-manager',
        name: 'skills-manager',
        redirect: '/skills',
      },
      {
        path: 'skillport-manager',
        name: 'skillport-manager',
        redirect: '/skills',
      },
      {
        path: 'skills/add',
        name: 'skills-add',
        redirect: '/skills',
      },
      {
        path: 'skills/hub',
        redirect: '/skills',
      },
      {
        path: 'skills/:platform/:name',
        redirect: '/skills',
      },
      {
        path: 'market',
        name: 'market',
        redirect: '/skills',
      },
      // 配置组 (depth: 1, group: 'config')
      {
        path: 'plugins',
        name: 'plugins',
        component: () => import('@/views/PluginsView.vue'),
        meta: { depth: 1, group: 'config' },
      },
      {
        path: 'hooks',
        name: 'hooks',
        component: () => import('@/views/HooksView.vue'),
        meta: { depth: 1, group: 'config' },
      },
      {
        path: 'output-styles',
        name: 'output-styles',
        component: () => import('@/views/OutputStylesView.vue'),
        meta: { depth: 1, group: 'config' },
      },
      {
        path: 'statusline',
        name: 'statusline',
        component: () => import('@/views/StatuslineView.vue'),
        meta: { depth: 1, group: 'config' },
      },
      // 工具组 - checkin (depth: 1, group: 'tools')
      {
        path: 'checkin/manage/:accountId',
        name: 'checkin-account-dashboard',
        component: () => import('@/views/checkin/CheckinAccountDashboardView.vue'),
        props: true,
        meta: { depth: 2, group: 'tools' },
      },
      {
        path: 'checkin',
        name: 'checkin',
        component: () => import('@/views/CheckinView.vue'),
        meta: { depth: 1, group: 'tools' },
      },
      // Codex 子页面 (depth: 2, group: 'codex')
      {
        path: 'codex/mcp',
        name: 'codex-mcp',
        component: () => import('@/views/CodexMcpView.vue'),
        meta: { cache: true, cacheKey: 'CodexMcpView', depth: 2, group: 'codex' },
      },
      {
        path: 'codex/profiles',
        name: 'codex-profiles',
        component: () => import('@/views/CodexProfilesView.vue'),
        meta: { cache: true, cacheKey: 'CodexProfilesView', depth: 2, group: 'codex' },
      },
      {
        path: 'codex/agents',
        name: 'codex-agents',
        component: () => import('@/views/codex/CodexAgentsView.vue'),
        meta: { depth: 2, group: 'codex' },
      },
      {
        path: 'codex/sessions',
        name: 'codex-sessions',
        component: () => import('@/views/CodexSessionsView.vue'),
        meta: { depth: 2, group: 'codex' },
      },
      {
        path: 'codex/slash-commands',
        name: 'codex-slash-commands',
        component: () => import('@/views/CodexSlashCommandsView.vue'),
        meta: { depth: 2, group: 'codex' },
      },
      {
        path: 'codex/auth',
        name: 'codex-auth',
        component: () => import('@/views/CodexAuthView.vue'),
        meta: { cache: true, cacheKey: 'CodexAuthView', depth: 2, group: 'codex' },
      },
      {
        path: 'codex/settings',
        name: 'codex-settings',
        component: () => import('@/views/CodexSettingsView.vue'),
        meta: { depth: 2, group: 'codex' },
      },
      // Antigravity CLI 子页面 (depth: 2, group: 'gemini')
      {
        path: 'antigravity/slash-commands',
        name: 'gemini-slash-commands',
        component: () => import('@/views/GeminiSlashCommandsView.vue'),
        meta: { depth: 2, group: 'gemini' },
      },
      {
        path: 'gemini-cli/slash-commands',
        redirect: '/antigravity/slash-commands',
        meta: { depth: 2, group: 'gemini' },
      },
      {
        path: 'gemini-cli/mcp',
        redirect: '/antigravity/mcp',
        meta: { depth: 2, group: 'gemini' },
      },
      {
        path: 'gemini-cli/agents',
        redirect: '/antigravity/agents',
        meta: { depth: 2, group: 'gemini' },
      },
      {
        path: 'gemini-cli/plugins',
        redirect: '/antigravity/plugins',
        meta: { depth: 2, group: 'gemini' },
      },
      // OpenCode 子页面 (depth: 2, group: 'opencode')
      {
        path: 'opencode',
        name: 'opencode',
        component: () => import('@/views/OpenCodeView.vue'),
        meta: { depth: 1, group: 'opencode', hideGlobalBackground: true },
      },
      {
        path: 'opencode/providers',
        name: 'opencode-providers',
        component: () => import('@/views/OpenCodeProvidersView.vue'),
        meta: { depth: 2, group: 'opencode', hideGlobalBackground: true },
      },
      {
        path: 'opencode/mcp',
        name: 'opencode-mcp',
        component: () => import('@/views/OpenCodeMcpView.vue'),
        meta: { depth: 2, group: 'opencode', hideGlobalBackground: true },
      },
      {
        path: 'opencode/agents',
        name: 'opencode-agents',
        component: () => import('@/views/OpenCodeAgentsView.vue'),
        meta: { depth: 2, group: 'opencode', hideGlobalBackground: true },
      },
      {
        path: 'opencode/commands',
        name: 'opencode-commands',
        component: () => import('@/views/OpenCodeCommandsView.vue'),
        meta: { depth: 2, group: 'opencode', hideGlobalBackground: true },
      },
      {
        path: 'opencode/skills',
        name: 'opencode-skills',
        redirect: '/skills',
        meta: { depth: 2, group: 'opencode', hideGlobalBackground: true },
      },
      {
        path: 'opencode/plugins',
        name: 'opencode-plugins',
        component: () => import('@/views/OpenCodePluginsView.vue'),
        meta: { depth: 2, group: 'opencode', hideGlobalBackground: true },
      },
      {
        path: 'opencode/settings',
        name: 'opencode-settings',
        component: () => import('@/views/OpenCodeSettingsView.vue'),
        meta: { depth: 2, group: 'opencode', hideGlobalBackground: true },
      },
      // 环境管理 (depth: 1, group: 'environment')
      {
        path: 'wsl',
        name: 'wsl-management',
        component: () => import('@/views/WslManagementView.vue'),
        meta: { depth: 1, group: 'environment' },
      },
      {
        path: 'ssh',
        name: 'ssh-management',
        component: () => import('@/views/SshManagementView.vue'),
        meta: { depth: 1, group: 'environment' },
      },
      ...genericPlatformRoutes,
    ],
  },
]

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes,
  scrollBehavior() {
    // 始终滚动到顶部
    return { top: 0 }
  },
})

const perfEnabled = initPerfTelemetry()

if (perfEnabled) {
  let navStartMs: number | null = null
  let navFrom = ''
  let navTo = ''

  router.beforeEach((to, from) => {
    if (typeof performance !== 'undefined' && typeof performance.now === 'function') {
      navStartMs = performance.now()
    } else {
      navStartMs = Date.now()
    }

    navFrom = from.fullPath ?? String(from.path ?? '')
    navTo = to.fullPath ?? String(to.path ?? '')
  })

  router.afterEach((to, from, failure) => {
    if (failure) {
      navStartMs = null
      navFrom = ''
      navTo = ''
      return
    }

    if (navStartMs === null) return

    const endMs =
      typeof performance !== 'undefined' && typeof performance.now === 'function'
        ? performance.now()
        : Date.now()

    recordRouteTiming(
      navFrom || (from.fullPath ?? String(from.path ?? '')),
      navTo || (to.fullPath ?? String(to.path ?? '')),
      endMs - navStartMs
    )
    navStartMs = null
    navFrom = ''
    navTo = ''
  })
}

export default router

/**
 * 扁平化路由树，收集所有 `meta.cache === true` 且带 `cacheKey` 的组件名。
 *
 * 用于 `<keep-alive :include>` —— Vue keep-alive 匹配组件自身的 name
 * （`<script setup>` 从文件名推导），路由 name 是 kebab-case 不能直接用。
 * 此函数保证 cachedViews 来自路由单一事实源，不再依赖视图硬编码数组。
 */
export function collectCachedComponentNames(
  source: ReadonlyArray<RouteRecordRaw> = routes
): string[] {
  const names = new Set<string>()
  const walk = (records: ReadonlyArray<RouteRecordRaw>): void => {
    for (const record of records) {
      if (record.meta?.cache === true && typeof record.meta.cacheKey === 'string') {
        names.add(record.meta.cacheKey)
      }
      if (record.children && record.children.length > 0) {
        walk(record.children)
      }
    }
  }
  walk(source)
  return Array.from(names)
}
