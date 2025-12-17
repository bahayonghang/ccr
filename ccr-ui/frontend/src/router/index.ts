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
        // 工具中心
        {
          path: 'ccr-control',
          name: 'ccr-control',
          component: () => import('@/views/CcrControlView.vue'),
          meta: { cache: true }
        },
        {
          path: 'commands',
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
          path: 'skills',
          name: 'skills',
          component: () => import('@/views/generic/SkillsView.vue')
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
        // Gemini CLI 子页面
        {
          path: 'gemini-cli/mcp',
          name: 'gemini-mcp',
          component: () => import('@/views/GeminiMcpView.vue')
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
          component: () => import('@/views/GeminiPluginsView.vue')
        },
        // Qwen 子页面
        {
          path: 'qwen/mcp',
          name: 'qwen-mcp',
          component: () => import('@/views/QwenMcpView.vue')
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
          component: () => import('@/views/QwenPluginsView.vue')
        },
        // iFlow 子页面
        {
          path: 'iflow/mcp',
          name: 'iflow-mcp',
          component: () => import('@/views/IflowMcpView.vue')
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
          component: () => import('@/views/IflowPluginsView.vue')
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
