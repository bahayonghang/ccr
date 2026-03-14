/**
 * TOML reader/writer for CCR configuration files
 *
 * Reads:
 * - config.toml (UnifiedConfig registry)
 * - platforms/{name}/profiles.toml (CcsConfig profiles)
 *
 * Writes:
 * - profiles.toml (preserving top-level fields)
 */

import * as fs from "fs";
import * as TOML from "smol-toml";
import { getRegistryPath, getProfilesPath, getPlatformDisplayName, getPlatformIcon } from "./ccrPaths";
import type {
  UnifiedConfig,
  PlatformConfigEntry,
  CcsConfig,
  ProfileConfig,
  PlatformInfo,
  ProfileInfo,
} from "../models/types";

// ── Top-level keys that are NOT profile sections ──
const CCS_TOP_KEYS = new Set(["default_config", "current_config", "settings"]);
const REGISTRY_TOP_KEYS = new Set(["default_platform", "current_platform"]);

// ── Registry (config.toml) ──

/** Read and parse the unified registry */
export function readRegistry(): { platforms: PlatformInfo[]; currentPlatform: string } | null {
  const registryPath = getRegistryPath();
  if (!fs.existsSync(registryPath)) {
    return null;
  }

  try {
    const content = fs.readFileSync(registryPath, "utf-8");
    const raw = TOML.parse(content) as unknown as UnifiedConfig;

    const currentPlatform = raw.current_platform ?? "claude";
    const platforms: PlatformInfo[] = [];

    for (const [key, value] of Object.entries(raw)) {
      if (REGISTRY_TOP_KEYS.has(key)) {
        continue;
      }
      if (typeof value === "object" && value !== null) {
        const entry = value as PlatformConfigEntry;
        platforms.push({
          name: key,
          displayName: getPlatformDisplayName(key),
          icon: getPlatformIcon(key),
          enabled: entry.enabled ?? true,
          currentProfile: entry.current_profile,
        });
      }
    }

    // Sort: current platform first, then alphabetical
    platforms.sort((a, b) => {
      if (a.name === currentPlatform) return -1;
      if (b.name === currentPlatform) return 1;
      return a.name.localeCompare(b.name);
    });

    return { platforms, currentPlatform };
  } catch (err) {
    console.error("Failed to read CCR registry:", err);
    return null;
  }
}

// ── Profiles (profiles.toml) ──

/** Read and parse profiles for a platform */
export function readProfiles(platformName: string): ProfileInfo[] {
  const profilesPath = getProfilesPath(platformName);
  if (!fs.existsSync(profilesPath)) {
    return [];
  }

  try {
    const content = fs.readFileSync(profilesPath, "utf-8");
    const raw = TOML.parse(content) as unknown as CcsConfig;
    const currentConfig = raw.current_config ?? "";

    const profiles: ProfileInfo[] = [];

    for (const [key, value] of Object.entries(raw)) {
      if (CCS_TOP_KEYS.has(key)) {
        continue;
      }
      if (typeof value === "object" && value !== null && !Array.isArray(value)) {
        const p = value as ProfileConfig;
        profiles.push({
          name: key,
          platformName,
          description: asString(p.description),
          baseUrl: asString(p.base_url),
          authToken: asString(p.auth_token),
          model: asString(p.model),
          smallFastModel: asString(p.small_fast_model),
          provider: asString(p.provider),
          providerType: asString(p.provider_type),
          account: asString(p.account),
          tags: p.tags as string[] | undefined,
          usageCount: (p.usage_count as number) ?? 0,
          enabled: (p.enabled as boolean) ?? true,
          isCurrent: key === currentConfig,
        });
      }
    }

    return profiles;
  } catch (err) {
    console.error(`Failed to read profiles for ${platformName}:`, err);
    return [];
  }
}

/** Read raw CcsConfig for a platform (used by writeProfiles) */
function readRawProfiles(platformName: string): Record<string, unknown> | null {
  const profilesPath = getProfilesPath(platformName);
  if (!fs.existsSync(profilesPath)) {
    return null;
  }

  try {
    const content = fs.readFileSync(profilesPath, "utf-8");
    return TOML.parse(content) as Record<string, unknown>;
  } catch {
    return null;
  }
}

/**
 * Write an updated profile field back to profiles.toml
 *
 * Preserves all top-level fields (default_config, current_config, settings)
 * and all other profile sections.
 */
export async function writeProfileField(
  platformName: string,
  profileName: string,
  field: string,
  value: string | boolean | number | string[] | undefined
): Promise<void> {
  const profilesPath = getProfilesPath(platformName);
  const raw = readRawProfiles(platformName);
  if (!raw) {
    throw new Error(`Cannot read profiles.toml for platform: ${platformName}`);
  }

  // Get or create the profile section
  const section = raw[profileName];
  if (!section || typeof section !== "object" || Array.isArray(section)) {
    throw new Error(`Profile '${profileName}' not found in ${platformName}/profiles.toml`);
  }

  // Update the field
  const profileObj = section as Record<string, unknown>;
  if (value === undefined || value === "") {
    delete profileObj[field];
  } else {
    profileObj[field] = value;
  }

  // Serialize and write back
  const tomlStr = TOML.stringify(raw as TOML.TomlPrimitive);
  await fs.promises.writeFile(profilesPath, tomlStr, "utf-8");
}

/**
 * Toggle the `enabled` field of a profile
 */
export async function toggleProfileEnabled(platformName: string, profileName: string): Promise<boolean> {
  const profilesPath = getProfilesPath(platformName);
  const raw = readRawProfiles(platformName);
  if (!raw) {
    throw new Error(`Cannot read profiles.toml for platform: ${platformName}`);
  }

  const section = raw[profileName];
  if (!section || typeof section !== "object" || Array.isArray(section)) {
    throw new Error(`Profile '${profileName}' not found in ${platformName}/profiles.toml`);
  }

  const profileObj = section as Record<string, unknown>;
  const currentEnabled = (profileObj["enabled"] as boolean) ?? true;
  profileObj["enabled"] = !currentEnabled;

  const tomlStr = TOML.stringify(raw as TOML.TomlPrimitive);
  await fs.promises.writeFile(profilesPath, tomlStr, "utf-8");

  return !currentEnabled;
}

// ── Helpers ──

function asString(val: unknown): string | undefined {
  if (typeof val === "string" && val.length > 0) {
    return val;
  }
  return undefined;
}

/** Mask an auth token for display: show only last 4 chars */
export function maskToken(token: string | undefined): string {
  if (!token || token.length <= 4) {
    return token ? "****" : "";
  }
  return "****" + token.slice(-4);
}
