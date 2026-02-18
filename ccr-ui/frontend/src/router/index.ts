import { createRouter, createWebHistory } from 'vue-router'

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
  }
}

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      component: () => import('@/components/MainLayout.vue'),
      children: [
        {
          path: '',
          name: 'home',
          component: () => import('@/views/HomeView.vue'),
          meta: { cache: true, depth: 0 }
        },
        // 主要模块 (depth: 1)
        {
          path: 'claude-code',
          name: 'claude-code',
          component: () => import('@/views/ClaudeCodeView.vue'),
          meta: { hideGlobalBackground: true, depth: 1, group: 'claude-code' }
        },
        {
          path: 'claude-code/settings',
          name: 'claude-code-settings',
          component: () => import('@/views/ClaudeCodeSettingsView.vue'),
          meta: { hideGlobalBackground: true, depth: 2, group: 'claude-code' }
        },
        {
          path: 'codex',
          name: 'codex',
          component: () => import('@/views/CodexView.vue'),
          meta: { hideGlobalBackground: true, depth: 1, group: 'codex' }
        },
        {
          path: 'gemini-cli',
          name: 'gemini-cli',
          component: () => import('@/views/GeminiCliView.vue'),
          meta: { hideGlobalBackground: true, depth: 1, group: 'gemini' }
        },
        {
          path: 'qwen',
          name: 'qwen',
          component: () => import('@/views/QwenView.vue'),
          meta: { hideGlobalBackground: true, depth: 1, group: 'qwen' }
        },
        {
          path: 'iflow',
          name: 'iflow',
          component: () => import('@/views/IflowView.vue'),
          meta: { hideGlobalBackground: true, depth: 1, group: 'iflow' }
        },
        {
          path: 'droid',
          name: 'droid',
          component: () => import('@/views/DroidView.vue'),
          meta: { hideGlobalBackground: true, depth: 1, group: 'droid' }
        },
        // 工具中心 (depth: 1, group: 'tools')
        {
          path: 'ccr-control',
          name: 'ccr-control',
          component: () => import('@/views/CcrControlView.vue'),
          meta: { cache: true, depth: 1, group: 'tools' }
        },
        {
          path: 'commands/:client?',
          name: 'commands',
          component: () => import('@/views/CommandsView.vue'),
          meta: { cache: true, stream: true, depth: 1, group: 'tools' }
        },
        {
          path: 'converter',
          name: 'converter',
          component: () => import('@/views/ConverterView.vue'),
          meta: { depth: 1, group: 'tools' }
        },
        {
          path: 'sync',
          name: 'sync',
          component: () => import('@/views/SyncView.vue'),
          meta: { hideGlobalBackground: true, depth: 1, group: 'tools' }
        },
        // 配置组 (depth: 1, group: 'config')
        {
          path: 'configs',
          name: 'configs',
          component: () => import('@/views/ConfigsView.vue'),
          meta: { cache: true, hideGlobalBackground: true, depth: 1, group: 'config' }
        },
        {
          path: 'stats',
          redirect: '/usage'
        },
        // 数据组 (depth: 1, group: 'data')
        {
          path: 'budget',
          name: 'budget',
          component: () => import('@/views/BudgetView.vue'),
          meta: { depth: 1, group: 'data' }
        },
        {
          path: 'pricing',
          name: 'pricing',
          component: () => import('@/views/PricingView.vue'),
          meta: { depth: 1, group: 'data' }
        },
        {
          path: 'usage',
          name: 'usage',
          component: () => import('@/views/UsageDashboardView.vue'),
          meta: { cache: true, hideGlobalBackground: true, depth: 1, group: 'data' }
        },
        {
          path: 'monitoring',
          name: 'monitoring',
          component: () => import('@/views/MonitoringView.vue'),
          meta: { depth: 1, group: 'data' }
        },
        // 配置组子页面 (depth: 1, group: 'config')
        {
          path: 'mcp',
          name: 'mcp',
          component: () => import('@/views/McpView.vue'),
          meta: { depth: 1, group: 'config' }
        },
        {
          path: 'slash-commands',
          name: 'slash-commands',
          component: () => import('@/views/SlashCommandsView.vue'),
          meta: { depth: 1, group: 'config' }
        },
        {
          path: 'agents',
          name: 'agents',
          component: () => import('@/views/generic/AgentsView.vue'),
          props: { module: 'agents' },
          meta: { depth: 1, group: 'config' }
        },
        {
          path: 'agents/:name',
          name: 'agent-detail',
          component: () => import('@/views/generic/AgentDetailView.vue'),
          meta: { depth: 2, group: 'config' }
        },
        // Skills Hub (depth: 1, group: 'skills')
        {
          path: 'skills',
          name: 'skills',
          component: () => import('@/views/skills/UnifiedSkillsView.vue'),
          meta: { cache: true, hideGlobalBackground: true, depth: 1, group: 'skills' }
        },
        {
          path: 'skills/add',
          name: 'skills-add',
          component: () => import('@/views/skills/AddSkillView.vue'),
          meta: { depth: 2, group: 'skills' }
        },
        {
          path: 'skills/hub',
          redirect: '/skills'
        },
        {
          path: 'skills/:platform/:name',
          name: 'skill-detail',
          component: () => import('@/views/generic/SkillDetailView.vue'),
          props: true,
          meta: { depth: 2, group: 'skills' }
        },
        {
          path: 'market',
          name: 'market',
          component: () => import('@/views/generic/MarketView.vue'),
          meta: { depth: 1, group: 'skills' }
        },
        // 配置组 (depth: 1, group: 'config')
        {
          path: 'plugins',
          name: 'plugins',
          component: () => import('@/views/PluginsView.vue'),
          meta: { depth: 1, group: 'config' }
        },
        {
          path: 'sessions',
          name: 'sessions',
          component: () => import('@/views/SessionsView.vue'),
          meta: { depth: 1, group: 'config' }
        },
        {
          path: 'hooks',
          name: 'hooks',
          component: () => import('@/views/HooksView.vue'),
          meta: { depth: 1, group: 'config' }
        },
        {
          path: 'output-styles',
          name: 'output-styles',
          component: () => import('@/views/OutputStylesView.vue'),
          meta: { depth: 1, group: 'config' }
        },
        {
          path: 'statusline',
          name: 'statusline',
          component: () => import('@/views/StatuslineView.vue'),
          meta: { depth: 1, group: 'config' }
        },
        {
          path: 'provider-health',
          name: 'provider-health',
          component: () => import('@/views/ProviderHealthView.vue'),
          meta: { depth: 1, group: 'config' }
        },
        // 工具组 - checkin (depth: 1, group: 'tools')
        {
          path: 'checkin/manage/:accountId',
          name: 'checkin-account-dashboard',
          component: () => import('@/views/checkin/CheckinAccountDashboardView.vue'),
          props: true,
          meta: { depth: 2, group: 'tools' }
        },
        {
          path: 'checkin',
          name: 'checkin',
          component: () => import('@/views/CheckinView.vue'),
          meta: { depth: 1, group: 'tools' }
        },
        // Codex 子页面 (depth: 2, group: 'codex')
        {
          path: 'codex/mcp',
          name: 'codex-mcp',
          component: () => import('@/views/CodexMcpView.vue'),
          meta: { hideGlobalBackground: true, depth: 2, group: 'codex' }
        },
        {
          path: 'codex/profiles',
          name: 'codex-profiles',
          component: () => import('@/views/CodexProfilesView.vue'),
          meta: { hideGlobalBackground: true, depth: 2, group: 'codex' }
        },
        {
          path: 'codex/slash-commands',
          name: 'codex-slash-commands',
          component: () => import('@/views/CodexSlashCommandsView.vue'),
          meta: { hideGlobalBackground: true, depth: 2, group: 'codex' }
        },
        {
          path: 'codex/auth',
          name: 'codex-auth',
          component: () => import('@/views/CodexAuthView.vue'),
          meta: { hideGlobalBackground: true, depth: 2, group: 'codex' }
        },
        {
          path: 'codex/settings',
          name: 'codex-settings',
          component: () => import('@/views/CodexSettingsView.vue'),
          meta: { hideGlobalBackground: true, depth: 2, group: 'codex' }
        },
        // Gemini CLI 子页面 (depth: 2, group: 'gemini')
        {
          path: 'gemini-cli/mcp',
          name: 'gemini-mcp',
          component: () => import('@/views/generic/PlatformMcpView.vue'),
          props: { platform: 'gemini' },
          meta: { hideGlobalBackground: true, depth: 2, group: 'gemini' }
        },
        {
          path: 'gemini-cli/agents',
          name: 'gemini-agents',
          component: () => import('@/views/generic/AgentsView.vue'),
          props: { module: 'gemini' },
          meta: { hideGlobalBackground: true, depth: 2, group: 'gemini' }
        },
        {
          path: 'gemini-cli/slash-commands',
          name: 'gemini-slash-commands',
          component: () => import('@/views/GeminiSlashCommandsView.vue'),
          meta: { hideGlobalBackground: true, depth: 2, group: 'gemini' }
        },
        {
          path: 'gemini-cli/plugins',
          name: 'gemini-plugins',
          component: () => import('@/views/generic/PlatformPluginsView.vue'),
          props: { platform: 'gemini' },
          meta: { hideGlobalBackground: true, depth: 2, group: 'gemini' }
        },
        // Qwen 子页面 (depth: 2, group: 'qwen')
        {
          path: 'qwen/mcp',
          name: 'qwen-mcp',
          component: () => import('@/views/generic/PlatformMcpView.vue'),
          props: { platform: 'qwen' },
          meta: { hideGlobalBackground: true, depth: 2, group: 'qwen' }
        },
        {
          path: 'qwen/agents',
          name: 'qwen-agents',
          component: () => import('@/views/generic/AgentsView.vue'),
          props: { module: 'qwen' },
          meta: { hideGlobalBackground: true, depth: 2, group: 'qwen' }
        },
        {
          path: 'qwen/slash-commands',
          name: 'qwen-slash-commands',
          component: () => import('@/views/QwenSlashCommandsView.vue'),
          meta: { hideGlobalBackground: true, depth: 2, group: 'qwen' }
        },
        {
          path: 'qwen/plugins',
          name: 'qwen-plugins',
          component: () => import('@/views/generic/PlatformPluginsView.vue'),
          props: { platform: 'qwen' },
          meta: { hideGlobalBackground: true, depth: 2, group: 'qwen' }
        },
        // iFlow 子页面 (depth: 2, group: 'iflow')
        {
          path: 'iflow/mcp',
          name: 'iflow-mcp',
          component: () => import('@/views/generic/PlatformMcpView.vue'),
          props: { platform: 'iflow' },
          meta: { hideGlobalBackground: true, depth: 2, group: 'iflow' }
        },
        {
          path: 'iflow/agents',
          name: 'iflow-agents',
          component: () => import('@/views/generic/AgentsView.vue'),
          props: { module: 'iflow' },
          meta: { hideGlobalBackground: true, depth: 2, group: 'iflow' }
        },
        {
          path: 'iflow/slash-commands',
          name: 'iflow-slash-commands',
          component: () => import('@/views/IflowSlashCommandsView.vue'),
          meta: { hideGlobalBackground: true, depth: 2, group: 'iflow' }
        },
        {
          path: 'iflow/plugins',
          name: 'iflow-plugins',
          component: () => import('@/views/generic/PlatformPluginsView.vue'),
          props: { platform: 'iflow' },
          meta: { hideGlobalBackground: true, depth: 2, group: 'iflow' }
        },
        // Droid 子页面 (depth: 2, group: 'droid')
        {
          path: 'droid/mcp',
          name: 'droid-mcp',
          component: () => import('@/views/generic/PlatformMcpView.vue'),
          props: { platform: 'droid' },
          meta: { hideGlobalBackground: true, depth: 2, group: 'droid' }
        },
        {
          path: 'droid/agents',
          name: 'droid-agents',
          component: () => import('@/views/generic/AgentsView.vue'),
          props: { module: 'droid' },
          meta: { hideGlobalBackground: true, depth: 2, group: 'droid' }
        },
        {
          path: 'droid/slash-commands',
          name: 'droid-slash-commands',
          component: () => import('@/views/DroidSlashCommandsView.vue'),
          meta: { hideGlobalBackground: true, depth: 2, group: 'droid' }
        },
        {
          path: 'droid/plugins',
          name: 'droid-plugins',
          component: () => import('@/views/DroidPluginsView.vue'),
          meta: { hideGlobalBackground: true, depth: 2, group: 'droid' }
        },
        {
          path: 'droid/models',
          name: 'droid-models',
          component: () => import('@/views/DroidModelsView.vue'),
          meta: { hideGlobalBackground: true, depth: 2, group: 'droid' }
        },
        {
          path: 'droid/profiles',
          name: 'droid-profiles',
          component: () => import('@/views/DroidProfilesView.vue'),
          meta: { hideGlobalBackground: true, depth: 2, group: 'droid' }
        },
        {
          path: 'droid/droids',
          name: 'droid-droids',
          component: () => import('@/views/DroidDroidsView.vue'),
          meta: { hideGlobalBackground: true, depth: 2, group: 'droid' }
        }
      ]
    }
  ],
  scrollBehavior() {
    // 始终滚动到顶部
    return { top: 0 }
  }
})

export default router
