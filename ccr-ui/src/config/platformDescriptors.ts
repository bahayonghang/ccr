/** 七个跨平台功能面。slash-commands 已统一，不在此列。 */
export const PLATFORM_SURFACES = [
  'settings',
  'profiles',
  'auth',
  'mcp',
  'agents',
  'plugins',
  'commands',
] as const

export type PlatformSurface = (typeof PLATFORM_SURFACES)[number]

/**
 * descriptor 层：声明该平台有哪些面，驱动导航与（未来）路由生成。
 * 平台路径与 `routeCatalog` 现网 76 条记录对齐；独立工具页不属于平台 surface。
 */
export interface PlatformSurfaceDescriptor {
  id: string
  rootPath: string
  surfaces: readonly PlatformSurface[]
}

export const platformSurfaceDescriptors = {
  claude: {
    id: 'claude',
    rootPath: '/claude-code',
    surfaces: ['settings', 'profiles', 'auth', 'mcp', 'agents', 'plugins', 'commands'],
  },
  codex: {
    id: 'codex',
    rootPath: '/codex',
    surfaces: ['settings', 'profiles', 'auth', 'mcp', 'agents'],
  },
  grok: {
    id: 'grok',
    rootPath: '/grok',
    surfaces: ['settings', 'profiles', 'auth'],
  },
  opencode: {
    id: 'opencode',
    rootPath: '/opencode',
    surfaces: ['settings', 'mcp', 'agents', 'plugins', 'commands'],
  },
  gemini: {
    id: 'gemini',
    rootPath: '/antigravity',
    surfaces: ['mcp', 'agents', 'plugins'],
  },
} as const satisfies Record<string, PlatformSurfaceDescriptor>

export type PlatformSurfaceId = keyof typeof platformSurfaceDescriptors

export const platformSurfaceDescriptorList: PlatformSurfaceDescriptor[] = Object.values(
  platformSurfaceDescriptors,
)

export function platformHasSurface(
  id: PlatformSurfaceId,
  surface: PlatformSurface,
): boolean {
  const descriptor: PlatformSurfaceDescriptor = platformSurfaceDescriptors[id]
  return descriptor.surfaces.includes(surface)
}

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
