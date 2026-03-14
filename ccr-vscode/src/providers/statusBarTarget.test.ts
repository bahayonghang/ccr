import { describe, it } from "node:test";
import assert from "node:assert/strict";
import type { PlatformInfo } from "../models/types";
import { normalizeStatusBarMode, resolveStatusBarTarget } from "./statusBarTarget";

const platforms: PlatformInfo[] = [
  {
    name: "claude",
    displayName: "Claude Code",
    icon: "🤖",
    enabled: true,
    currentProfile: "primary",
  },
  {
    name: "codex",
    displayName: "Codex",
    icon: "💻",
    enabled: true,
    currentProfile: "default",
  },
];

describe("statusBarTarget", () => {
  it("normalizes invalid mode to pinned", () => {
    assert.equal(normalizeStatusBarMode("unexpected"), "pinned");
    assert.equal(normalizeStatusBarMode(undefined), "pinned");
  });

  it("selects the pinned platform when configured", () => {
    const result = resolveStatusBarTarget({
      platforms,
      currentPlatform: "claude",
      mode: "pinned",
      pinnedPlatform: "codex",
    });

    assert.equal(result.visible, true);
    assert.equal(result.mode, "pinned");
    assert.equal(result.platform?.name, "codex");
    assert.equal(result.warning, undefined);
  });

  it("follows the current platform in current mode", () => {
    const result = resolveStatusBarTarget({
      platforms,
      currentPlatform: "codex",
      mode: "current",
    });

    assert.equal(result.visible, true);
    assert.equal(result.mode, "current");
    assert.equal(result.platform?.name, "codex");
  });

  it("hides the status bar in hidden mode", () => {
    const result = resolveStatusBarTarget({
      platforms,
      currentPlatform: "claude",
      mode: "hidden",
      pinnedPlatform: "codex",
    });

    assert.deepEqual(result, {
      mode: "hidden",
      visible: false,
    });
  });

  it("falls back to the current platform when pinned platform is empty", () => {
    const result = resolveStatusBarTarget({
      platforms,
      currentPlatform: "claude",
      mode: "pinned",
      pinnedPlatform: "   ",
    });

    assert.equal(result.visible, true);
    assert.equal(result.platform?.name, "claude");
    assert.equal(result.warning, undefined);
  });

  it("falls back and warns when pinned platform is missing", () => {
    const result = resolveStatusBarTarget({
      platforms,
      currentPlatform: "claude",
      mode: "pinned",
      pinnedPlatform: "gemini",
    });

    assert.equal(result.visible, true);
    assert.equal(result.platform?.name, "claude");
    assert.match(result.warning ?? "", /gemini/);
  });
});
