import type { CodexRuntimeSnapshot, PlatformInfo, ProfileInfo } from "../models/types";

const COMPACT_PLATFORM_LABELS: Record<string, string> = {
  claude: "CC",
  codex: "CDX",
};

function compactAuthIdentity(snapshot?: CodexRuntimeSnapshot | null): string {
  if (!snapshot) {
    return "auth?";
  }

  return snapshot.runtimeSummary.currentAuthName
    || snapshot.runtimeSummary.authLabel
    || "auth?";
}

export function getCompactPlatformLabel(platformName: string): string {
  return COMPACT_PLATFORM_LABELS[platformName] || platformName.toUpperCase();
}

export function buildStatusBarText(
  platform: PlatformInfo,
  profileName: string,
  runtime?: CodexRuntimeSnapshot | null,
): string {
  const label = getCompactPlatformLabel(platform.name);

  if (platform.name !== "codex") {
    return `${label}: ${profileName}`;
  }

  const runtimeProfile = runtime?.runtimeSummary.currentProfileName || profileName;
  return `${label}: ${runtimeProfile}`;
}

export function buildStatusBarTooltipLines(
  mode: "pinned" | "current" | "hidden",
  platform: PlatformInfo,
  profileName: string,
  currentProfile: ProfileInfo | undefined,
  runtime?: CodexRuntimeSnapshot | null,
  warning?: string,
): string[] {
  const lines = [
    `**CCR Status · ${platform.displayName}**`,
    `Platform: ${platform.displayName}`,
  ];

  if (platform.name === "codex" && runtime) {
    lines.push(`Profile: ${runtime.runtimeSummary.profileLabel}`);
    lines.push(`Auth: ${runtime.runtimeSummary.authLabel}`);
    lines.push(`Control: ${runtime.runtimeSummary.mode}`);
    if (runtime.authSidecarLabel) {
      lines.push(`Sidecar: ${runtime.authSidecarLabel}`);
    }
    lines.push(`Source: ${runtime.dataSource}`);
    if (runtime.binaryPath) {
      lines.push(`Binary: ${runtime.binaryPath}`);
    }
    if (runtime.capabilityWarnings.length > 0) {
      lines.push(...runtime.capabilityWarnings.map((warningText) => `$(warning) ${warningText}`));
    }
  } else {
    lines.push(`Profile: ${profileName}`);
  }

  if (currentProfile?.model) {
    lines.push(`Model: ${currentProfile.model}`);
  }
  if (currentProfile?.provider) {
    lines.push(`Provider: ${currentProfile.provider}`);
  }
  if (platform.name === "codex" && runtime && !runtime.currentAuthInfo) {
    lines.push(`Identity: ${compactAuthIdentity(runtime)}`);
  }
  if (warning) {
    lines.push(`$(warning) ${warning}`);
  }

  lines.push("");
  lines.push(
    mode === "hidden"
      ? `_Status bar hidden_`
      : `_Click to switch ${platform.displayName} profile_`,
  );

  return lines;
}
