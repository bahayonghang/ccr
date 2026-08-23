import {
  addGeminiPlugin,
  addOpenCodePlugin,
  addPlugin,
  deleteGeminiPlugin,
  deleteOpenCodePlugin,
  deletePlugin,
  listGeminiPlugins,
  listOpenCodeLocalPlugins,
  listOpenCodePlugins,
  listPlugins,
  toggleGeminiPlugin,
  togglePlugin,
  updateGeminiPlugin,
  updatePlugin,
} from '@/api'
import { surfaceNotify, type SurfaceNotify } from '@/configs/surfaceNotify'

export interface PluginRecord {
  id: string
  name: string
  version?: string
  enabled?: boolean
  localPath?: string
}

export interface PluginDraft {
  id: string
  name: string
  version?: string
  enabled?: boolean
  configJson?: string
}

export interface PluginsFeatures {
  toggle?: boolean
  configJson?: boolean
  localFiles?: boolean
  version?: boolean
}

export interface PluginsConfig {
  cacheKey: string
  homePath: string
  module: string
  i18nPrefix: string
  titleKey: string
  subtitleKey: string
  parentPath: string
  features: PluginsFeatures
  notify: SurfaceNotify
  list: () => Promise<PluginRecord[]>
  create: (draft: PluginDraft) => Promise<void>
  update?: (id: string, draft: PluginDraft) => Promise<void>
  remove: (id: string) => Promise<void>
  toggle?: (id: string) => Promise<void>
}

const asRecord = (value: unknown): Record<string, unknown> =>
  value && typeof value === 'object' ? (value as Record<string, unknown>) : {}

const toPlugin = (value: unknown): PluginRecord | null => {
  const source = asRecord(value)
  const name = typeof source.name === 'string' ? source.name : typeof source.id === 'string' ? source.id : ''
  if (!name) return null
  return {
    id: typeof source.id === 'string' ? source.id : name,
    name,
    version: typeof source.version === 'string' ? source.version : undefined,
    enabled: typeof source.enabled === 'boolean' ? source.enabled : undefined,
    localPath: typeof source.path === 'string' ? source.path : undefined,
  }
}

const readPlugins = (payload: unknown): unknown[] => {
  if (Array.isArray(payload)) return payload
  const source = asRecord(payload)
  if (Array.isArray(source.plugins)) return source.plugins
  return []
}

export const claudePluginsConfig: PluginsConfig = {
  cacheKey: 'plugins-claude',
  homePath: '/plugins',
  module: 'config',
  i18nPrefix: 'plugins',
  titleKey: 'plugins.title',
  subtitleKey: 'plugins.subtitle',
  parentPath: '/',
  features: { toggle: true, configJson: true, version: true },
  notify: surfaceNotify,
  list: async () => (await listPlugins()).map((row) => ({
    id: row.id,
    name: row.name,
    version: row.version,
    enabled: row.enabled,
  })),
  create: async (draft) => {
    await addPlugin(draft.id || draft.name, {
      name: draft.name,
      version: draft.version,
      enabled: draft.enabled,
    })
  },
  update: async (id, draft) => {
    await updatePlugin(id, { name: draft.name, version: draft.version, enabled: draft.enabled })
  },
  remove: async (id) => {
    await deletePlugin(id)
  },
  toggle: async (id) => {
    await togglePlugin(id)
  },
}

export const geminiPluginsConfig: PluginsConfig = {
  cacheKey: 'plugins-gemini',
  homePath: '/antigravity/plugins',
  module: 'antigravity',
  i18nPrefix: 'gemini.plugins',
  titleKey: 'gemini.plugins.title',
  subtitleKey: 'gemini.plugins.subtitle',
  parentPath: '/antigravity',
  features: { toggle: true, configJson: true, version: true },
  notify: surfaceNotify,
  list: async () =>
    readPlugins(await listGeminiPlugins()).map(toPlugin).filter((item): item is PluginRecord => item !== null),
  create: async (draft) => {
    await addGeminiPlugin({ id: draft.id, name: draft.name, version: draft.version, enabled: draft.enabled })
  },
  update: async (id, draft) => {
    await updateGeminiPlugin(id, { id: draft.id, name: draft.name, version: draft.version, enabled: draft.enabled })
  },
  remove: async (id) => {
    await deleteGeminiPlugin(id)
  },
  toggle: async (id) => {
    await toggleGeminiPlugin(id)
  },
}

export const opencodePluginsConfig: PluginsConfig = {
  cacheKey: 'plugins-opencode',
  homePath: '/opencode/plugins',
  module: 'opencode',
  i18nPrefix: 'opencode.plugins',
  titleKey: 'opencode.plugins.title',
  subtitleKey: 'opencode.plugins.subtitle',
  parentPath: '/opencode',
  features: { localFiles: true },
  notify: surfaceNotify,
  list: async () => {
    const [names, localFiles] = await Promise.all([listOpenCodePlugins(), listOpenCodeLocalPlugins()])
    const fromNames = names.map((name) => ({ id: name, name }))
    const fromFiles = localFiles.map((file) => ({
      id: file.path,
      name: file.name,
      localPath: file.path,
    }))
    return [...fromNames, ...fromFiles]
  },
  create: async (draft) => {
    await addOpenCodePlugin(draft.name)
  },
  remove: async (id) => {
    await deleteOpenCodePlugin(id)
  },
}

export const pluginsConfigs = {
  claude: claudePluginsConfig,
  gemini: geminiPluginsConfig,
  opencode: opencodePluginsConfig,
} as const
