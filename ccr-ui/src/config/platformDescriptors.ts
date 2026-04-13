export interface GenericPlatformFeatureRoute {
  path: string
  name: string
}

export interface GenericPlatformDescriptor {
  id: 'gemini' | 'droid'
  rootPath: 'gemini-cli' | 'droid'
  color: string
  mcp: GenericPlatformFeatureRoute & {
    i18nPrefix: string
  }
  agents: GenericPlatformFeatureRoute & {
    module: 'gemini' | 'droid'
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
  droid: {
    id: 'droid',
    rootPath: 'droid',
    color: '#ec4899',
    mcp: {
      path: 'mcp',
      name: 'droid-mcp',
      i18nPrefix: 'droid.mcp',
    },
    agents: {
      path: 'agents',
      name: 'droid-agents',
      module: 'droid',
    },
  },
} as const satisfies Record<string, GenericPlatformDescriptor>

export type GenericPlatformId = keyof typeof genericPlatformDescriptors

export const genericPlatformDescriptorList: GenericPlatformDescriptor[] = Object.values(
  genericPlatformDescriptors,
)
