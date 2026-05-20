export interface GenericPlatformFeatureRoute {
  path: string
  name: string
}

export interface GenericPlatformDescriptor {
  id: 'gemini'
  rootPath: 'antigravity'
  color: string
  mcp: GenericPlatformFeatureRoute & {
    i18nPrefix: string
  }
  agents: GenericPlatformFeatureRoute & {
    module: 'gemini'
  }
  plugins?: GenericPlatformFeatureRoute & {
    i18nPrefix: string
    sidebarModule: 'antigravity'
  }
}

export const genericPlatformDescriptors = {
  gemini: {
    id: 'gemini',
    rootPath: 'antigravity',
    color: '#8b5cf6',
    mcp: {
      path: 'mcp',
      name: 'gemini-mcp',
      i18nPrefix: 'gemini.mcp',
    },
    agents: {
      path: 'agents',
      name: 'gemini-agents',
      module: 'gemini',
    },
    plugins: {
      path: 'plugins',
      name: 'gemini-plugins',
      i18nPrefix: 'gemini.plugins',
      sidebarModule: 'antigravity',
    },
  },
} as const satisfies Record<string, GenericPlatformDescriptor>

export type GenericPlatformId = keyof typeof genericPlatformDescriptors

export const genericPlatformDescriptorList: GenericPlatformDescriptor[] = Object.values(
  genericPlatformDescriptors
)
