/**
 * CCR path resolution — mirrors Rust PlatformPaths::get_ccr_root() logic
 */

import * as os from "os";
import * as path from "path";
import * as fs from "fs";

/** Resolve CCR root directory: $CCR_ROOT || ~/.ccr/ */
export function getCcrRoot(): string {
  const envRoot = process.env["CCR_ROOT"];
  if (envRoot) {
    return envRoot;
  }
  return path.join(os.homedir(), ".ccr");
}

/** Check if CCR root directory exists */
export function ccrRootExists(): boolean {
  return fs.existsSync(getCcrRoot());
}

/** Path to the unified registry: ~/.ccr/config.toml */
export function getRegistryPath(): string {
  return path.join(getCcrRoot(), "config.toml");
}

/** Path to a platform's profiles: ~/.ccr/platforms/{name}/profiles.toml */
export function getProfilesPath(platformName: string): string {
  return path.join(getCcrRoot(), "platforms", platformName, "profiles.toml");
}

/** Path to Codex auth registry: ~/.ccr/platforms/codex/auth_registry.toml */
export function getCodexAuthRegistryPath(): string {
  return path.join(getCcrRoot(), "platforms", "codex", "auth_registry.toml");
}

/** Path to Codex auth directory: ~/.ccr/platforms/codex/auth/ */
export function getCodexAuthDir(): string {
  return path.join(getCcrRoot(), "platforms", "codex", "auth");
}

/** Path to the platforms directory: ~/.ccr/platforms/ */
export function getPlatformsDir(): string {
  return path.join(getCcrRoot(), "platforms");
}

/** Glob pattern for watching all profiles.toml files */
export function getProfilesGlob(): string {
  return path.join(getCcrRoot(), "platforms", "*", "profiles.toml");
}

/** Supported platforms exposed by the VSCode extension */
export const SUPPORTED_PLATFORMS = ["claude", "codex"] as const;

/** Known platform metadata — aligned with Rust Platform enum */
export const PLATFORM_META: Record<
  string,
  { displayName: string; icon: string; shortName: string; codiconId: string }
> = {
  claude: {
    displayName: "Claude Code",
    icon: "\u{1F916}",
    shortName: "claude",
    codiconId: "hubot",
  },
  codex: {
    displayName: "Codex",
    icon: "\u{1F4BB}",
    shortName: "codex",
    codiconId: "terminal",
  },
};

/** Get display name for a platform, fallback to capitalized name */
export function getPlatformDisplayName(name: string): string {
  return PLATFORM_META[name]?.displayName ?? name.charAt(0).toUpperCase() + name.slice(1);
}

/** Get icon for a platform */
export function getPlatformIcon(name: string): string {
  return PLATFORM_META[name]?.icon ?? "\u{1F4E6}";
}

/** Get codicon ID for a platform (for VSCode ThemeIcon) */
export function getPlatformCodiconId(name: string): string {
  return PLATFORM_META[name]?.codiconId ?? "package";
}
