/**
 * Unit tests for path resolution and TOML read/write
 *
 * Uses Node built-in test runner (node:test)
 */

import { describe, it, before, after } from "node:test";
import assert from "node:assert/strict";
import * as fs from "fs";
import * as path from "path";
import * as os from "os";

// ── Path resolution tests ──

describe("ccrPaths", () => {
  const originalEnv = process.env["CCR_ROOT"];

  after(() => {
    // Restore original env
    if (originalEnv !== undefined) {
      process.env["CCR_ROOT"] = originalEnv;
    } else {
      delete process.env["CCR_ROOT"];
    }
  });

  it("getCcrRoot returns $CCR_ROOT when set", async () => {
    process.env["CCR_ROOT"] = "/tmp/test-ccr-root";
    // Re-import to pick up env change
    const { getCcrRoot } = await import("../services/ccrPaths.js");
    assert.equal(getCcrRoot(), "/tmp/test-ccr-root");
  });

  it("getCcrRoot falls back to ~/.ccr/ when $CCR_ROOT is not set", async () => {
    delete process.env["CCR_ROOT"];
    const { getCcrRoot } = await import("../services/ccrPaths.js");
    const expected = path.join(os.homedir(), ".ccr");
    assert.equal(getCcrRoot(), expected);
  });

  it("getRegistryPath appends config.toml", async () => {
    process.env["CCR_ROOT"] = "/tmp/test-ccr";
    const { getRegistryPath } = await import("../services/ccrPaths.js");
    assert.equal(getRegistryPath(), path.join("/tmp/test-ccr", "config.toml"));
  });

  it("getProfilesPath builds correct platform path", async () => {
    process.env["CCR_ROOT"] = "/tmp/test-ccr";
    const { getProfilesPath } = await import("../services/ccrPaths.js");
    const result = getProfilesPath("claude");
    assert.equal(result, path.join("/tmp/test-ccr", "platforms", "claude", "profiles.toml"));
  });

  it("getPlatformDisplayName returns known platform names", async () => {
    const { getPlatformDisplayName } = await import("../services/ccrPaths.js");
    assert.equal(getPlatformDisplayName("claude"), "Claude Code");
    assert.equal(getPlatformDisplayName("codex"), "Codex");
    assert.equal(getPlatformDisplayName("gemini"), "Gemini CLI");
  });

  it("getPlatformDisplayName capitalizes unknown platforms", async () => {
    const { getPlatformDisplayName } = await import("../services/ccrPaths.js");
    assert.equal(getPlatformDisplayName("custom"), "Custom");
  });

  it("getPlatformIcon returns fallback for unknown platforms", async () => {
    const { getPlatformIcon } = await import("../services/ccrPaths.js");
    assert.equal(getPlatformIcon("unknown"), "\u{1F4E6}");
  });
});

// ── TOML read/write tests ──

describe("tomlReader", () => {
  let tmpDir: string;

  before(() => {
    tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "ccr-test-"));
    process.env["CCR_ROOT"] = tmpDir;
  });

  after(() => {
    // Cleanup temp dir
    fs.rmSync(tmpDir, { recursive: true, force: true });
    delete process.env["CCR_ROOT"];
  });

  describe("readRegistry", () => {
    it("returns null when config.toml does not exist", async () => {
      const { readRegistry } = await import("../services/tomlReader.js");
      assert.equal(readRegistry(), null);
    });

    it("parses config.toml with platform entries", async () => {
      const configToml = `
default_platform = "claude"
current_platform = "claude"

[claude]
enabled = true
current_profile = "anthropic"
description = "Claude Code"

[codex]
enabled = true
current_profile = "default"
`;
      fs.writeFileSync(path.join(tmpDir, "config.toml"), configToml, "utf-8");

      const { readRegistry } = await import("../services/tomlReader.js");
      const result = readRegistry();

      assert.notEqual(result, null);
      assert.equal(result!.platforms.length, 2);

      // Current platform (claude) should be sorted first
      assert.equal(result!.platforms[0].name, "claude");
      assert.equal(result!.platforms[0].enabled, true);
      assert.equal(result!.platforms[0].currentProfile, "anthropic");

      assert.equal(result!.platforms[1].name, "codex");
    });

    it("filters out top-level scalar keys from platform entries", async () => {
      const configToml = `
default_platform = "claude"
current_platform = "claude"

[claude]
enabled = true
`;
      fs.writeFileSync(path.join(tmpDir, "config.toml"), configToml, "utf-8");

      const { readRegistry } = await import("../services/tomlReader.js");
      const result = readRegistry();

      assert.notEqual(result, null);
      // Only platform entries, not default_platform/current_platform
      assert.equal(result!.platforms.length, 1);
      assert.equal(result!.platforms[0].name, "claude");
    });
  });

  describe("readProfiles", () => {
    it("returns empty array when profiles.toml does not exist", async () => {
      const { readProfiles } = await import("../services/tomlReader.js");
      const result = readProfiles("nonexistent");
      assert.deepEqual(result, []);
    });

    it("parses profiles.toml with CcsConfig format", async () => {
      const platformDir = path.join(tmpDir, "platforms", "claude");
      fs.mkdirSync(platformDir, { recursive: true });

      const profilesToml = `
default_config = "anthropic"
current_config = "anthropic"

[anthropic]
description = "Anthropic Official API"
base_url = "https://api.anthropic.com"
auth_token = "sk-ant-api03-test-key-1234"
model = "claude-sonnet-4-5-20250929"
provider = "Anthropic"
enabled = true

[relay]
description = "Relay Service"
base_url = "https://relay.example.com/v1"
auth_token = "relay-token-5678"
model = "claude-sonnet-4-5-20250929"
provider = "Relay"
enabled = false
`;
      fs.writeFileSync(path.join(platformDir, "profiles.toml"), profilesToml, "utf-8");

      const { readProfiles } = await import("../services/tomlReader.js");
      const profiles = readProfiles("claude");

      assert.equal(profiles.length, 2);

      const anthropic = profiles.find((p) => p.name === "anthropic");
      assert.notEqual(anthropic, undefined);
      assert.equal(anthropic!.description, "Anthropic Official API");
      assert.equal(anthropic!.baseUrl, "https://api.anthropic.com");
      assert.equal(anthropic!.model, "claude-sonnet-4-5-20250929");
      assert.equal(anthropic!.isCurrent, true);
      assert.equal(anthropic!.enabled, true);

      const relay = profiles.find((p) => p.name === "relay");
      assert.notEqual(relay, undefined);
      assert.equal(relay!.isCurrent, false);
      assert.equal(relay!.enabled, false);
    });

    it("filters CCS top-level keys from profile entries", async () => {
      const platformDir = path.join(tmpDir, "platforms", "codex");
      fs.mkdirSync(platformDir, { recursive: true });

      const profilesToml = `
default_config = "default"
current_config = "default"

[settings]
some_global = "value"

[default]
description = "Default Codex"
enabled = true
`;
      fs.writeFileSync(path.join(platformDir, "profiles.toml"), profilesToml, "utf-8");

      const { readProfiles } = await import("../services/tomlReader.js");
      const profiles = readProfiles("codex");

      // Only "default" profile, not "settings" or top-level keys
      assert.equal(profiles.length, 1);
      assert.equal(profiles[0].name, "default");
    });
  });

  describe("writeProfileField", () => {
    it("updates a string field and preserves structure", async () => {
      const platformDir = path.join(tmpDir, "platforms", "write-test");
      fs.mkdirSync(platformDir, { recursive: true });

      const profilesToml = `
default_config = "main"
current_config = "main"

[main]
description = "Original"
model = "old-model"
enabled = true
`;
      const filePath = path.join(platformDir, "profiles.toml");
      fs.writeFileSync(filePath, profilesToml, "utf-8");

      const { writeProfileField, readProfiles } = await import("../services/tomlReader.js");
      await writeProfileField("write-test", "main", "model", "new-model");

      // Re-read and verify
      const profiles = readProfiles("write-test");
      const main = profiles.find((p) => p.name === "main");
      assert.equal(main!.model, "new-model");
      assert.equal(main!.description, "Original"); // Preserved

      // Verify top-level fields preserved
      const raw = fs.readFileSync(filePath, "utf-8");
      assert.ok(raw.includes("default_config"));
      assert.ok(raw.includes("current_config"));
    });

    it("deletes a field when value is undefined", async () => {
      const platformDir = path.join(tmpDir, "platforms", "delete-test");
      fs.mkdirSync(platformDir, { recursive: true });

      const profilesToml = `
default_config = "main"
current_config = "main"

[main]
description = "Has model"
model = "to-be-removed"
enabled = true
`;
      fs.writeFileSync(path.join(platformDir, "profiles.toml"), profilesToml, "utf-8");

      const { writeProfileField, readProfiles } = await import("../services/tomlReader.js");
      await writeProfileField("delete-test", "main", "model", undefined);

      const profiles = readProfiles("delete-test");
      const main = profiles.find((p) => p.name === "main");
      assert.equal(main!.model, undefined);
    });

    it("deletes model when value is an empty string", async () => {
      const platformDir = path.join(tmpDir, "platforms", "empty-model-test");
      fs.mkdirSync(platformDir, { recursive: true });

      const profilesToml = `
default_config = "main"
current_config = "main"

[main]
description = "Optional model"
model = "to-be-cleared"
enabled = true
`;
      fs.writeFileSync(path.join(platformDir, "profiles.toml"), profilesToml, "utf-8");

      const { writeProfileField, readProfiles } = await import("../services/tomlReader.js");
      await writeProfileField("empty-model-test", "main", "model", "");

      const profiles = readProfiles("empty-model-test");
      const main = profiles.find((p) => p.name === "main");
      assert.equal(main!.model, undefined);
    });

    it("throws when profile does not exist", async () => {
      const platformDir = path.join(tmpDir, "platforms", "throw-test");
      fs.mkdirSync(platformDir, { recursive: true });

      const profilesToml = `
default_config = "main"
current_config = "main"

[main]
description = "Exists"
`;
      fs.writeFileSync(path.join(platformDir, "profiles.toml"), profilesToml, "utf-8");

      const { writeProfileField } = await import("../services/tomlReader.js");
      await assert.rejects(
        writeProfileField("throw-test", "nonexistent", "model", "value"),
        /not found/,
      );
    });
  });

  describe("toggleProfileEnabled", () => {
    it("flips enabled from true to false", async () => {
      const platformDir = path.join(tmpDir, "platforms", "toggle-test");
      fs.mkdirSync(platformDir, { recursive: true });

      const profilesToml = `
default_config = "main"
current_config = "main"

[main]
description = "Toggle me"
enabled = true
`;
      fs.writeFileSync(path.join(platformDir, "profiles.toml"), profilesToml, "utf-8");

      const { toggleProfileEnabled, readProfiles } = await import("../services/tomlReader.js");
      const newState = await toggleProfileEnabled("toggle-test", "main");

      assert.equal(newState, false);

      const profiles = readProfiles("toggle-test");
      assert.equal(profiles[0].enabled, false);
    });
  });

  describe("maskToken", () => {
    it("masks long tokens showing last 4 chars", async () => {
      const { maskToken } = await import("../services/tomlReader.js");
      assert.equal(maskToken("sk-ant-api03-abcdef1234"), "****1234");
    });

    it("returns **** for short tokens", async () => {
      const { maskToken } = await import("../services/tomlReader.js");
      assert.equal(maskToken("abc"), "****");
    });

    it("returns empty string for undefined", async () => {
      const { maskToken } = await import("../services/tomlReader.js");
      assert.equal(maskToken(undefined), "");
    });
  });
});
