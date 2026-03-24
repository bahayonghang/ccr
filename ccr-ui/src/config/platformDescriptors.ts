export interface GenericPlatformFeatureRoute {
  path: string
  name: string
}

export interface GenericPlatformDescriptor {
  id: 'gemini' | 'qwen' | 'qoder' | 'droid'
  rootPath: 'gemini-cli' | 'qwen' | 'qoder' | 'droid'
  color: string
  mcp: GenericPlatformFeatureRoute & {
    i18nPrefix: string
  }
  agents: GenericPlatformFeatureRoute & {
    module: 'gemini' | 'qwen' | 'qoder' | 'droid'
  }
  plugins?: GenericPlatformFeatureRoute & {
    i18nPrefix: string
    sidebarModule: 'gemini-cli' | 'qwen' | 'qoder'
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
  qwen: {
    id: 'qwen',
    rootPath: 'qwen',
    color: '#14b8a6',
    mcp: {
      path: 'mcp',
      name: 'qwen-mcp',
      i18nPrefix: 'qwen.mcp',
    },
    agents: {
      path: 'agents',
      name: 'qwen-agents',
      module: 'qwen',
    },
    plugins: {
      path: 'plugins',
      name: 'qwen-plugins',
      i18nPrefix: 'qwen.plugins',
      sidebarModule: 'qwen',
    },
  },
  qoder: {
    id: 'qoder',
    rootPath: 'qoder',
    color: '#f97316',
    mcp: {
      path: 'mcp',
      name: 'qoder-mcp',
      i18nPrefix: 'qoder.mcp',
    },
    agents: {
      path: 'subagents',
      name: 'qoder-subagents',
      module: 'qoder',
    },
    plugins: {
      path: 'plugins',
      name: 'qoder-plugins',
      i18nPrefix: 'qoder.plugins',
      sidebarModule: 'qoder',
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
