import type { ProfileCreateRequest, ProfileCreationPlatform } from "../models/types";

export interface PlatformProfileMutationData {
  ok: boolean;
  platform: string;
  name: string;
  message: string;
  enabled?: boolean;
  current_profile?: string;
}

export interface CodexAuthUpdateData {
  ok: boolean;
  name: string;
  description?: string;
  message: string;
}

export interface PlatformAuthOffData {
  ok: boolean;
  changed: boolean;
  path: "file" | "native_logout";
  profile_pointer?: string;
  warnings?: string[];
}

export type ProfileFieldValue = string | number | boolean | string[] | undefined;

type ProfileScopedAction = "create" | "set-field" | "enable" | "disable" | "delete" | "switch" | "off";

function buildPlatformScopedProfileArgs(platformName: string, action: ProfileScopedAction, rest: string[] = []): string[] {
  return [platformName, "profile", action, ...rest];
}

export function buildPlatformProfileCreateArgs(
  platformName: ProfileCreationPlatform,
  profileName: string,
  config: ProfileCreateRequest,
): string[] {
  const args = buildPlatformScopedProfileArgs(platformName, "create", [profileName, "--json"]);

  if (config.description) args.push("--description", config.description);
  if (config.base_url) args.push("--base-url", config.base_url);
  if (config.auth_token) args.push("--auth-token", config.auth_token);
  if (config.model) args.push("--model", config.model);
  if (config.small_fast_model) args.push("--small-fast-model", config.small_fast_model);
  if (config.provider) args.push("--provider", config.provider);
  if (config.provider_type) args.push("--provider-type", config.provider_type);
  if (config.account) args.push("--account", config.account);
  for (const tag of config.tags ?? []) {
    const trimmed = tag.trim();
    if (trimmed) {
      args.push("--tag", trimmed);
    }
  }
  if (config.enabled === false) {
    args.push("--disabled");
  }

  return args;
}

export function buildPlatformProfileSetFieldArgs(
  platformName: string,
  profileName: string,
  field: string,
  value: ProfileFieldValue,
): string[] {
  const args = buildPlatformScopedProfileArgs(platformName, "set-field", [profileName, field, "--json"]);

  if (value === undefined || value === "") {
    args.push("--clear");
  } else if (Array.isArray(value)) {
    args.push("--value-json", JSON.stringify(value));
  } else {
    args.push("--value", String(value));
  }

  return args;
}

export function buildPlatformProfileEnableArgs(platformName: string, profileName: string): string[] {
  return buildPlatformScopedProfileArgs(platformName, "enable", [profileName, "--json"]);
}

export function buildPlatformProfileDisableArgs(
  platformName: string,
  profileName: string,
  force = false,
): string[] {
  const args = buildPlatformScopedProfileArgs(platformName, "disable", [profileName, "--json"]);
  if (force) {
    args.push("--force");
  }
  return args;
}

export function buildPlatformProfileDeleteArgs(
  platformName: string,
  profileName: string,
  force = false,
): string[] {
  const args = buildPlatformScopedProfileArgs(platformName, "delete", [profileName, "--json"]);
  if (force) {
    args.push("--force");
  }
  return args;
}

export function buildPlatformProfileSwitchArgs(platformName: string, profileName: string): string[] {
  return buildPlatformScopedProfileArgs(platformName, "switch", [profileName]);
}

export function buildPlatformProfileOffArgs(platformName: string): string[] {
  return buildPlatformScopedProfileArgs(platformName, "off", ["--json"]);
}

export function buildPlatformAuthOffArgs(platformName: string): string[] {
  return [platformName, "auth", "off", "--json"];
}

export function buildClaudeProfileSwitchArgs(profileName: string): string[] {
  return buildPlatformProfileSwitchArgs("claude", profileName);
}

export function buildCodexProfileSwitchArgs(profileName: string): string[] {
  return buildPlatformProfileSwitchArgs("codex", profileName);
}

export function buildClaudeProfileOffArgs(): string[] {
  return buildPlatformProfileOffArgs("claude");
}

export function buildCodexProfileOffArgs(): string[] {
  return buildPlatformProfileOffArgs("codex");
}

export function buildCodexAuthUpdateArgs(name: string, description: string | undefined): string[] {
  const args = ["codex", "auth", "update", name, "--json"];
  if (description === undefined) {
    args.push("--clear-description");
  } else {
    args.push("--description", description);
  }
  return args;
}
