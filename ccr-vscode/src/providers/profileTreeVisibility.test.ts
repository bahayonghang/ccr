import { describe, it } from "node:test";
import assert from "node:assert/strict";
import type { PlatformInfo, ProfileInfo, TreeSectionInfo } from "../models/types";
import {
  getPlatformNodeContextValue,
  getProfileNodeContextValue,
  getSectionNodeContextValue,
  supportsProfileMutation,
} from "./profileTreeVisibility";

const writablePlatform: PlatformInfo = {
  name: "claude",
  displayName: "Claude Code",
  icon: "🤖",
  enabled: true,
  currentProfile: "dev",
};

const readonlyPlatform: PlatformInfo = {
  name: "gemini",
  displayName: "Antigravity CLI",
  icon: "✨",
  enabled: true,
  currentProfile: "flash",
};

describe("profileTreeVisibility", () => {
  it("keeps mutation support limited to Claude and Codex", () => {
    assert.equal(supportsProfileMutation("claude"), true);
    assert.equal(supportsProfileMutation("codex"), true);
    assert.equal(supportsProfileMutation("gemini"), false);
    assert.equal(supportsProfileMutation("qwen"), false);
    assert.equal(supportsProfileMutation("droid"), false);
  });

  it("marks writable and read-only platform nodes differently", () => {
    assert.equal(getPlatformNodeContextValue(writablePlatform), "platform-create-supported");
    assert.equal(getPlatformNodeContextValue(readonlyPlatform), "platform");
    assert.equal(
      getPlatformNodeContextValue({ ...readonlyPlatform, enabled: false }),
      "platform-disabled",
    );
  });

  it("marks writable and read-only sections differently", () => {
    const writableSection: TreeSectionInfo = {
      kind: "profiles",
      platformName: "codex",
      label: "Codex Profiles",
      description: "Switch and manage Codex profiles",
    };
    const readonlySection: TreeSectionInfo = {
      kind: "profiles",
      platformName: "droid",
      label: "Factory Droid Profiles",
      description: "Browse and inspect Factory Droid profiles",
    };

    assert.equal(getSectionNodeContextValue(writableSection), "section-profiles-create-supported");
    assert.equal(getSectionNodeContextValue(readonlySection), "section-profiles");
  });

  it("marks writable and read-only profiles differently", () => {
    const writableProfile: ProfileInfo = {
      name: "dev",
      platformName: "claude",
      usageCount: 0,
      enabled: true,
      isCurrent: false,
    };
    const readonlyProfile: ProfileInfo = {
      name: "flash",
      platformName: "gemini",
      usageCount: 0,
      enabled: true,
      isCurrent: false,
    };

    assert.equal(getProfileNodeContextValue(writableProfile), "profile-supported");
    assert.equal(getProfileNodeContextValue({ ...writableProfile, isCurrent: true }), "profile-current-supported");
    assert.equal(getProfileNodeContextValue(readonlyProfile), "readonly-profile");
    assert.equal(
      getProfileNodeContextValue({ ...readonlyProfile, isCurrent: true }),
      "readonly-profile-current",
    );
  });
});
