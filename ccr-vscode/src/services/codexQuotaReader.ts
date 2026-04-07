import type { CodexAuthQuotaInfo, CodexQuotaInfo } from "../models/types";
import { detectCcrCapabilities, execCodexQuotaJson } from "./ccrCli";

const QUOTA_CACHE_TTL_MS = 30_000;

interface RawCodexQuotaInfo {
  hourly_percentage: number;
  hourly_reset_time?: number;
  hourly_window_minutes?: number;
  hourly_window_present?: boolean;
  weekly_percentage: number;
  weekly_reset_time?: number;
  weekly_window_minutes?: number;
  weekly_window_present?: boolean;
  plan_type?: string;
}

interface RawCodexAuthQuotaInfo {
  account_name: string;
  email?: string;
  quota?: RawCodexQuotaInfo;
  error?: string;
  fetched_at: string;
}

let cachedByAccount: Record<string, CodexAuthQuotaInfo> = {};
let cachedError: string | null = null;
let cachedAt = 0;
let inflight: Promise<Record<string, CodexAuthQuotaInfo>> | null = null;

function isCacheFresh(): boolean {
  return cachedAt > 0 && Date.now() - cachedAt < QUOTA_CACHE_TTL_MS;
}

function normalizeQuota(raw?: RawCodexQuotaInfo): CodexQuotaInfo | undefined {
  if (!raw) {
    return undefined;
  }

  return {
    hourlyPercentage: raw.hourly_percentage,
    hourlyResetTime: raw.hourly_reset_time,
    hourlyWindowMinutes: raw.hourly_window_minutes,
    hourlyWindowPresent: raw.hourly_window_present,
    weeklyPercentage: raw.weekly_percentage,
    weeklyResetTime: raw.weekly_reset_time,
    weeklyWindowMinutes: raw.weekly_window_minutes,
    weeklyWindowPresent: raw.weekly_window_present,
    planType: raw.plan_type,
  };
}

function normalizeQuotas(rawItems: RawCodexAuthQuotaInfo[]): Record<string, CodexAuthQuotaInfo> {
  const next: Record<string, CodexAuthQuotaInfo> = {};

  for (const item of rawItems) {
    next[item.account_name] = {
      accountName: item.account_name,
      email: item.email,
      quota: normalizeQuota(item.quota),
      error: item.error,
      fetchedAt: item.fetched_at,
    };
  }

  return next;
}

export function getCachedCodexQuotaByAccount(): Record<string, CodexAuthQuotaInfo> {
  return cachedByAccount;
}

export function getCodexQuotaError(): string | null {
  return cachedError;
}

export function invalidateCodexQuotaCache(): void {
  cachedByAccount = {};
  cachedAt = 0;
  cachedError = null;
}

export function ensureCodexQuotaSnapshot(onChange?: () => void): void {
  if (isCacheFresh() || inflight) {
    return;
  }

  inflight = detectCcrCapabilities()
    .then(async (capabilities) => {
      if (!capabilities.supportsCodexQuotaJson) {
        cachedError = capabilities.binaryPath
          ? "Current CCR CLI does not support `codex quota --json`."
          : "CCR CLI not found in PATH.";
        cachedAt = Date.now();
        return cachedByAccount;
      }

      const result = await execCodexQuotaJson<RawCodexAuthQuotaInfo[]>();
      if (!result.success || !result.data) {
        cachedError = result.stderr || "Failed to load Codex auth quota.";
        cachedAt = Date.now();
        return cachedByAccount;
      }

      cachedByAccount = normalizeQuotas(result.data);
      cachedError = null;
      cachedAt = Date.now();
      return cachedByAccount;
    })
    .catch((error) => {
      cachedError = error instanceof Error ? error.message : String(error);
      cachedAt = Date.now();
      return cachedByAccount;
    })
    .finally(() => {
      inflight = null;
      onChange?.();
    });
}
