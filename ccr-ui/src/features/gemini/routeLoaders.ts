const loadHome = () => import('./GeminiCliView').then((mod) => ({ Component: mod.GeminiCliView }))
const loadSlash = () =>
  import('./GeminiSlashCommandsView').then((mod) => ({ Component: mod.GeminiSlashCommandsView }))
const loadMcp = () => import('./GeminiMcpView').then((mod) => ({ Component: mod.GeminiMcpView }))
const loadAgents = () => import('./GeminiAgentsView').then((mod) => ({ Component: mod.GeminiAgentsView }))
const loadPlugins = () => import('./GeminiPluginsView').then((mod) => ({ Component: mod.GeminiPluginsView }))
const loadSystemPrompts = () =>
  import('./GeminiSystemPromptsView').then((mod) => ({ Component: mod.GeminiSystemPromptsView }))
const loadAgentDetail = () =>
  import('@/features/platform/agents/AgentDetailView').then((mod) => ({ Component: mod.AgentDetailView }))
const loadAgentsHome = () =>
  import('@/features/platform/agents/AgentsHomeView').then((mod) => ({ Component: mod.AgentsHomeView }))

/** Gemini / Antigravity 域懒加载表。含 generic agents 留守路由。 */
export const geminiRouteLoaders = {
  antigravity: loadHome,
  'gemini-cli': loadHome,
  'gemini-slash-commands': loadSlash,
  'slash-commands': loadSlash,
  'gemini-mcp': loadMcp,
  'gemini-agents': loadAgents,
  'gemini-plugins': loadPlugins,
  'gemini-system-prompts': loadSystemPrompts,
  'system-prompts': loadSystemPrompts,
  'agent-detail': loadAgentDetail,
  agents: loadAgentsHome,
} as const

export const antigravityRouteLoaders = geminiRouteLoaders
