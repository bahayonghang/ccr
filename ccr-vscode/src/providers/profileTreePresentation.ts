import type {
  CodexAuthQuotaInfo,
  CodexRuntimeSnapshot,
  PlatformInfo,
  TreeSectionInfo,
  TreeSectionKind,
} from "../models/types";

export interface RuntimeDetailDescriptor {
  label: string;
  description: string;
  icon: string;
  tooltip: string;
}

export interface CodexAuthDetailDescriptor {
  key: string;
  label: string;
  description?: string;
  icon: string;
  tone: "success" | "warning" | "danger" | "neutral";
  tooltip: string;
}

function compactPathTail(filePath?: string): string {
  if (!filePath) {
    return "not found";
  }

  const normalized = filePath.replace(/\\/g, "/");
  const parts = normalized.split("/");
  return parts.length <= 3
    ? normalized
    : `.../${parts.slice(-3).join("/")}`;
}

export function getSectionInfo(platformName: string, kind: TreeSectionKind): TreeSectionInfo {
  if (platformName === "claude") {
    return {
      kind,
      platformName,
      label: "Claude Profiles",
      description: "Switch and manage Claude profiles",
    };
  }

  if (kind === "runtime") {
    return {
      kind,
      platformName,
      label: "Codex Runtime",
      description: "Current control mode, profile route, and auth identity",
    };
  }

  if (kind === "auth") {
    return {
      kind,
      platformName,
      label: "Codex Auth",
      description: "Switch and inspect saved Codex auth accounts",
    };
  }

  return {
    kind,
    platformName,
    label: "Codex Profiles",
    description: "Switch and manage Codex profiles",
  };
}

export function getRuntimeModeCompactLabel(mode?: string): string {
  switch (mode) {
    case "profile_only":
      return "ProfileOnly";
    case "profile_with_auth":
      return "Profile+Auth";
    case "profile_pending_auth":
      return "PendingAuth";
    case "runtime_only":
      return "RuntimeOnly";
    case "unresolved":
      return "Unresolved";
    default:
      return "Runtime";
  }
}

export function getRuntimeModeDisplayLabel(mode?: string): string {
  switch (mode) {
    case "profile_only":
      return "Profile 驱动";
    case "profile_with_auth":
      return "Profile 路由 + Auth 身份";
    case "profile_pending_auth":
      return "Profile 路由，等待 Auth";
    case "runtime_only":
      return "仅 Runtime/Auth 生效";
    case "unresolved":
      return "未解析";
    default:
      return "运行态加载中";
  }
}

function compactProfileIdentity(platform: PlatformInfo, snapshot?: CodexRuntimeSnapshot): string {
  return snapshot?.runtimeSummary.currentProfileName?.trim()
    || platform.currentProfile?.trim()
    || "none";
}

export function getCodexPlatformDescription(
  platform: PlatformInfo,
  snapshot?: CodexRuntimeSnapshot,
): string | undefined {
  if (!platform.enabled) {
    return "(disabled)";
  }

  const mode = getRuntimeModeCompactLabel(snapshot?.runtimeSummary.mode);
  const profile = compactProfileIdentity(platform, snapshot);
  return `${mode} · ${profile}`;
}

function getRemainingTone(remaining?: number): "success" | "warning" | "danger" | "neutral" {
  if (remaining === undefined) {
    return "neutral";
  }

  if (remaining < 30) {
    return "danger";
  }
  if (remaining < 60) {
    return "warning";
  }
  return "success";
}

function formatQuotaWindowDescription(remaining: number, resetTime?: number): string {
  const resetLabel = formatQuotaReset(resetTime);
  return resetLabel
    ? `${remaining}% · reset ${resetLabel}`
    : `${remaining}% · reset unavailable`;
}

function formatQuotaWindowTooltip(label: string, remaining: number, resetTime?: number): string {
  const resetLabel = formatQuotaReset(resetTime) ?? "Unavailable";
  return `${label} quota\n\nRemaining: ${remaining}%\nReset: ${resetLabel}`;
}

export function formatCodexAuthDescription(
  quota?: CodexAuthQuotaInfo,
  quotaFetchError?: string | null,
): string | undefined {
  if (quota?.quota?.planType) {
    return quota.quota.planType;
  }

  if (quota?.error) {
    return "quota unavailable";
  }

  if (quotaFetchError) {
    return "quota unavailable";
  }

  return quota ? undefined : "loading quota…";
}

export function getQuotaTone(quota?: CodexAuthQuotaInfo): "success" | "warning" | "danger" | "neutral" {
  if (!quota) {
    return "neutral";
  }
  if (quota.error) {
    return "warning";
  }
  if (!quota.quota) {
    return "neutral";
  }

  return getRemainingTone(Math.min(quota.quota.hourlyPercentage, quota.quota.weeklyPercentage));
}

export function formatQuotaReset(timestamp?: number): string | undefined {
  if (!timestamp) {
    return undefined;
  }

  const date = new Date(timestamp * 1000);
  if (Number.isNaN(date.getTime())) {
    return undefined;
  }

  return new Intl.DateTimeFormat("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  }).format(date);
}

export function buildCodexAuthDetailDescriptors(
  quota?: CodexAuthQuotaInfo,
  quotaFetchError?: string | null,
): CodexAuthDetailDescriptor[] {
  if (quota?.quota) {
    return [
      {
        key: "5h",
        label: "5h",
        description: formatQuotaWindowDescription(
          quota.quota.hourlyPercentage,
          quota.quota.hourlyResetTime,
        ),
        icon: "history",
        tone: getRemainingTone(quota.quota.hourlyPercentage),
        tooltip: formatQuotaWindowTooltip(
          "5h",
          quota.quota.hourlyPercentage,
          quota.quota.hourlyResetTime,
        ),
      },
      {
        key: "7d",
        label: "7d",
        description: formatQuotaWindowDescription(
          quota.quota.weeklyPercentage,
          quota.quota.weeklyResetTime,
        ),
        icon: "calendar",
        tone: getRemainingTone(quota.quota.weeklyPercentage),
        tooltip: formatQuotaWindowTooltip(
          "7d",
          quota.quota.weeklyPercentage,
          quota.quota.weeklyResetTime,
        ),
      },
    ];
  }

  if (quota?.error) {
    return [{
      key: "status",
      label: "Quota unavailable",
      description: quota.error,
      icon: "warning",
      tone: "warning",
      tooltip: `Quota unavailable\n\n${quota.error}`,
    }];
  }

  if (quotaFetchError) {
    return [{
      key: "status",
      label: "Quota unavailable",
      description: quotaFetchError,
      icon: "warning",
      tone: "warning",
      tooltip: `Quota unavailable\n\n${quotaFetchError}`,
    }];
  }

  return [{
    key: "status",
    label: "Quota loading…",
    description: "Waiting for CCR CLI snapshot",
    icon: "clock",
    tone: "neutral",
    tooltip: "Quota loading…\n\nWaiting for CCR CLI snapshot.",
  }];
}

export function buildCodexRuntimeDetails(
  platform: PlatformInfo,
  snapshot?: CodexRuntimeSnapshot | null,
  runtimeError?: string | null,
): RuntimeDetailDescriptor[] {
  const modeDescription = snapshot
    ? getRuntimeModeDisplayLabel(snapshot.runtimeSummary.mode)
    : runtimeError
      ? "Runtime summary unavailable"
      : "Loading Codex runtime summary...";

  const profileDescription = snapshot?.runtimeSummary.profileLabel
    || platform.currentProfile
    || "未绑定";
  const authDescription = snapshot?.runtimeSummary.authLabel
    || (runtimeError ? "Unavailable" : "Loading...");

  const authSource = snapshot?.runtimeSummary.currentProfileAuthSource
    || (snapshot ? "runtime auth.json" : "—");

  return [
    {
      label: "Control Mode",
      description: snapshot
        ? getRuntimeModeCompactLabel(snapshot.runtimeSummary.mode)
        : runtimeError
          ? "Unavailable"
          : "Loading...",
      icon: "pulse",
      tooltip: `${modeDescription}${runtimeError ? `\n\n${runtimeError}` : ""}`,
    },
    {
      label: "Profile Route",
      description: profileDescription,
      icon: "git-branch",
      tooltip: snapshot?.runtimeSummary.currentProfileProvider
        ? `${profileDescription}\n\nProvider: ${snapshot.runtimeSummary.currentProfileProvider}`
        : profileDescription,
    },
    {
      label: "Auth Identity",
      description: authDescription,
      icon: "key",
      tooltip: `Identity: ${authDescription}\n\nSource: ${authSource}${
        snapshot?.currentAuthInfo
          ? `\n\nAccount: ${snapshot.currentAuthInfo.accountId}`
          : ""
      }${
        snapshot?.currentAuthInfo?.email
          ? `\nEmail: ${snapshot.currentAuthInfo.email}`
          : ""
      }${
        snapshot?.authState.reason
          ? `\nReason: ${snapshot.authState.reason}`
          : ""
      }${
        runtimeError ? `\n\n${runtimeError}` : ""
      }`,
    },
    {
      label: "Auth Sidecar",
      description: snapshot?.authSidecarLabel ?? "None",
      icon: "plug",
      tooltip: snapshot?.authSidecarLabel
        ? `Profile is primary. ${snapshot.authSidecarLabel}.`
        : "No sidecar auth detected outside the current profile contract.",
    },
    {
      label: "Data Source",
      description: snapshot
        ? snapshot.dataSource === "cli_json"
          ? "CCR CLI JSON"
          : snapshot.dataSource === "local_fallback"
            ? "Local fallback"
            : "Unavailable"
        : runtimeError
          ? "Unavailable"
          : "Loading...",
      icon: "database",
      tooltip: snapshot?.capabilityWarnings.length
        ? snapshot.capabilityWarnings.join("\n")
        : "Using the preferred runtime data path.",
    },
    {
      label: "CCR Binary",
      description: compactPathTail(snapshot?.binaryPath),
      icon: "terminal",
      tooltip: snapshot?.binaryPath
        ? snapshot.binaryPath
        : "No CCR CLI found in PATH. Runtime view is using local fallback only.",
    },
  ];
}
