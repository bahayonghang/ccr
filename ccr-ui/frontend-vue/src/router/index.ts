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
  ]
})

export default router