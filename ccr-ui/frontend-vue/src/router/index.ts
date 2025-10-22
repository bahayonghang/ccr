import { createRouter, createWebHistory } from 'vue-router'
import HomeView from '@/views/HomeView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: HomeView
    },
    {
      path: '/claude-code',
      name: 'claude-code',
      component: () => import('@/views/ClaudeCodeView.vue')
    },
    {
      path: '/codex',
      name: 'codex',
      component: () => import('@/views/CodexView.vue')
    },
    {
      path: '/gemini-cli',
      name: 'gemini-cli',
      component: () => import('@/views/GeminiCliView.vue')
    },
    {
      path: '/qwen',
      name: 'qwen',
      component: () => import('@/views/QwenView.vue')
    },
    {
      path: '/iflow',
      name: 'iflow',
      component: () => import('@/views/IflowView.vue')
    },
    {
      path: '/commands',
      name: 'commands',
      component: () => import('@/views/CommandsView.vue')
    },
    {
      path: '/converter',
      name: 'converter',
      component: () => import('@/views/ConverterView.vue')
    },
    {
      path: '/sync',
      name: 'sync',
      component: () => import('@/views/SyncView.vue')
    },
    {
      path: '/configs',
      name: 'configs',
      component: () => import('@/views/ConfigsView.vue')
    },
    {
      path: '/mcp',
      name: 'mcp',
      component: () => import('@/views/McpView.vue')
    },
    {
      path: '/slash-commands',
      name: 'slash-commands',
      component: () => import('@/views/SlashCommandsView.vue')
    },
    {
      path: '/agents',
      name: 'agents',
      component: () => import('@/views/AgentsView.vue')
    },
    {
      path: '/plugins',
      name: 'plugins',
      component: () => import('@/views/PluginsView.vue')
    },
    {
      path: '/codex/mcp',
      name: 'codex-mcp',
      component: () => import('@/views/CodexMcpView.vue')
    },
    {
      path: '/codex/profiles',
      name: 'codex-profiles',
      component: () => import('@/views/CodexProfilesView.vue')
    },
    {
      path: '/gemini-cli/mcp',
      name: 'gemini-mcp',
      component: () => import('@/views/GeminiMcpView.vue')
    },
    {
      path: '/qwen/mcp',
      name: 'qwen-mcp',
      component: () => import('@/views/QwenMcpView.vue')
    },
    // Codex 子页面
    {
      path: '/codex/agents',
      name: 'codex-agents',
      component: () => import('@/views/CodexAgentsView.vue')
    },
    {
      path: '/codex/slash-commands',
      name: 'codex-slash-commands',
      component: () => import('@/views/CodexSlashCommandsView.vue')
    },
    {
      path: '/codex/plugins',
      name: 'codex-plugins',
      component: () => import('@/views/CodexPluginsView.vue')
    },
    // Gemini CLI 子页面
    {
      path: '/gemini-cli/agents',
      name: 'gemini-agents',
      component: () => import('@/views/GeminiAgentsView.vue')
    },
    {
      path: '/gemini-cli/slash-commands',
      name: 'gemini-slash-commands',
      component: () => import('@/views/GeminiSlashCommandsView.vue')
    },
    {
      path: '/gemini-cli/plugins',
      name: 'gemini-plugins',
      component: () => import('@/views/GeminiPluginsView.vue')
    },
    // Qwen 子页面
    {
      path: '/qwen/agents',
      name: 'qwen-agents',
      component: () => import('@/views/QwenAgentsView.vue')
    },
    {
      path: '/qwen/slash-commands',
      name: 'qwen-slash-commands',
      component: () => import('@/views/QwenSlashCommandsView.vue')
    },
    {
      path: '/qwen/plugins',
      name: 'qwen-plugins',
      component: () => import('@/views/QwenPluginsView.vue')
    },
    // iFlow 子页面
    {
      path: '/iflow/mcp',
      name: 'iflow-mcp',
      component: () => import('@/views/IflowMcpView.vue')
    },
    {
      path: '/iflow/agents',
      name: 'iflow-agents',
      component: () => import('@/views/IflowAgentsView.vue')
    },
    {
      path: '/iflow/slash-commands',
      name: 'iflow-slash-commands',
      component: () => import('@/views/IflowSlashCommandsView.vue')
    },
    {
      path: '/iflow/plugins',
      name: 'iflow-plugins',
      component: () => import('@/views/IflowPluginsView.vue')
    },
  ]
})

export default router