const loadCodex = () => import('./CodexView').then((mod) => ({ Component: mod.CodexView }))
const loadMcp = () => import('./CodexMcpView').then((mod) => ({ Component: mod.CodexMcpView }))
const loadProfiles = () => import('./CodexProfilesView').then((mod) => ({ Component: mod.CodexProfilesView }))
const loadAgents = () => import('./CodexAgentsView').then((mod) => ({ Component: mod.CodexAgentsView }))
const loadSessions = () => import('./CodexSessionsView').then((mod) => ({ Component: mod.CodexSessionsView }))
const loadSlashCommands = () => import('./CodexSlashCommandsView').then((mod) => ({ Component: mod.CodexSlashCommandsView }))
const loadAuth = () => import('./CodexAuthView').then((mod) => ({ Component: mod.CodexAuthView }))
const loadSettings = () => import('./CodexSettingsView').then((mod) => ({ Component: mod.CodexSettingsView }))
const loadSystemPrompts = () => import('./CodexSystemPromptsView').then((mod) => ({ Component: mod.CodexSystemPromptsView }))

/** Codex 域懒加载表。key 对齐 routeCatalog id 与任务简写 id。tray 不在本任务。 */
export const codexRouteLoaders = {
  codex: loadCodex,
  'codex-mcp': loadMcp,
  profiles: loadProfiles,
  'codex-profiles': loadProfiles,
  agents: loadAgents,
  'codex-agents': loadAgents,
  sessions: loadSessions,
  'codex-sessions': loadSessions,
  'slash-commands': loadSlashCommands,
  'codex-slash-commands': loadSlashCommands,
  auth: loadAuth,
  'codex-auth': loadAuth,
  settings: loadSettings,
  'codex-settings': loadSettings,
  'system-prompts': loadSystemPrompts,
  'codex-system-prompts': loadSystemPrompts,
} as const
