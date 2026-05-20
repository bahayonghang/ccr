export type PlatformCapabilityId = 'claude' | 'codex' | 'gemini' | 'qwen' | 'droid'

export interface PlatformCapability {
  id: PlatformCapabilityId
  displayName: string
  implemented: boolean
  supportsProfiles: boolean
  supportsMcp: boolean
  supportsAgents: boolean
  supportsStatusBar: boolean
}

export const platformCapabilities = {
  claude: {
    id: 'claude',
    displayName: 'Claude Code',
    implemented: true,
    supportsProfiles: true,
    supportsMcp: true,
    supportsAgents: true,
    supportsStatusBar: true,
  },
  codex: {
    id: 'codex',
    displayName: 'Codex',
    implemented: true,
    supportsProfiles: true,
    supportsMcp: true,
    supportsAgents: true,
    supportsStatusBar: true,
  },
  gemini: {
    id: 'gemini',
    displayName: 'Antigravity CLI',
    implemented: true,
    supportsProfiles: true,
    supportsMcp: true,
    supportsAgents: true,
    supportsStatusBar: false,
  },
  qwen: {
    id: 'qwen',
    displayName: 'Qwen CLI',
    implemented: false,
    supportsProfiles: false,
    supportsMcp: false,
    supportsAgents: false,
    supportsStatusBar: false,
  },
  droid: {
    id: 'droid',
    displayName: 'Factory Droid',
    implemented: true,
    supportsProfiles: true,
    supportsMcp: false,
    supportsAgents: false,
    supportsStatusBar: false,
  },
} as const satisfies Record<PlatformCapabilityId, PlatformCapability>

export const platformCapabilityList: PlatformCapability[] = Object.values(platformCapabilities)

export const statusBarPlatformIds = platformCapabilityList
  .filter(capability => capability.supportsStatusBar)
  .map(capability => capability.id)
