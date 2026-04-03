/**
 * TOML reader for CCR configuration files
 *
 * Reads:
 * - config.toml (UnifiedConfig registry)
 * - platforms/{name}/profiles.toml (CcsConfig profiles)
 */

import * as fs from "fs";
import * as TOML from "smol-toml";
import { getRegistryPath, getProfilesPath, getPlatformDisplayName, getPlatformIcon, SUPPORTED_PLATFORMS } from "./ccrPaths";
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

function isNotFoundError(error: unknown): boolean {
  if (!error || typeof error !== "object") return false;
  if (!("code" in error)) return false;
  return (error as { code?: string }).code === "ENOENT";
}

/** Read and parse the unified registry */
export async function readRegistry(): Promise<{ platforms: PlatformInfo[]; currentPlatform: string } | null> {
  const registryPath = getRegistryPath();

  try {
    const content = await fs.promises.readFile(registryPath, "utf-8");
    const raw = TOML.parse(content) as unknown as UnifiedConfig;

    const currentPlatform = raw.current_platform ?? "claude";
    const platforms: PlatformInfo[] = [];

    for (const [key, value] of Object.entries(raw)) {
      if (REGISTRY_TOP_KEYS.has(key) || !SUPPORTED_PLATFORMS.includes(key as typeof SUPPORTED_PLATFORMS[number])) {
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
    if (isNotFoundError(err)) {
      return null;
    }
    console.error("Failed to read CCR registry:", err);
    return null;
  }
}

// ── Profiles (profiles.toml) ──

/** Read and parse profiles for a platform */
export async function readProfiles(platformName: string): Promise<ProfileInfo[]> {
  const profilesPath = getProfilesPath(platformName);

  try {
    const content = await fs.promises.readFile(profilesPath, "utf-8");
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
    if (isNotFoundError(err)) {
      return [];
    }
    console.error(`Failed to read profiles for ${platformName}:`, err);
    return [];
  }
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
