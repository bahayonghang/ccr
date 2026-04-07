import type { CodexRuntimeSnapshot, PlatformInfo, ProfileInfo } from "../models/types";

function compactAuthIdentity(snapshot?: CodexRuntimeSnapshot | null): string {
  if (!snapshot) {
    return "auth?";
  }

  return snapshot.runtimeSummary.currentAuthName
    || snapshot.runtimeSummary.authLabel
    || "auth?";
}

export function buildStatusBarText(
  platform: PlatformInfo,
  profileName: string,
  runtime?: CodexRuntimeSnapshot | null,
): string {
  if (platform.name !== "codex") {
    return `${platform.icon} ${platform.displayName}: ${profileName}`;
  }

  const runtimeProfile = runtime?.runtimeSummary.currentProfileName || profileName;
  const auth = compactAuthIdentity(runtime);
  return `${platform.icon} ${platform.displayName}: ${runtimeProfile} · ${auth}`;
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
    `**CCR Profile Status**`,
    `Mode: ${mode === "pinned" ? "Pinned platform" : "Current platform"}`,
    `Platform: ${platform.displayName}`,
  ];

  if (platform.name === "codex" && runtime) {
    lines.push(`Runtime: ${runtime.runtimeSummary.profileLabel}`);
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
  if (warning) {
    lines.push(`$(warning) ${warning}`);
  }

  lines.push("");
  lines.push(
    mode === "pinned"
      ? `_Click to switch profiles for ${platform.displayName}_`
      : `_Click to switch profile_`,
  );

  return lines;
}
