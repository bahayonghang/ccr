export type PlatformCapabilityId = "claude" | "codex" | "gemini" | "qwen" | "droid";

export interface PlatformCapability {
  id: PlatformCapabilityId;
  displayName: string;
  implemented: boolean;
  supportsProfiles: boolean;
  supportsMcp: boolean;
  supportsAgents: boolean;
  supportsStatusBar: boolean;
}

export const PLATFORM_CAPABILITIES: readonly PlatformCapability[] = [
  {
    id: "claude",
    displayName: "Claude Code",
    implemented: true,
    supportsProfiles: true,
    supportsMcp: true,
    supportsAgents: true,
    supportsStatusBar: true,
  },
  {
    id: "codex",
    displayName: "Codex",
    implemented: true,
    supportsProfiles: true,
    supportsMcp: true,
    supportsAgents: true,
    supportsStatusBar: true,
  },
  {
    id: "gemini",
    displayName: "Antigravity CLI",
    implemented: true,
    supportsProfiles: true,
    supportsMcp: true,
    supportsAgents: true,
    supportsStatusBar: false,
  },
  {
    id: "qwen",
    displayName: "Qwen CLI",
    implemented: false,
    supportsProfiles: false,
    supportsMcp: false,
    supportsAgents: false,
    supportsStatusBar: false,
  },
  {
    id: "droid",
    displayName: "Factory Droid",
    implemented: true,
    supportsProfiles: true,
    supportsMcp: false,
    supportsAgents: false,
    supportsStatusBar: false,
  },
] as const;

export const STATUS_BAR_PLATFORM_NAMES = PLATFORM_CAPABILITIES
  .filter((capability) => capability.supportsStatusBar)
  .map((capability) => capability.id);
