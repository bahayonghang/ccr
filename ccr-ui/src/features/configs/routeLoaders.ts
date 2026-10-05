/** configs 域懒加载表。key 对齐 routeCatalog id。 */
export const configsRouteLoaders = {
  configs: () => import('./ConfigsView').then((mod) => ({ Component: mod.ConfigsView })),
  settings: () => import('./AppSettingsView').then((mod) => ({ Component: mod.AppSettingsView })),
  converter: () => import('./ConverterView').then((mod) => ({ Component: mod.ConverterView })),
} as const

export const configsRouteIds = Object.keys(configsRouteLoaders)
