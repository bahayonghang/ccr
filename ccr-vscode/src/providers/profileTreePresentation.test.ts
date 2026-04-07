import { describe, it } from "node:test";
import assert from "node:assert/strict";
import type {
  CodexAuthQuotaInfo,
  CodexRuntimeSnapshot,
  PlatformInfo,
} from "../models/types";
import {
  buildCodexAuthDetailDescriptors,
  buildCodexRuntimeDetails,
  formatCodexAuthDescription,
  formatQuotaReset,
  getCodexPlatformDescription,
} from "./profileTreePresentation";

const codexPlatform: PlatformInfo = {
  name: "codex",
  displayName: "Codex",
  icon: "💻",
  enabled: true,
  currentProfile: "52api",
};

const runtimeSnapshot: CodexRuntimeSnapshot = {
  runtimeSummary: {
    mode: "profile_only",
    currentProfileName: "52api",
    currentProfileProvider: "52api",
    currentProfileAuthMode: "no_auth",
    currentProfileAuthSource: "none",
    currentAuthName: undefined,
    loginState: {
      type: "ApiKeyActive",
    },
    authState: {
      intent: {
        kind: "open_ai_auth",
        method: "api",
      },
      store: "file",
      status: "valid",
      reason: "ok",
    },
    profileLabel: "52api",
    authLabel: "OpenAI / API Key",
  },
  authState: {
    intent: {
      kind: "open_ai_auth",
      method: "api",
    },
    store: "file",
    status: "valid",
    reason: "ok",
  },
  currentAuthInfo: {
    accountId: "api:***bc93:len67",
    freshness: "Unknown",
  },
  authSidecarLabel: "Runtime API Key active",
  dataSource: "local_fallback",
  binaryPath: "C:/Users/lyh/.cargo/bin/ccr.exe",
  capabilityWarnings: [
    "Current CCR CLI does not support `codex auth current --json`; using local fallback.",
  ],
};

describe("profileTreePresentation", () => {
  it("formats Codex platform description from runtime summary", () => {
    assert.equal(
      getCodexPlatformDescription(codexPlatform, runtimeSnapshot),
      "ProfileOnly · 52api",
    );
  });

  it("formats auth description from quota first", () => {
    const quota: CodexAuthQuotaInfo = {
      accountName: "aiuc-team",
      quota: {
        hourlyPercentage: 82,
        weeklyPercentage: 61,
        planType: "Plus",
      },
      fetchedAt: "2026-04-07T12:00:00Z",
    };

    assert.equal(
      formatCodexAuthDescription(quota),
      "Plus",
    );
  });

  it("keeps auth description empty when quota exists without a plan label", () => {
    const quota: CodexAuthQuotaInfo = {
      accountName: "aiuc-team",
      quota: {
        hourlyPercentage: 82,
        weeklyPercentage: 61,
      },
      fetchedAt: "2026-04-07T12:00:00Z",
    };

    assert.equal(
      formatCodexAuthDescription(quota),
      undefined,
    );
  });

  it("surfaces quota errors instead of falling back to email text", () => {
    assert.equal(
      formatCodexAuthDescription({
        accountName: "aiuc-team",
        error: "unsupported",
        fetchedAt: "2026-04-07T12:00:00Z",
      }),
      "quota unavailable",
    );
  });

  it("builds quota detail descriptors with reset times", () => {
    const hourlyResetTime = 1_744_020_800;
    const weeklyResetTime = 1_744_625_600;
    const quota: CodexAuthQuotaInfo = {
      accountName: "aiuc-team",
      quota: {
        hourlyPercentage: 82,
        hourlyResetTime,
        weeklyPercentage: 61,
        weeklyResetTime,
        planType: "Plus",
      },
      fetchedAt: "2026-04-07T12:00:00Z",
    };

    const details = buildCodexAuthDetailDescriptors(quota);

    assert.equal(details.length, 2);
    assert.deepEqual(
      details.map((detail) => detail.label),
      ["5h", "7d"],
    );
    assert.equal(
      details[0]?.description,
      `82% · reset ${formatQuotaReset(hourlyResetTime)}`,
    );
    assert.equal(
      details[1]?.description,
      `61% · reset ${formatQuotaReset(weeklyResetTime)}`,
    );
  });

  it("marks missing reset times as unavailable in quota detail descriptors", () => {
    const quota: CodexAuthQuotaInfo = {
      accountName: "simple_apple",
      quota: {
        hourlyPercentage: 44,
        weeklyPercentage: 100,
      },
      fetchedAt: "2026-04-07T12:00:00Z",
    };

    const details = buildCodexAuthDetailDescriptors(quota);

    assert.equal(details[0]?.description, "44% · reset unavailable");
    assert.equal(details[1]?.description, "100% · reset unavailable");
  });

  it("builds loading and unavailable quota detail fallbacks", () => {
    const loading = buildCodexAuthDetailDescriptors();
    const unavailable = buildCodexAuthDetailDescriptors(undefined, "CCR CLI not found in PATH.");

    assert.deepEqual(
      loading.map((detail) => ({
        label: detail.label,
        description: detail.description,
      })),
      [{
        label: "Quota loading…",
        description: "Waiting for CCR CLI snapshot",
      }],
    );
    assert.deepEqual(
      unavailable.map((detail) => ({
        label: detail.label,
        description: detail.description,
      })),
      [{
        label: "Quota unavailable",
        description: "CCR CLI not found in PATH.",
      }],
    );
  });

  it("builds runtime details using fallback source and sidecar state", () => {
    const details = buildCodexRuntimeDetails(codexPlatform, runtimeSnapshot, null);

    assert.equal(details[0]?.label, "Control Mode");
    assert.equal(details[0]?.description, "ProfileOnly");
    assert.equal(details[1]?.description, "52api");
    assert.equal(details[2]?.description, "OpenAI / API Key");
    assert.equal(details[3]?.description, "Runtime API Key active");
    assert.equal(details[4]?.description, "Local fallback");
    assert.match(details[4]?.tooltip ?? "", /does not support `codex auth current --json`/);
  });
});
