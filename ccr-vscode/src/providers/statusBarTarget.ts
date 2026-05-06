import type { PlatformInfo } from "../models/types";
import {
  STATUS_BAR_PLATFORM_NAMES,
  type PlatformCapabilityId,
} from "../models/platformCapabilities";

export type StatusBarMode = "pinned" | "current" | "hidden";
export type StatusBarPlatformName = Extract<PlatformCapabilityId, "claude" | "codex">;

export interface StatusBarTargetInput {
  platforms: PlatformInfo[];
  currentPlatform?: string;
  mode?: string;
  pinnedPlatform?: string;
  showClaude?: boolean;
  showCodex?: boolean;
}

export interface StatusBarTargetResult {
  mode: StatusBarMode;
  visible: boolean;
  platform?: PlatformInfo;
  warning?: string;
}

const SUPPORTED_STATUS_BAR_PLATFORMS = STATUS_BAR_PLATFORM_NAMES as readonly StatusBarPlatformName[];

export function normalizeStatusBarMode(mode: string | undefined): StatusBarMode {
  switch (mode) {
    case "current":
    case "hidden":
    case "pinned":
      return mode;
    default:
      return "pinned";
  }
}

export function getSupportedStatusBarPlatforms(platforms: PlatformInfo[]): PlatformInfo[] {
  return SUPPORTED_STATUS_BAR_PLATFORMS
    .map((name) => platforms.find((platform) => platform.name === name))
    .filter((platform): platform is PlatformInfo => Boolean(platform));
}

function getEnabledPlatforms(input: StatusBarTargetInput): PlatformInfo[] {
  const supportedPlatforms = getSupportedStatusBarPlatforms(input.platforms);
  const enabledMap: Record<StatusBarPlatformName, boolean> = {
    claude: input.showClaude ?? true,
    codex: input.showCodex ?? true,
  };

  return supportedPlatforms.filter((platform) => enabledMap[platform.name as StatusBarPlatformName] ?? false);
}

export function resolveStatusBarItems(input: StatusBarTargetInput): StatusBarTargetResult[] {
  const mode = normalizeStatusBarMode(input.mode);
  if (mode === "hidden") {
    return [];
  }

  const enabledPlatforms = getEnabledPlatforms(input);
  if (enabledPlatforms.length === 0) {
    return [];
  }

  if (mode === "current") {
    const currentPlatform = enabledPlatforms.find((platform) => platform.name === input.currentPlatform)
      ?? enabledPlatforms[0];

    return [{
      mode,
      visible: true,
      platform: currentPlatform,
    }];
  }

  return enabledPlatforms.map((platform) => ({
    mode,
    visible: true,
    platform,
  }));
}

export function resolveStatusBarTarget(input: StatusBarTargetInput): StatusBarTargetResult {
  const mode = normalizeStatusBarMode(input.mode);
  const [firstItem] = resolveStatusBarItems(input);

  if (!firstItem) {
    return {
      mode,
      visible: false,
    };
  }

  return firstItem;
}
