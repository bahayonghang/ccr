/**
 * TypeScript interfaces aligned with CCR Rust structs
 *
 * Mapping:
 * - UnifiedConfig      ← crates/ccr/src/managers/platform_config.rs:63
 * - PlatformConfigEntry ← crates/ccr/src/managers/platform_config.rs:21
 * - CcsConfig          ← crates/ccr/src/managers/config/ccs_config.rs:13
 * - ProfileConfig      ← crates/ccr/src/models/platform.rs:143
 */

// ── Registry: ~/.ccr/config.toml ──

/** Per-platform entry in the unified registry */
export interface PlatformConfigEntry {
  enabled: boolean;
  current_profile?: string;
  description?: string;
  last_used?: string;
}

/**
 * Unified config registry (~/.ccr/config.toml)
 *
 * TOML uses `#[serde(flatten)]` so platform entries are top-level keys
 * alongside `default_platform` and `current_platform`.
 */
export interface UnifiedConfig {
  default_platform: string;
  current_platform: string;
  /** Flattened platform entries — keys are platform short names */
  [platform: string]: string | PlatformConfigEntry;
}

// ── Profiles: ~/.ccr/platforms/{name}/profiles.toml ──

/** A single profile configuration */
export interface ProfileConfig {
  description?: string;
  base_url?: string;
  auth_token?: string;
  model?: string;
  small_fast_model?: string;
  provider?: string;
  provider_type?: string;
  account?: string;
  tags?: string[];
  usage_count?: number;
  enabled?: boolean;
  /** Platform-specific data (flattened in TOML) */
  [key: string]: unknown;
}

export interface ProfileCreateRequest {
  description?: string;
  base_url?: string;
  auth_token?: string;
  model?: string;
  small_fast_model?: string;
  provider?: string;
  provider_type?: string;
  account?: string;
  tags?: string[];
  enabled?: boolean;
}

export const PROFILE_CREATION_PLATFORMS = ["claude", "codex"] as const;
export type ProfileCreationPlatform = typeof PROFILE_CREATION_PLATFORMS[number];

/** Global settings in profiles.toml (optional) */
export interface GlobalSettings {
  [key: string]: unknown;
}

/**
 * Per-platform profiles file (~/.ccr/platforms/{name}/profiles.toml)
 *
 * CcsConfig format with top-level scalars + flattened profile sections.
 */
export interface CcsConfig {
  default_config: string;
  current_config: string;
  settings?: GlobalSettings;
  /** Flattened profile sections — keys are profile names */
  [profile: string]: string | GlobalSettings | ProfileConfig | undefined;
}

// ── Parsed data (post-processing) ──

/** Parsed platform info for TreeView consumption */
export interface PlatformInfo {
  name: string;
  displayName: string;
  icon: string;
  enabled: boolean;
  currentProfile?: string;
}

/** Parsed profile info for TreeView consumption */
export interface ProfileInfo {
  name: string;
  platformName: string;
  description?: string;
  baseUrl?: string;
  authToken?: string;
  model?: string;
  smallFastModel?: string;
  provider?: string;
  providerType?: string;
  account?: string;
  tags?: string[];
  usageCount: number;
  enabled: boolean;
  isCurrent: boolean;
}

export type TreeSectionKind = "profiles" | "auth";

export interface TreeSectionInfo {
  kind: TreeSectionKind;
  platformName: string;
  label: string;
  description: string;
}

export interface CodexAuthInfo {
  name: string;
  description?: string;
  email?: string;
  savedAt?: string;
  lastUsed?: string;
  lastRefresh?: string;
  expiresAt?: string;
  isCurrent: boolean;
  isVirtual: boolean;
}

export type ProfileEditorMode = "edit" | "create";

export interface ProfileEditorDraft {
  name: string;
  platformName: ProfileCreationPlatform;
  description?: string;
  baseUrl?: string;
  authToken?: string;
  model?: string;
  smallFastModel?: string;
  provider?: string;
  providerType?: string;
  account?: string;
  tags?: string[];
  enabled: boolean;
}

export type EditableFieldDefinition = {
  key: string;
  tomlKey: string;
  label: string;
};

/** Editable fields of a profile (key = camelCase matching ProfileInfo, tomlKey = TOML snake_case) */
export const EDITABLE_FIELDS: EditableFieldDefinition[] = [
  { key: "description",    tomlKey: "description",      label: "description" },
  { key: "baseUrl",        tomlKey: "base_url",         label: "base_url" },
  { key: "authToken",      tomlKey: "auth_token",       label: "auth_token" },
  { key: "model",          tomlKey: "model",            label: "model" },
  { key: "smallFastModel", tomlKey: "small_fast_model", label: "small_fast_model" },
  { key: "provider",       tomlKey: "provider",         label: "provider" },
  { key: "providerType",   tomlKey: "provider_type",    label: "provider_type" },
  { key: "account",        tomlKey: "account",          label: "account" },
  { key: "tags",           tomlKey: "tags",             label: "tags" },
];

export const DEFAULT_PROFILE_EDITABLE_FIELDS = EDITABLE_FIELDS.map((field) => field.key);

export const PROFILE_EDITABLE_FIELDS_BY_PLATFORM: Record<string, string[]> = {
  claude: DEFAULT_PROFILE_EDITABLE_FIELDS,
  codex: ["description", "model", "smallFastModel", "provider", "providerType", "account", "tags"],
};

export function getEditableProfileFields(platformName: string): string[] {
  return PROFILE_EDITABLE_FIELDS_BY_PLATFORM[platformName] ?? DEFAULT_PROFILE_EDITABLE_FIELDS;
}

export function isProfileCreationPlatform(platformName: string): platformName is ProfileCreationPlatform {
  return PROFILE_CREATION_PLATFORMS.includes(platformName as ProfileCreationPlatform);
}
