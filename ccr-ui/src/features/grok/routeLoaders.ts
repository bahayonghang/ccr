const loadHome = () => import('./GrokView').then((mod) => ({ Component: mod.GrokView }))
const loadAuth = () => import('./GrokAuthView').then((mod) => ({ Component: mod.GrokAuthView }))
const loadProfiles = () => import('./GrokProfilesView').then((mod) => ({ Component: mod.GrokProfilesView }))
const loadSettings = () => import('./GrokSettingsView').then((mod) => ({ Component: mod.GrokSettingsView }))

/** Grok 域懒加载表。key 对齐 routeCatalog id。 */
export const grokRouteLoaders = {
  grok: loadHome,
  'grok-auth': loadAuth,
  auth: loadAuth,
  'grok-profiles': loadProfiles,
  profiles: loadProfiles,
  'grok-settings': loadSettings,
  settings: loadSettings,
} as const
