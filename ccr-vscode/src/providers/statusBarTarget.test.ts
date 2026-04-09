import { describe, it } from "node:test";
import assert from "node:assert/strict";
import type { PlatformInfo } from "../models/types";
import { getSupportedStatusBarPlatforms, normalizeStatusBarMode, resolveStatusBarItems, resolveStatusBarTarget } from "./statusBarTarget";

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
  {
    name: "gemini",
    displayName: "Gemini",
    icon: "✨",
    enabled: true,
    currentProfile: "flash",
  },
];

describe("statusBarTarget", () => {
  it("normalizes invalid mode to pinned", () => {
    assert.equal(normalizeStatusBarMode("unexpected"), "pinned");
    assert.equal(normalizeStatusBarMode(undefined), "pinned");
  });

  it("keeps only supported status bar platforms", () => {
    assert.deepEqual(
      getSupportedStatusBarPlatforms(platforms).map((platform) => platform.name),
      ["claude", "codex"],
    );
  });

  it("resolves both items by default in pinned mode", () => {
    const result = resolveStatusBarItems({
      platforms,
      currentPlatform: "claude",
      mode: "pinned",
    });

    assert.deepEqual(result.map((item) => item.platform?.name), ["claude", "codex"]);
  });

  it("follows only the current platform in current mode", () => {
    const result = resolveStatusBarItems({
      platforms,
      currentPlatform: "codex",
      mode: "current",
    });

    assert.deepEqual(result.map((item) => item.platform?.name), ["codex"]);
  });

  it("falls back to first enabled platform in current mode when current platform is unsupported", () => {
    const result = resolveStatusBarItems({
      platforms,
      currentPlatform: "gemini",
      mode: "current",
      showClaude: false,
      showCodex: true,
    });

    assert.deepEqual(result.map((item) => item.platform?.name), ["codex"]);
  });

  it("hides all items in hidden mode", () => {
    const result = resolveStatusBarItems({
      platforms,
      currentPlatform: "claude",
      mode: "hidden",
    });

    assert.deepEqual(result, []);
  });

  it("can disable codex item independently", () => {
    const result = resolveStatusBarItems({
      platforms,
      currentPlatform: "claude",
      mode: "pinned",
      showClaude: true,
      showCodex: false,
    });

    assert.deepEqual(result.map((item) => item.platform?.name), ["claude"]);
  });

  it("keeps single target compatibility with first visible platform", () => {
    const result = resolveStatusBarTarget({
      platforms,
      currentPlatform: "claude",
      mode: "pinned",
      showClaude: false,
      showCodex: true,
    });

    assert.equal(result.visible, true);
    assert.equal(result.platform?.name, "codex");
  });
});
