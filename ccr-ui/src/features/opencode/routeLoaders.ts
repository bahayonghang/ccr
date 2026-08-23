const loadHome = () => import('./OpenCodeView').then((mod) => ({ Component: mod.OpenCodeView }))
const loadProviders = () =>
  import('./OpenCodeProvidersView').then((mod) => ({ Component: mod.OpenCodeProvidersView }))
const loadMcp = () => import('./OpenCodeMcpView').then((mod) => ({ Component: mod.OpenCodeMcpView }))
const loadAgents = () => import('./OpenCodeAgentsView').then((mod) => ({ Component: mod.OpenCodeAgentsView }))
const loadCommands = () =>
  import('./OpenCodeCommandsView').then((mod) => ({ Component: mod.OpenCodeCommandsView }))
const loadPlugins = () =>
  import('./OpenCodePluginsView').then((mod) => ({ Component: mod.OpenCodePluginsView }))
const loadSettings = () =>
  import('./OpenCodeSettingsView').then((mod) => ({ Component: mod.OpenCodeSettingsView }))
const loadSystemPrompts = () =>
  import('./OpenCodeSystemPromptsView').then((mod) => ({ Component: mod.OpenCodeSystemPromptsView }))

/** OpenCode 域懒加载表。key 对齐 routeCatalog id。 */
export const opencodeRouteLoaders = {
  opencode: loadHome,
  'opencode-providers': loadProviders,
  providers: loadProviders,
  'opencode-mcp': loadMcp,
  'opencode-agents': loadAgents,
  'opencode-commands': loadCommands,
  'opencode-plugins': loadPlugins,
  'opencode-settings': loadSettings,
  'opencode-system-prompts': loadSystemPrompts,
  'system-prompts': loadSystemPrompts,
} as const
