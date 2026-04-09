import { describe, it } from "node:test";
import assert from "node:assert/strict";
import type { CodexRuntimeSnapshot, PlatformInfo, ProfileInfo } from "../models/types";
import { buildStatusBarText, buildStatusBarTooltipLines, getCompactPlatformLabel } from "./statusBarPresentation";

const claudePlatform: PlatformInfo = {
  name: "claude",
  displayName: "Claude Code",
  icon: "🤖",
  enabled: true,
  currentProfile: "dev",
};

const codexPlatform: PlatformInfo = {
  name: "codex",
  displayName: "Codex",
  icon: "💻",
  enabled: true,
  currentProfile: "52api",
};

const codexProfile: ProfileInfo = {
  name: "52api",
  platformName: "codex",
  model: "gpt-5.4",
  provider: "openai",
  usageCount: 12,
  enabled: true,
  isCurrent: true,
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
  authSidecarLabel: "Runtime API Key active",
  dataSource: "local_fallback",
  binaryPath: "C:/Users/lyh/.cargo/bin/ccr.exe",
  capabilityWarnings: [
    "Current CCR CLI does not support `codex auth current --json`; using local fallback.",
  ],
};

describe("statusBarPresentation", () => {
  it("builds compact platform labels", () => {
    assert.equal(getCompactPlatformLabel("claude"), "CC");
    assert.equal(getCompactPlatformLabel("codex"), "CDX");
  });

  it("builds a Claude compact status bar label", () => {
    assert.equal(buildStatusBarText(claudePlatform, "dev"), "CC: dev");
  });

  it("builds a compact Codex status bar label", () => {
    assert.equal(
      buildStatusBarText(codexPlatform, "52api", runtimeSnapshot),
      "CDX: 52api",
    );
  });

  it("adds runtime-specific tooltip lines for Codex", () => {
    const tooltipLines = buildStatusBarTooltipLines(
      "pinned",
      codexPlatform,
      "52api",
      codexProfile,
      runtimeSnapshot,
    );

    assert.ok(tooltipLines.includes("Profile: 52api"));
    assert.ok(tooltipLines.includes("Auth: OpenAI / API Key"));
    assert.ok(tooltipLines.includes("Control: profile_only"));
    assert.ok(tooltipLines.includes("Sidecar: Runtime API Key active"));
    assert.ok(tooltipLines.includes("Source: local_fallback"));
  });
});
