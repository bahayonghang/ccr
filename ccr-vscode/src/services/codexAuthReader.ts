import * as fs from "fs";
import * as TOML from "smol-toml";
import { getCodexAuthRegistryPath } from "./ccrPaths";
import type { CodexAuthInfo } from "../models/types";

interface CodexAuthRegistryEntry {
  description?: string;
  email?: string;
  saved_at?: string;
  last_used?: string;
  last_refresh?: string;
  expires_at?: string;
}

interface CodexAuthRegistryShape {
  current_auth?: string;
  accounts?: Record<string, CodexAuthRegistryEntry>;
}

function isNotFoundError(error: unknown): boolean {
  if (!error || typeof error !== "object") return false;
  if (!("code" in error)) return false;
  return (error as { code?: string }).code === "ENOENT";
}

export async function readCodexAuthAccounts(): Promise<CodexAuthInfo[]> {
  const registryPath = getCodexAuthRegistryPath();

  try {
    const content = await fs.promises.readFile(registryPath, "utf-8");
    const raw = TOML.parse(content) as unknown as CodexAuthRegistryShape;
    const currentAuth = raw.current_auth;
    const accounts = Object.entries(raw.accounts ?? {}).map(([name, value]) => ({
      name,
      description: value.description,
      email: value.email,
      savedAt: value.saved_at,
      lastUsed: value.last_used,
      lastRefresh: value.last_refresh,
      expiresAt: value.expires_at,
      isCurrent: name === currentAuth,
      isVirtual: false,
    } satisfies CodexAuthInfo));

    accounts.sort((a, b) => {
      if (a.isCurrent !== b.isCurrent) return a.isCurrent ? -1 : 1;
      return a.name.localeCompare(b.name);
    });

    return accounts;
  } catch (error) {
    if (isNotFoundError(error)) {
      return [];
    }
    console.error("Failed to read Codex auth registry:", error);
    return [];
  }
}
