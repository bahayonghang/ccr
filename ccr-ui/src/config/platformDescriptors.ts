export interface GenericPlatformFeatureRoute {
  path: string
  name: string
}

export interface GenericPlatformDescriptor {
  id: 'gemini'
  rootPath: 'gemini-cli'
  color: string
  mcp: GenericPlatformFeatureRoute & {
    i18nPrefix: string
  }
  agents: GenericPlatformFeatureRoute & {
    module: 'gemini'
  }
  plugins?: GenericPlatformFeatureRoute & {
    i18nPrefix: string
    sidebarModule: 'gemini-cli'
  }
}

export const genericPlatformDescriptors = {
  gemini: {
    id: 'gemini',
    rootPath: 'gemini-cli',
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
      sidebarModule: 'gemini-cli',
    },
  },
} as const satisfies Record<string, GenericPlatformDescriptor>

export type GenericPlatformId = keyof typeof genericPlatformDescriptors

export const genericPlatformDescriptorList: GenericPlatformDescriptor[] = Object.values(
  genericPlatformDescriptors,
)
