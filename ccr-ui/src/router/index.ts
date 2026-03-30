import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'
import { genericPlatformDescriptorList } from '@/config/platformDescriptors'
import { initPerfTelemetry, recordRouteTiming } from '@/utils/perfTelemetry'

// RouteMeta 类型扩展
declare module 'vue-router' {
  interface RouteMeta {
    cache?: boolean
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
    path: '/',
    component: () => import('@/components/MainLayout.vue'),
    children: [
      {
        path: '',
        name: 'home',
        component: () => import('@/views/HomeView.vue'),
        meta: { cache: true, depth: 0 },
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
        path: 'codex',
        name: 'codex',
        component: () => import('@/views/CodexView.vue'),
        meta: { depth: 1, group: 'codex' },
      },
      {
        path: 'gemini-cli',
        name: 'gemini-cli',
        component: () => import('@/views/GeminiCliView.vue'),
        meta: { depth: 1, group: 'gemini', hideGlobalBackground: true },
      },
      {
        path: 'qwen',
        name: 'qwen',
        component: () => import('@/views/QwenView.vue'),
        meta: { depth: 1, group: 'qwen', hideGlobalBackground: true },
      },
      {
        path: 'qoder',
        name: 'qoder',
        component: () => import('@/views/QoderView.vue'),
        meta: { depth: 1, group: 'qoder', hideGlobalBackground: true },
      },
      {
        path: 'droid',
        name: 'droid',
        component: () => import('@/views/DroidView.vue'),
        meta: { depth: 1, group: 'droid', hideGlobalBackground: true },
      },
      // 工具中心 (depth: 1, group: 'tools')
      {
        path: 'ccr-control',
        name: 'ccr-control',
        component: () => import('@/views/CcrControlView.vue'),
        meta: { cache: true, depth: 1, group: 'tools' },
      },
      {
        path: 'commands/:client?',
        name: 'commands',
        component: () => import('@/views/CommandsView.vue'),
        meta: { cache: true, stream: true, depth: 1, group: 'tools' },
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
        meta: { cache: true, depth: 1, group: 'config', hideGlobalBackground: true },
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
        meta: { cache: true, depth: 1, group: 'data', hideGlobalBackground: true },
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
        component: () => import('@/views/McpView.vue'),
        meta: { depth: 1, group: 'config' },
      },
      {
        path: 'mcp/unified',
        name: 'mcp-unified',
        component: () => import('@/views/mcp/UnifiedMcpView.vue'),
        meta: { cache: true, hideSidebar: true, depth: 1, group: 'config' },
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
      // Skills Hub (depth: 1, group: 'skills')
      {
        path: 'skills',
        name: 'skills',
        component: () => import('@/views/skills/UnifiedSkillsView.vue'),
        meta: { cache: true, depth: 1, group: 'skills' },
      },
      {
        path: 'skills/add',
        name: 'skills-add',
        redirect: '/skills?tab=marketplace',
      },
      {
        path: 'skills/hub',
        redirect: '/skills?tab=marketplace',
      },
      {
        path: 'skills/:platform/:name',
        redirect: '/skills',
      },
      {
        path: 'market',
        name: 'market',
        redirect: '/skills?tab=marketplace',
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
        meta: { depth: 2, group: 'codex' },
      },
      {
        path: 'codex/profiles',
        name: 'codex-profiles',
        component: () => import('@/views/CodexProfilesView.vue'),
        meta: { depth: 2, group: 'codex' },
      },
      {
        path: 'codex/agents',
        name: 'codex-agents',
        component: () => import('@/views/generic/AgentsView.vue'),
        props: { module: 'codex' },
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
        meta: { depth: 2, group: 'codex' },
      },
      {
        path: 'codex/settings',
        name: 'codex-settings',
        component: () => import('@/views/CodexSettingsView.vue'),
        meta: { depth: 2, group: 'codex' },
      },
      // Gemini CLI 子页面 (depth: 2, group: 'gemini')
      {
        path: 'gemini-cli/slash-commands',
        name: 'gemini-slash-commands',
        component: () => import('@/views/GeminiSlashCommandsView.vue'),
        meta: { depth: 2, group: 'gemini' },
      },
      // Qwen 子页面 (depth: 2, group: 'qwen')
      {
        path: 'qwen/slash-commands',
        name: 'qwen-slash-commands',
        component: () => import('@/views/QwenSlashCommandsView.vue'),
        meta: { depth: 2, group: 'qwen' },
      },
      // Qoder 子页面 (depth: 2, group: 'qoder')
      {
        path: 'qoder/commands',
        name: 'qoder-commands',
        component: () => import('@/views/QoderCommandsView.vue'),
        meta: { depth: 2, group: 'qoder' },
      },
      {
        path: 'qoder/hooks',
        name: 'qoder-hooks',
        component: () => import('@/views/QoderHooksView.vue'),
        meta: { depth: 2, group: 'qoder' },
      },
      // Droid 子页面 (depth: 2, group: 'droid')
      {
        path: 'droid/slash-commands',
        name: 'droid-slash-commands',
        component: () => import('@/views/DroidSlashCommandsView.vue'),
        meta: { depth: 2, group: 'droid' },
      },
      {
        path: 'droid/plugins',
        name: 'droid-plugins',
        component: () => import('@/views/DroidPluginsView.vue'),
        meta: { depth: 2, group: 'droid' },
      },
      {
        path: 'droid/models',
        name: 'droid-models',
        component: () => import('@/views/DroidModelsView.vue'),
        meta: { depth: 2, group: 'droid' },
      },
      {
        path: 'droid/profiles',
        name: 'droid-profiles',
        component: () => import('@/views/DroidProfilesView.vue'),
        meta: { depth: 2, group: 'droid' },
      },
      {
        path: 'droid/droids',
        name: 'droid-droids',
        component: () => import('@/views/DroidDroidsView.vue'),
        meta: { depth: 2, group: 'droid' },
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
        path: 'opencode/plugins',
        name: 'opencode-plugins',
        component: () => import('@/views/OpenCodePluginsView.vue'),
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

    const endMs = typeof performance !== 'undefined' && typeof performance.now === 'function'
      ? performance.now()
      : Date.now()

    recordRouteTiming(
      navFrom || (from.fullPath ?? String(from.path ?? '')),
      navTo || (to.fullPath ?? String(to.path ?? '')),
      endMs - navStartMs,
    )
    navStartMs = null
    navFrom = ''
    navTo = ''
  })
}

export default router
