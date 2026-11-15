import { afterEach, beforeEach, describe, it } from "node:test";
import assert from "node:assert/strict";
import * as fs from "fs";
import * as os from "os";
import * as path from "path";
import {
  catalogEditPath,
  exportCanonicalToml,
  parseExportedCatalogDocument,
  readProfileDocument,
} from "./catalogReader.js";
import { setCcrCommandHooksForTest } from "./ccrCliCore.js";
import { readProfiles } from "./tomlReader.js";

describe("catalogReader", () => {
  let tmpDir: string;
  const originalEnv = process.env["CCR_ROOT"];

  beforeEach(() => {
    tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "ccr-catalog-reader-"));
    process.env["CCR_ROOT"] = tmpDir;
  });

  afterEach(() => {
    setCcrCommandHooksForTest();
    fs.rmSync(tmpDir, { recursive: true, force: true });
    if (originalEnv !== undefined) {
      process.env["CCR_ROOT"] = originalEnv;
    } else {
      delete process.env["CCR_ROOT"];
    }
  });

  it("unwraps ExchangeV1 [profiles.<name>] documents and preserves flat TOML fallback", () => {
    const exchangeToml = `# CCR Profile Exchange File
schema_version = 1
platform = "claude"
exported_at = 2026-11-14T12:00:00Z
secrets_mode = "included"

[profiles.anthropic]
description = "Anthropic Official"
base_url = "https://api.anthropic.com"
auth_token = "sk-ant-secret-1234"
enabled = true

[profiles.relay]
description = "Relay Backup"
enabled = false
`;
    const unwrapped = parseExportedCatalogDocument(exchangeToml, "anthropic");
    assert.equal(unwrapped.current_config, "anthropic");
    assert.equal("schema_version" in unwrapped, false);
    assert.equal("profiles" in unwrapped, false);
    assert.deepEqual(unwrapped.anthropic, {
      description: "Anthropic Official",
      base_url: "https://api.anthropic.com",
      auth_token: "sk-ant-secret-1234",
      enabled: true,
    });
    assert.deepEqual(unwrapped.relay, {
      description: "Relay Backup",
      enabled: false,
    });

    const flatToml = `default_config = "default"
current_config = "work"

[work]
description = "Legacy Flat"
model = "claude-sonnet-4-5"
`;
    const flatParsed = parseExportedCatalogDocument(flatToml, "ignored");
    assert.equal(flatParsed.current_config, "work");
    assert.deepEqual(flatParsed.work, {
      description: "Legacy Flat",
      model: "claude-sonnet-4-5",
    });
  });

  it("exports via canonical <platform> profile export --stdout when supported and falls back on empty catalog", async () => {
    const calls: string[][] = [];
    let emptyCatalog = false;

    setCcrCommandHooksForTest({
      findBinary: async () => "/usr/local/bin/ccr",
      runCommand: async (_cmd, args) => {
        calls.push(args);
        if (args[args.length - 1] === "--help") {
          return {
            success: true,
            stdout: "--json --stdout --replace",
            stderr: "",
            exitCode: 0,
          };
        }
        if (args.join(" ") === "claude profile export --stdout") {
          if (emptyCatalog) {
            return {
              success: false,
              stdout: "",
              stderr: "Error: empty profile catalog",
              exitCode: 90,
            };
          }
          return {
            success: true,
            stdout: "schema_version = 1\nplatform = \"claude\"\n\n[profiles.work]\ndescription = \"Work\"\n",
            stderr: "",
            exitCode: 0,
          };
        }
        if (args.join(" ") === "export --platform claude --stdout") {
          return {
            success: true,
            stdout: "default_config = \"default\"\ncurrent_config = \"\"\n",
            stderr: "⚠ `ccr export` 已弃用\n",
            exitCode: 0,
          };
        }
        return { success: false, stdout: "", stderr: "error", exitCode: 1 };
      },
    });

    const exported = await exportCanonicalToml("claude");
    assert.notEqual(exported, null);
    assert.match(exported!, /\[profiles\.work\]/);

    emptyCatalog = true;
    const emptyExported = await exportCanonicalToml("claude");
    assert.notEqual(emptyExported, null);
    assert.match(emptyExported!, /default_config = "default"/);
    assert.equal(
      calls.some((entry) => entry.join(" ") === "export --platform claude --stdout"),
      true,
    );
  });

  it("falls back to legacy export --stdout and ignores stderr deprecation warnings when reading broken/unopenable sqlite catalog", async () => {
    const platformDir = path.join(tmpDir, "platforms", "claude");
    fs.mkdirSync(platformDir, { recursive: true });
    // Write non-sqlite bytes so readSqliteDocument fails and triggers CLI export fallback
    fs.writeFileSync(path.join(platformDir, "profiles.db"), "not-a-sqlite-db", "utf-8");
    fs.writeFileSync(
      path.join(tmpDir, "config.toml"),
      "[claude]\nenabled = true\ncurrent_profile = \"work\"\n",
      "utf-8",
    );

    setCcrCommandHooksForTest({
      findBinary: async () => "/usr/local/bin/ccr-legacy",
      runCommand: async (_cmd, args) => {
        if (args[args.length - 1] === "--help") {
          return {
            success: false,
            stdout: "",
            stderr: "unrecognized subcommand",
            exitCode: 2,
          };
        }
        if (args.join(" ") === "export --platform claude --stdout") {
          return {
            success: true,
            stdout: "default_config = \"work\"\ncurrent_config = \"work\"\n\n[work]\ndescription = \"From Legacy Export\"\nenabled = true\n",
            stderr: "⚠ `ccr export` 已弃用，请改用 `ccr claude profile export`。\n",
            exitCode: 0,
          };
        }
        return { success: false, stdout: "", stderr: "unexpected", exitCode: 1 };
      },
    });

    const profiles = await readProfiles("claude");
    assert.equal(profiles.length, 1);
    assert.equal(profiles[0]?.name, "work");
    assert.equal(profiles[0]?.isCurrent, true);
    assert.equal(profiles[0]?.description, "From Legacy Export");

    // Now test canonical ExchangeV1 export fallback through readProfileDocument
    setCcrCommandHooksForTest({
      findBinary: async () => "/usr/local/bin/ccr",
      runCommand: async (_cmd, args) => {
        if (args[args.length - 1] === "--help") {
          return {
            success: true,
            stdout: "--json --stdout --replace",
            stderr: "",
            exitCode: 0,
          };
        }
        if (args.join(" ") === "claude profile export --stdout") {
          return {
            success: true,
            stdout: "schema_version = 1\nplatform = \"claude\"\n\n[profiles.work]\ndescription = \"From ExchangeV1\"\nenabled = true\n",
            stderr: "",
            exitCode: 0,
          };
        }
        return { success: false, stdout: "", stderr: "unexpected", exitCode: 1 };
      },
    });

    const doc = await readProfileDocument("claude");
    assert.notEqual(doc, null);
    assert.equal(doc?.current_config, "work");
    assert.deepEqual(doc?.work, {
      description: "From ExchangeV1",
      enabled: true,
    });

    assert.match(catalogEditPath("claude"), /ccr-claude-catalog-/);
  });
});
