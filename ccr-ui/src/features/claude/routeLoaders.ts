import type { LazyRouteFunction, RouteObject } from 'react-router'

type ClaudeLazyLoader = LazyRouteFunction<RouteObject>

/** Claude 域懒加载器。shell router 按 route id 消费，本文件不改 router。 */
export const claudeRouteLoaders: Record<string, ClaudeLazyLoader> = {
  'claude-code': () =>
    import('./ClaudeCodeView').then((mod) => ({ Component: mod.ClaudeCodeView })),
  'claude-code-settings': () =>
    import('./ClaudeSettingsView').then((mod) => ({ Component: mod.ClaudeSettingsView })),
  'claude-system-prompts': () =>
    import('./ClaudeSystemPromptsView').then((mod) => ({ Component: mod.ClaudeSystemPromptsView })),
  'claude-code-profiles': () =>
    import('./ClaudeProfilesView').then((mod) => ({ Component: mod.ClaudeProfilesView })),
  'claude-code-auth': () =>
    import('./ClaudeAuthView').then((mod) => ({ Component: mod.ClaudeAuthView })),
  hooks: () => import('./HooksView').then((mod) => ({ Component: mod.HooksView })),
  'output-styles': () =>
    import('./OutputStylesView').then((mod) => ({ Component: mod.OutputStylesView })),
  statusline: () => import('./StatuslineView').then((mod) => ({ Component: mod.StatuslineView })),
  'slash-commands': () =>
    import('./ClaudeSlashCommandsView').then((mod) => ({ Component: mod.ClaudeSlashCommandsView })),
  plugins: () => import('./ClaudePluginsView').then((mod) => ({ Component: mod.ClaudePluginsView })),
  skills: () =>
    import('./SkillsMigrationView').then((mod) => ({ Component: mod.SkillsMigrationView })),
}

export const claudeRouteIds = Object.keys(claudeRouteLoaders)
