import { createRouter, createWebHistory } from 'vue-router'

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
          meta: { cache: true }
        },
        // 主要模块
        {
          path: 'claude-code',
          name: 'claude-code',
          component: () => import('@/views/ClaudeCodeView.vue')
        },
        {
          path: 'codex',
          name: 'codex',
          component: () => import('@/views/CodexView.vue')
        },
        {
          path: 'gemini-cli',
          name: 'gemini-cli',
          component: () => import('@/views/GeminiCliView.vue')
        },
        {
          path: 'qwen',
          name: 'qwen',
          component: () => import('@/views/QwenView.vue')
        },
        {
          path: 'iflow',
          name: 'iflow',
          component: () => import('@/views/IflowView.vue')
        },
        {
          path: 'droid',
          name: 'droid',
          component: () => import('@/views/DroidView.vue')
        },
        // 工具中心
        {
          path: 'ccr-control',
          name: 'ccr-control',
          component: () => import('@/views/CcrControlView.vue'),
          meta: { cache: true }
        },
        {
          path: 'commands/:client?',
          name: 'commands',
          component: () => import('@/views/CommandsView.vue'),
          meta: { cache: true, stream: true }
        },
        {
          path: 'converter',
          name: 'converter',
          component: () => import('@/views/ConverterView.vue')
        },
        {
          path: 'sync',
          name: 'sync',
          component: () => import('@/views/SyncView.vue')
        },
        {
          path: 'configs',
          name: 'configs',
          component: () => import('@/views/ConfigsView.vue'),
          meta: { cache: true }
        },
        {
          path: 'stats',
          name: 'stats',
          component: () => import('@/views/StatsView.vue')
        },
        {
          path: 'budget',
          name: 'budget',
          component: () => import('@/views/BudgetView.vue')
        },
        {
          path: 'pricing',
          name: 'pricing',
          component: () => import('@/views/PricingView.vue')
        },
        {
          path: 'usage',
          name: 'usage',
          component: () => import('@/views/UsageView.vue'),
          meta: { cache: true }
        },
        {
          path: 'monitoring',
          name: 'monitoring',
          component: () => import('@/views/MonitoringView.vue')
        },
        {
          path: 'mcp',
          name: 'mcp',
          component: () => import('@/views/McpView.vue')
        },
        {
          path: 'slash-commands',
          name: 'slash-commands',
          component: () => import('@/views/SlashCommandsView.vue')
        },
        {
          path: 'agents',
          name: 'agents',
          component: () => import('@/views/generic/AgentsView.vue'),
          props: { module: 'agents' }
        },
        {
          path: 'agents/:name',
          name: 'agent-detail',
          component: () => import('@/views/generic/AgentDetailView.vue')
        },
        // Skills Hub (统一管理中心)
        {
          path: 'skills',
          name: 'skills',
          component: () => import('@/views/skills/UnifiedSkillsView.vue'),
          meta: { cache: true }
        },
        {
          path: 'skills/add',
          name: 'skills-add',
          component: () => import('@/views/skills/AddSkillView.vue'),
        },
        {
          path: 'skills/hub',
          redirect: '/skills'
        },
        {
          path: 'skills/:platform/:name',
          name: 'skill-detail',
          component: () => import('@/views/generic/SkillDetailView.vue'),
          props: true
        },
        {
          path: 'market',
          name: 'market',
          component: () => import('@/views/generic/MarketView.vue')
        },
        {
          path: 'plugins',
          name: 'plugins',
          component: () => import('@/views/PluginsView.vue')
        },
        {
          path: 'sessions',
          name: 'sessions',
          component: () => import('@/views/SessionsView.vue')
        },
        {
          path: 'hooks',
          name: 'hooks',
          component: () => import('@/views/HooksView.vue')
        },
        {
          path: 'output-styles',
          name: 'output-styles',
          component: () => import('@/views/OutputStylesView.vue')
        },
        {
          path: 'statusline',
          name: 'statusline',
          component: () => import('@/views/StatuslineView.vue')
        },
        {
          path: 'provider-health',
          name: 'provider-health',
          component: () => import('@/views/ProviderHealthView.vue')
        },
        {
          path: 'checkin/manage/:accountId',
          name: 'checkin-account-dashboard',
          component: () => import('@/views/checkin/CheckinAccountDashboardView.vue'),
          props: true
        },
        {
          path: 'checkin',
          name: 'checkin',
          component: () => import('@/views/CheckinView.vue')
        },
        // Codex 子页面
        {
          path: 'codex/mcp',
          name: 'codex-mcp',
          component: () => import('@/views/CodexMcpView.vue')
        },
        {
          path: 'codex/profiles',
          name: 'codex-profiles',
          component: () => import('@/views/CodexProfilesView.vue')
        },
        {
          path: 'codex/slash-commands',
          name: 'codex-slash-commands',
          component: () => import('@/views/CodexSlashCommandsView.vue')
        },
        {
          path: 'codex/auth',
          name: 'codex-auth',
          component: () => import('@/views/CodexAuthView.vue')
        },
        // Gemini CLI 子页面
        {
          path: 'gemini-cli/mcp',
          name: 'gemini-mcp',
          component: () => import('@/views/generic/PlatformMcpView.vue'),
          props: { platform: 'gemini' }
        },
        {
          path: 'gemini-cli/agents',
          name: 'gemini-agents',
          component: () => import('@/views/generic/AgentsView.vue'),
          props: { module: 'gemini' }
        },
        {
          path: 'gemini-cli/slash-commands',
          name: 'gemini-slash-commands',
          component: () => import('@/views/GeminiSlashCommandsView.vue')
        },
        {
          path: 'gemini-cli/plugins',
          name: 'gemini-plugins',
          component: () => import('@/views/generic/PlatformPluginsView.vue'),
          props: { platform: 'gemini' }
        },
        // Qwen 子页面
        {
          path: 'qwen/mcp',
          name: 'qwen-mcp',
          component: () => import('@/views/generic/PlatformMcpView.vue'),
          props: { platform: 'qwen' }
        },
        {
          path: 'qwen/agents',
          name: 'qwen-agents',
          component: () => import('@/views/generic/AgentsView.vue'),
          props: { module: 'qwen' }
        },
        {
          path: 'qwen/slash-commands',
          name: 'qwen-slash-commands',
          component: () => import('@/views/QwenSlashCommandsView.vue')
        },
        {
          path: 'qwen/plugins',
          name: 'qwen-plugins',
          component: () => import('@/views/generic/PlatformPluginsView.vue'),
          props: { platform: 'qwen' }
        },
        // iFlow 子页面
        {
          path: 'iflow/mcp',
          name: 'iflow-mcp',
          component: () => import('@/views/generic/PlatformMcpView.vue'),
          props: { platform: 'iflow' }
        },
        {
          path: 'iflow/agents',
          name: 'iflow-agents',
          component: () => import('@/views/generic/AgentsView.vue'),
          props: { module: 'iflow' }
        },
        {
          path: 'iflow/slash-commands',
          name: 'iflow-slash-commands',
          component: () => import('@/views/IflowSlashCommandsView.vue')
        },
        {
          path: 'iflow/plugins',
          name: 'iflow-plugins',
          component: () => import('@/views/generic/PlatformPluginsView.vue'),
          props: { platform: 'iflow' }
        },
        // Droid 子页面
        {
          path: 'droid/mcp',
          name: 'droid-mcp',
          component: () => import('@/views/generic/PlatformMcpView.vue'),
          props: { platform: 'droid' }
        },
        {
          path: 'droid/agents',
          name: 'droid-agents',
          component: () => import('@/views/generic/AgentsView.vue'),
          props: { module: 'droid' }
        },
        {
          path: 'droid/slash-commands',
          name: 'droid-slash-commands',
          component: () => import('@/views/DroidSlashCommandsView.vue')
        },
        {
          path: 'droid/plugins',
          name: 'droid-plugins',
          component: () => import('@/views/DroidPluginsView.vue')
        },
        {
          path: 'droid/models',
          name: 'droid-models',
          component: () => import('@/views/DroidModelsView.vue')
        },
        {
          path: 'droid/profiles',
          name: 'droid-profiles',
          component: () => import('@/views/DroidProfilesView.vue')
        },
        {
          path: 'droid/droids',
          name: 'droid-droids',
          component: () => import('@/views/DroidDroidsView.vue')
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
