import type { PlatformInfo } from "../models/types";

export type StatusBarMode = "pinned" | "current" | "hidden";

export interface StatusBarTargetInput {
  platforms: PlatformInfo[];
  currentPlatform?: string;
  mode?: string;
  pinnedPlatform?: string;
}

export interface StatusBarTargetResult {
  mode: StatusBarMode;
  visible: boolean;
  platform?: PlatformInfo;
  warning?: string;
}

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

export function resolveStatusBarTarget(input: StatusBarTargetInput): StatusBarTargetResult {
  const mode = normalizeStatusBarMode(input.mode);

  if (mode === "hidden") {
    return { mode, visible: false };
  }

  if (input.platforms.length === 0) {
    return { mode, visible: true };
  }

  const fallbackPlatform = input.platforms.find((platform) => platform.name === input.currentPlatform)
    ?? input.platforms[0];

  if (mode === "current") {
    return {
      mode,
      visible: true,
      platform: fallbackPlatform,
    };
  }

  const pinnedPlatformName = input.pinnedPlatform?.trim();
  if (!pinnedPlatformName) {
    return {
      mode,
      visible: true,
      platform: fallbackPlatform,
    };
  }

  const pinnedPlatform = input.platforms.find((platform) => platform.name === pinnedPlatformName);
  if (pinnedPlatform) {
    return {
      mode,
      visible: true,
      platform: pinnedPlatform,
    };
  }

  return {
    mode,
    visible: true,
    platform: fallbackPlatform,
    warning: `Configured status bar platform "${pinnedPlatformName}" was not found. Falling back to ${fallbackPlatform.displayName}.`,
  };
}
