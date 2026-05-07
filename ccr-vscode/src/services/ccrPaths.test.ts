/**
 * Unit tests for path resolution and TOML readers
 *
 * Uses Node built-in test runner (node:test)
 */

import { describe, it, before, after } from "node:test";
import assert from "node:assert/strict";
import * as fs from "fs";
import * as path from "path";
import * as os from "os";

describe("ccrPaths", () => {
  const originalEnv = process.env["CCR_ROOT"];

  after(() => {
    if (originalEnv !== undefined) {
      process.env["CCR_ROOT"] = originalEnv;
    } else {
      delete process.env["CCR_ROOT"];
    }
  });

  it("getCcrRoot returns $CCR_ROOT when set", async () => {
    process.env["CCR_ROOT"] = "/tmp/test-ccr-root";
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
});

describe("tomlReader", () => {
  let tmpDir: string;

  before(() => {
    tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "ccr-test-"));
    process.env["CCR_ROOT"] = tmpDir;
  });

  after(() => {
    fs.rmSync(tmpDir, { recursive: true, force: true });
    delete process.env["CCR_ROOT"];
  });

  describe("readRegistry", () => {
    it("returns null when config.toml does not exist", async () => {
      const { readRegistry } = await import("../services/tomlReader.js");
      assert.equal(await readRegistry(), null);
    });

    it("parses config.toml with platform entries", async () => {
      const configToml = `
[claude]
enabled = true
current_profile = "anthropic"
description = "Claude Code"
last_used = "2026-05-07T10:00:00Z"

[codex]
enabled = true
current_profile = "default"
last_used = "2026-05-06T10:00:00Z"
`;
      fs.writeFileSync(path.join(tmpDir, "config.toml"), configToml, "utf-8");

      const { readRegistry } = await import("../services/tomlReader.js");
      const result = await readRegistry();

      assert.notEqual(result, null);
      assert.equal(result!.platforms.length, 2);
      assert.equal(result!.platforms[0].name, "claude");
      assert.equal(result!.platforms[1].name, "codex");
      assert.equal("currentPlatform" in result!, false);
    });
  });

  describe("readProfiles", () => {
    it("returns empty array when profiles.toml does not exist", async () => {
      const { readProfiles } = await import("../services/tomlReader.js");
      const result = await readProfiles("nonexistent");
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
      const profiles = await readProfiles("claude");

      assert.equal(profiles.length, 2);
      assert.equal(profiles.find((p) => p.name === "anthropic")?.isCurrent, true);
      assert.equal(profiles.find((p) => p.name === "relay")?.enabled, false);
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
