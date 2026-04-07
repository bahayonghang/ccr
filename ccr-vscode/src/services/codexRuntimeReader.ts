import * as fs from "fs";
import * as TOML from "smol-toml";
import type {
  CodexAuthStateInfo,
  CodexCurrentAuthInfo,
  CodexLoginStateInfo,
  CodexRuntimeSnapshot,
  CodexRuntimeSummaryInfo,
} from "../models/types";
import { detectCcrCapabilities, execCodexAuthCurrentJson } from "./ccrCli";
import {
  getCodexAuthRegistryPath,
  getCodexRuntimeAuthPath,
  getProfilesPath,
} from "./ccrPaths";

const RUNTIME_CACHE_TTL_MS = 5_000;

interface RawCodexAuthState {
  intent: {
    kind: string;
    method?: string;
    env_key?: string;
    [key: string]: unknown;
  };
  store: string;
  status: string;
  reason: string;
}

interface RawCodexLoginState {
  type: string;
  account_name?: string;
  env_key?: string;
  [key: string]: unknown;
}

interface RawCodexCurrentAuthInfo {
  account_id: string;
  auth_method?: string;
  email?: string;
  last_refresh?: string;
  freshness: string;
}

interface RawCodexRuntimeSummary {
  mode: string;
  current_profile_name?: string;
  current_profile_provider?: string;
  current_profile_auth_mode?: string;
  current_profile_auth_source?: string;
  current_auth_name?: string;
  login_state: RawCodexLoginState;
  auth_state: RawCodexAuthState;
  profile_label: string;
  auth_label: string;
}

interface RawCodexRuntimeSnapshot {
  runtime_summary: RawCodexRuntimeSummary;
  auth_state: RawCodexAuthState;
  current_auth_info?: RawCodexCurrentAuthInfo;
}

interface LocalCodexProfile {
  provider?: string;
  provider_type?: string;
  base_url?: string;
  auth_token?: string;
  auth_mode?: string;
  requires_openai_auth?: boolean;
  env_key?: string;
  openai_login_method?: string;
  forced_login_method?: string;
  login_method?: string;
  openai_auth_method?: string;
}

interface LocalCodexProfilesShape {
  current_config?: string;
  [key: string]: unknown;
}

interface LocalCodexAuthRegistryEntry {
  account_id?: string;
  auth_method?: string;
  email?: string;
  saved_at?: string;
  last_used?: string;
  last_refresh?: string;
  expires_at?: string;
}

interface LocalCodexAuthRegistryShape {
  current_auth?: string;
  accounts?: Record<string, LocalCodexAuthRegistryEntry>;
}

interface LocalCodexRuntimeTokens {
  account_id?: string;
  access_token?: string;
  id_token?: string;
  refresh_token?: string;
}

interface LocalCodexRuntimeAuthShape {
  OPENAI_API_KEY?: string;
  tokens?: LocalCodexRuntimeTokens;
  last_refresh?: string;
  [key: string]: unknown;
}

type LocalRuntimeAuthKind =
  | { kind: "openai_api_key" }
  | { kind: "openai_chatgpt"; accountId?: string; matchedAccountName?: string }
  | { kind: "provider_env_key"; envKey: string }
  | { kind: "none" };

let cachedSnapshot: CodexRuntimeSnapshot | null = null;
let cachedError: string | null = null;
let cachedAt = 0;
let inflight: Promise<CodexRuntimeSnapshot | null> | null = null;

function isCacheFresh(): boolean {
  return cachedAt > 0 && Date.now() - cachedAt < RUNTIME_CACHE_TTL_MS;
}

function isNotFoundError(error: unknown): boolean {
  if (!error || typeof error !== "object") return false;
  if (!("code" in error)) return false;
  return (error as { code?: string }).code === "ENOENT";
}

function asNonEmptyString(value: unknown): string | undefined {
  return typeof value === "string" && value.trim().length > 0
    ? value.trim()
    : undefined;
}

function asBoolean(value: unknown): boolean | undefined {
  return typeof value === "boolean"
    ? value
    : undefined;
}

function canonicalAuthMode(value?: string): string | undefined {
  switch (value?.trim().toLowerCase()) {
    case "openai_chatgpt":
    case "chatgpt":
      return "openai_chatgpt";
    case "openai_api_key":
    case "api":
    case "api_key":
      return "openai_api_key";
    case "provider_env_key":
      return "provider_env_key";
    case "no_auth":
      return "no_auth";
    default:
      return undefined;
  }
}

function canonicalOpenAiMethod(value?: string): "api" | "chatgpt" | undefined {
  switch (value?.trim().toLowerCase()) {
    case "api":
    case "api_key":
      return "api";
    case "chatgpt":
      return "chatgpt";
    default:
      return undefined;
  }
}

function isOfficialProfile(profile?: LocalCodexProfile): boolean {
  if (!profile) {
    return false;
  }

  if (profile.provider_type?.trim().toLowerCase() === "official_relay") {
    return true;
  }

  return !profile.base_url?.trim();
}

function resolveProfileAuthMode(profile?: LocalCodexProfile): string {
  const explicit = canonicalAuthMode(profile?.auth_mode);
  if (explicit) {
    return explicit;
  }

  if (isOfficialProfile(profile)) {
    return profile?.auth_token?.trim()
      ? "openai_api_key"
      : "openai_chatgpt";
  }

  if (profile?.requires_openai_auth) {
    return canonicalOpenAiMethod(
      profile.openai_login_method
      ?? profile.forced_login_method
      ?? profile.login_method
      ?? profile.openai_auth_method,
    ) === "api"
      ? "openai_api_key"
      : "openai_chatgpt";
  }

  if (profile?.env_key?.trim()) {
    return "provider_env_key";
  }

  return "no_auth";
}

function resolveProfileAuthSource(profile: LocalCodexProfile | undefined, mode: string): string {
  switch (mode) {
    case "openai_chatgpt":
      return "openai_chatgpt";
    case "openai_api_key":
      return "openai_api_key";
    case "provider_env_key":
      return profile?.env_key?.trim()
        ? `provider:${profile.env_key.trim()}`
        : "provider";
    default:
      return "none";
  }
}

function buildProfileLabel(profileName?: string, profile?: LocalCodexProfile): string {
  if (!profileName) {
    return "未绑定";
  }

  const parts = [profileName];
  const provider = profile?.provider?.trim();
  if (provider && provider !== profileName) {
    parts.push(provider);
  }
  return parts.join(" · ");
}

function buildExpectedAuthLabel(profileMode: string, profile?: LocalCodexProfile): string {
  switch (profileMode) {
    case "openai_chatgpt":
      return "未登录 · OpenAI / ChatGPT";
    case "openai_api_key":
      return "未登录 · OpenAI / API Key";
    case "provider_env_key":
      return profile?.env_key?.trim()
        ? `Provider / ${profile.env_key.trim()}`
        : "Provider Key";
    default:
      return "No Auth";
  }
}

function computeFreshness(lastRefresh?: string): string {
  if (!lastRefresh) {
    return "Unknown";
  }

  const parsed = Date.parse(lastRefresh);
  if (Number.isNaN(parsed)) {
    return "Unknown";
  }

  const ageMs = Date.now() - parsed;
  if (ageMs < 24 * 60 * 60 * 1000) {
    return "Fresh";
  }
  if (ageMs < 7 * 24 * 60 * 60 * 1000) {
    return "Stale";
  }
  return "Old";
}

function fingerprintApiKey(apiKey: string): string {
  const trimmed = apiKey.trim();
  if (!trimmed) {
    return "api:unknown";
  }

  const suffix = trimmed.slice(-4);
  return `api:***${suffix}:len${trimmed.length}`;
}

function buildCapabilityWarnings(
  binaryPath: string | undefined,
  missingCurrentJson: boolean,
): string[] {
  const warnings: string[] = [];

  if (!binaryPath) {
    warnings.push("CCR CLI not found in PATH; using local fallback.");
    return warnings;
  }

  if (missingCurrentJson) {
    warnings.push("Current CCR CLI does not support `codex auth current --json`; using local fallback.");
  }

  return warnings;
}

function normalizeAuthState(raw: RawCodexAuthState): CodexAuthStateInfo {
  return {
    intent: {
      ...raw.intent,
    },
    store: raw.store,
    status: raw.status,
    reason: raw.reason,
  };
}

function normalizeLoginState(raw: RawCodexLoginState): CodexLoginStateInfo {
  return {
    ...raw,
  };
}

function normalizeCurrentAuthInfo(raw?: RawCodexCurrentAuthInfo): CodexCurrentAuthInfo | undefined {
  if (!raw) {
    return undefined;
  }

  return {
    accountId: raw.account_id,
    authMethod: raw.auth_method,
    email: raw.email,
    lastRefresh: raw.last_refresh,
    freshness: raw.freshness,
  };
}

function normalizeRuntimeSummary(raw: RawCodexRuntimeSummary): CodexRuntimeSummaryInfo {
  return {
    mode: raw.mode,
    currentProfileName: raw.current_profile_name,
    currentProfileProvider: raw.current_profile_provider,
    currentProfileAuthMode: raw.current_profile_auth_mode,
    currentProfileAuthSource: raw.current_profile_auth_source,
    currentAuthName: raw.current_auth_name,
    loginState: normalizeLoginState(raw.login_state),
    authState: normalizeAuthState(raw.auth_state),
    profileLabel: raw.profile_label,
    authLabel: raw.auth_label,
  };
}

function normalizeSnapshot(
  raw: RawCodexRuntimeSnapshot,
  binaryPath?: string,
): CodexRuntimeSnapshot {
  return {
    runtimeSummary: normalizeRuntimeSummary(raw.runtime_summary),
    authState: normalizeAuthState(raw.auth_state),
    currentAuthInfo: normalizeCurrentAuthInfo(raw.current_auth_info),
    authSidecarLabel: undefined,
    dataSource: "cli_json",
    binaryPath,
    capabilityWarnings: [],
  };
}

async function readLocalToml<T>(filePath: string): Promise<T | null> {
  try {
    const content = await fs.promises.readFile(filePath, "utf-8");
    return TOML.parse(content) as unknown as T;
  } catch (error) {
    if (isNotFoundError(error)) {
      return null;
    }
    throw error;
  }
}

async function readLocalJson<T>(filePath: string): Promise<T | null> {
  try {
    const content = await fs.promises.readFile(filePath, "utf-8");
    return JSON.parse(content) as T;
  } catch (error) {
    if (isNotFoundError(error)) {
      return null;
    }
    throw error;
  }
}

function resolveMatchedAccountName(
  auth: LocalCodexRuntimeAuthShape | null,
  registry: LocalCodexAuthRegistryShape | null,
): string | undefined {
  const accounts = registry?.accounts ?? {};
  const runtimeAccountId = auth?.tokens?.account_id?.trim();

  if (runtimeAccountId) {
    const matched = Object.entries(accounts).find(([, entry]) => entry.account_id === runtimeAccountId);
    if (matched) {
      return matched[0];
    }
  }

  const currentAuth = registry?.current_auth?.trim();
  return currentAuth && accounts[currentAuth]
    ? currentAuth
    : undefined;
}

function resolveLocalRuntimeAuthKind(
  auth: LocalCodexRuntimeAuthShape | null,
  profile: LocalCodexProfile | undefined,
  matchedAccountName: string | undefined,
): LocalRuntimeAuthKind {
  const apiKey = asNonEmptyString(auth?.OPENAI_API_KEY);
  if (apiKey) {
    return { kind: "openai_api_key" };
  }

  const tokenAccountId = asNonEmptyString(auth?.tokens?.account_id);
  const hasOAuthTokens = Boolean(
    tokenAccountId
    || asNonEmptyString(auth?.tokens?.access_token)
    || asNonEmptyString(auth?.tokens?.id_token)
    || asNonEmptyString(auth?.tokens?.refresh_token),
  );
  if (hasOAuthTokens) {
    return {
      kind: "openai_chatgpt",
      accountId: tokenAccountId,
      matchedAccountName,
    };
  }

  const envKey = profile?.env_key?.trim();
  if (envKey && asNonEmptyString(auth?.[envKey])) {
    return {
      kind: "provider_env_key",
      envKey,
    };
  }

  return { kind: "none" };
}

function buildFallbackSnapshot(
  profileName: string | undefined,
  profile: LocalCodexProfile | undefined,
  registry: LocalCodexAuthRegistryShape | null,
  auth: LocalCodexRuntimeAuthShape | null,
  binaryPath: string | undefined,
  capabilityWarnings: string[],
): CodexRuntimeSnapshot {
  const matchedAccountName = resolveMatchedAccountName(auth, registry);
  const runtimeAuthKind = resolveLocalRuntimeAuthKind(auth, profile, matchedAccountName);
  const profileMode = resolveProfileAuthMode(profile);
  const profileSource = resolveProfileAuthSource(profile, profileMode);

  let controlMode: string;
  if (!profileName) {
    controlMode = runtimeAuthKind.kind === "none"
      ? "unresolved"
      : "runtime_only";
  } else if (profileMode === "openai_chatgpt" || profileMode === "openai_api_key") {
    controlMode = runtimeAuthKind.kind === "none"
      ? "profile_pending_auth"
      : "profile_with_auth";
  } else {
    controlMode = "profile_only";
  }

  let authState: CodexAuthStateInfo;
  let loginState: CodexLoginStateInfo;
  let authLabel: string;
  let currentAuthName: string | undefined;
  let currentAuthInfo: CodexCurrentAuthInfo | undefined;

  switch (runtimeAuthKind.kind) {
    case "openai_api_key":
      authState = {
        intent: {
          kind: "open_ai_auth",
          method: "api",
        },
        store: "file",
        status: "valid",
        reason: "Detected OPENAI_API_KEY via local fallback",
      };
      loginState = {
        type: "ApiKeyActive",
      };
      authLabel = "OpenAI / API Key";
      currentAuthInfo = {
        accountId: fingerprintApiKey(auth?.OPENAI_API_KEY ?? ""),
        authMethod: "api",
        freshness: computeFreshness(auth?.last_refresh),
        lastRefresh: auth?.last_refresh,
      };
      break;
    case "openai_chatgpt":
      currentAuthName = runtimeAuthKind.matchedAccountName;
      authState = {
        intent: {
          kind: "open_ai_auth",
          method: "chatgpt",
        },
        store: "file",
        status: "valid",
        reason: "Detected OAuth tokens via local fallback",
      };
      loginState = currentAuthName
        ? {
            type: "LoggedInSaved",
            account_name: currentAuthName,
          }
        : {
            type: "LoggedInUnsaved",
          };
      authLabel = currentAuthName
        ? `${currentAuthName} · OpenAI / ChatGPT`
        : "未保存账号 · OpenAI / ChatGPT";
      currentAuthInfo = {
        accountId: runtimeAuthKind.accountId ?? "unknown",
        authMethod: "chatgpt",
        email: currentAuthName
          ? registry?.accounts?.[currentAuthName]?.email
          : undefined,
        lastRefresh: auth?.last_refresh,
        freshness: computeFreshness(auth?.last_refresh),
      };
      break;
    case "provider_env_key":
      authState = {
        intent: {
          kind: "provider_env_key",
          env_key: runtimeAuthKind.envKey,
        },
        store: "file",
        status: "valid",
        reason: `Detected ${runtimeAuthKind.envKey} via local fallback`,
      };
      loginState = {
        type: "ProviderKeyActive",
        env_key: runtimeAuthKind.envKey,
      };
      authLabel = `Provider / ${runtimeAuthKind.envKey}`;
      break;
    case "none":
    default:
      authState = {
        intent: profileMode === "provider_env_key" && profile?.env_key
          ? {
              kind: "provider_env_key",
              env_key: profile.env_key,
            }
          : {
              kind: "no_auth",
            },
        store: "file",
        status: "missing",
        reason: "No runtime auth material detected via local fallback",
      };
      loginState = profileMode === "provider_env_key" && profile?.env_key
        ? {
            type: "ProviderKeyConfigured",
            env_key: profile.env_key,
          }
        : {
            type: "NotLoggedIn",
          };
      authLabel = buildExpectedAuthLabel(profileMode, profile);
      break;
  }

  const authSidecarLabel = (
    (runtimeAuthKind.kind === "openai_api_key" || runtimeAuthKind.kind === "openai_chatgpt")
    && !(profileMode === "openai_api_key" || profileMode === "openai_chatgpt")
  )
    ? runtimeAuthKind.kind === "openai_api_key"
      ? "Runtime API Key active"
      : "Runtime ChatGPT auth active"
    : undefined;

  return {
    runtimeSummary: {
      mode: controlMode,
      currentProfileName: profileName,
      currentProfileProvider: profile?.provider?.trim(),
      currentProfileAuthMode: profileMode,
      currentProfileAuthSource: profileSource,
      currentAuthName,
      loginState,
      authState,
      profileLabel: buildProfileLabel(profileName, profile),
      authLabel,
    },
    authState,
    currentAuthInfo,
    authSidecarLabel,
    dataSource: "local_fallback",
    binaryPath,
    capabilityWarnings,
  };
}

async function loadLocalFallbackSnapshot(
  binaryPath: string | undefined,
  capabilityWarnings = buildCapabilityWarnings(binaryPath, true),
): Promise<CodexRuntimeSnapshot> {
  const [profilesRaw, registryRaw, authRaw] = await Promise.all([
    readLocalToml<LocalCodexProfilesShape>(getProfilesPath("codex")),
    readLocalToml<LocalCodexAuthRegistryShape>(getCodexAuthRegistryPath()),
    readLocalJson<LocalCodexRuntimeAuthShape>(getCodexRuntimeAuthPath()),
  ]);

  const currentProfileName = asNonEmptyString(profilesRaw?.current_config);
  const currentProfile = currentProfileName
    ? profilesRaw?.[currentProfileName] as LocalCodexProfile | undefined
    : undefined;

  return buildFallbackSnapshot(
    currentProfileName,
    currentProfile,
    registryRaw,
    authRaw,
    binaryPath,
    capabilityWarnings,
  );
}

export function getCachedCodexRuntimeSnapshot(): CodexRuntimeSnapshot | null {
  return cachedSnapshot;
}

export function getCodexRuntimeError(): string | null {
  return cachedError;
}

export function invalidateCodexRuntimeCache(): void {
  cachedSnapshot = null;
  cachedAt = 0;
  cachedError = null;
}

export function ensureCodexRuntimeSnapshot(onChange?: () => void): void {
  if (isCacheFresh() || inflight) {
    return;
  }

  inflight = (async () => {
    const capabilities = await detectCcrCapabilities();

    if (capabilities.supportsCodexAuthCurrentJson) {
      const result = await execCodexAuthCurrentJson<RawCodexRuntimeSnapshot>();
      if (result.success && result.data) {
        return normalizeSnapshot(result.data, capabilities.binaryPath);
      }

      return loadLocalFallbackSnapshot(capabilities.binaryPath, [
        result.stderr || "CLI JSON runtime probe failed; using local fallback.",
      ]);
    }

    return loadLocalFallbackSnapshot(capabilities.binaryPath);
  })()
    .then((snapshot) => {
      cachedSnapshot = snapshot;
      cachedError = null;
      cachedAt = Date.now();
      return snapshot;
    })
    .catch((error) => {
      cachedError = error instanceof Error ? error.message : String(error);
      cachedAt = Date.now();
      return cachedSnapshot;
    })
    .finally(() => {
      inflight = null;
      onChange?.();
    });
}
