import { afterEach, describe, it } from "node:test";
import assert from "node:assert/strict";
import {
  CCR_UNINITIALIZED_HINT,
  isProfileCreationPlatform,
  isProfileExchangePlatform,
} from "../models/types.js";
import {
  buildCatalogExportArgs,
  buildCatalogImportArgs,
  buildCodexAuthUpdateArgs,
  buildClaudeProfileOffArgs,
  buildClaudeProfileSwitchArgs,
  buildCodexProfileOffArgs,
  buildCodexProfileSwitchArgs,
  buildPlatformAuthOffArgs,
  buildPlatformProfileCreateArgs,
  buildPlatformProfileDeleteArgs,
  buildPlatformProfileDisableArgs,
  buildPlatformProfileEnableArgs,
  buildPlatformProfileOffArgs,
  buildPlatformProfileSetFieldArgs,
  buildPlatformProfileSwitchArgs,
} from "./ccrCliArgs.js";
import {
  detectCcrCapabilities,
  execCatalogImport,
  execCcr,
  execCcrJson,
  findCcrBinary,
  getCachedCcrCapabilitySnapshot,
  invalidateCcrCapabilityCache,
  setCcrCommandHooksForTest,
} from "./ccrCliCore.js";

describe("ccrCli arg builders", () => {
  afterEach(() => {
    setCcrCommandHooksForTest();
  });

  it("builds profile create args with structured flags", () => {
    const args = buildPlatformProfileCreateArgs("claude", "work", {
      description: "Work profile",
      base_url: "https://api.example.com/v1",
      auth_token: "secret",
      model: "claude-sonnet",
      small_fast_model: "claude-haiku",
      provider: "Anthropic",
      tags: [" work ", "team"],
      enabled: false,
    });

    assert.deepEqual(args, [
      "claude", "profile", "create", "work", "--json",
      "--description", "Work profile",
      "--base-url", "https://api.example.com/v1",
      "--auth-token", "secret",
      "--model", "claude-sonnet",
      "--small-fast-model", "claude-haiku",
      "--provider", "Anthropic",
      "--tag", "work",
      "--tag", "team",
      "--disabled",
    ]);
  });

  it("builds set-field args for scalar and array values", () => {
    assert.deepEqual(
      buildPlatformProfileSetFieldArgs("claude", "work", "model", "claude-opus"),
      ["claude", "profile", "set-field", "work", "model", "--json", "--value", "claude-opus"],
    );

    assert.deepEqual(
      buildPlatformProfileSetFieldArgs("codex", "prod", "tags", ["prod", "shared"]),
      ["codex", "profile", "set-field", "prod", "tags", "--json", "--value-json", "[\"prod\",\"shared\"]"],
    );

    assert.deepEqual(
      buildPlatformProfileSetFieldArgs("claude", "work", "model", undefined),
      ["claude", "profile", "set-field", "work", "model", "--json", "--clear"],
    );
  });

  it("builds enable/disable/delete args with force when requested", () => {
    assert.deepEqual(
      buildPlatformProfileEnableArgs("claude", "work"),
      ["claude", "profile", "enable", "work", "--json"],
    );
    assert.deepEqual(
      buildPlatformProfileDisableArgs("claude", "work", true),
      ["claude", "profile", "disable", "work", "--json", "--force"],
    );
    assert.deepEqual(
      buildPlatformProfileDeleteArgs("codex", "old", true),
      ["codex", "profile", "delete", "old", "--json", "--force"],
    );
  });

  it("builds named Claude and Codex profile wrappers", () => {
    assert.deepEqual(
      buildClaudeProfileSwitchArgs("work"),
      ["claude", "profile", "switch", "work"],
    );
    assert.deepEqual(
      buildCodexProfileSwitchArgs("team"),
      ["codex", "profile", "switch", "team"],
    );
    assert.deepEqual(
      buildClaudeProfileOffArgs(),
      ["claude", "profile", "off", "--json"],
    );
    assert.deepEqual(
      buildCodexProfileOffArgs(),
      ["codex", "profile", "off", "--json"],
    );
  });

  it("builds platform-scoped profile switch and off args", () => {
    assert.deepEqual(
      buildPlatformProfileSwitchArgs("codex", "work"),
      ["codex", "profile", "switch", "work"],
    );
    assert.deepEqual(
      buildPlatformProfileOffArgs("claude"),
      ["claude", "profile", "off", "--json"],
    );
  });

  it("builds platform-scoped auth off args", () => {
    assert.deepEqual(
      buildPlatformAuthOffArgs("claude"),
      ["claude", "auth", "off", "--json"],
    );
    assert.deepEqual(
      buildPlatformAuthOffArgs("codex"),
      ["codex", "auth", "off", "--json"],
    );
    assert.deepEqual(
      buildPlatformAuthOffArgs("grok"),
      ["grok", "auth", "off", "--json"],
    );
  });

  it("builds codex auth update args for set and clear", () => {
    assert.deepEqual(
      buildCodexAuthUpdateArgs("work", "Team account"),
      ["codex", "auth", "update", "work", "--json", "--description", "Team account"],
    );
    assert.deepEqual(
      buildCodexAuthUpdateArgs("work", undefined),
      ["codex", "auth", "update", "work", "--json", "--clear-description"],
    );
  });

  it("routes catalog export/import args between canonical 3-platform exchange and legacy fallback", () => {
    assert.equal(isProfileCreationPlatform("claude"), true);
    assert.equal(isProfileCreationPlatform("grok"), false);
    assert.equal(isProfileExchangePlatform("claude"), true);
    assert.equal(isProfileExchangePlatform("codex"), true);
    assert.equal(isProfileExchangePlatform("grok"), true);
    assert.equal(isProfileExchangePlatform("gemini"), false);
    assert.match(CCR_UNINITIALIZED_HINT, /ccr claude profile init/);

    for (const platform of ["claude", "codex", "grok"]) {
      assert.deepEqual(
        buildCatalogExportArgs(platform, true),
        [platform, "profile", "export", "--stdout"],
      );
      assert.deepEqual(
        buildCatalogImportArgs(platform, "/tmp/edit.toml", true),
        [platform, "profile", "import", "/tmp/edit.toml", "--replace", "--yes"],
      );
      assert.deepEqual(
        buildCatalogExportArgs(platform, false),
        ["export", "--platform", platform, "--stdout"],
      );
      assert.deepEqual(
        buildCatalogImportArgs(platform, "/tmp/edit.toml", false),
        ["import", "/tmp/edit.toml", "--platform", platform, "--force"],
      );
    }

    assert.deepEqual(
      buildCatalogExportArgs("gemini", true),
      ["export", "--platform", "gemini", "--stdout"],
    );
    assert.deepEqual(
      buildCatalogImportArgs("gemini", "/tmp/edit.toml", true),
      ["import", "/tmp/edit.toml", "--platform", "gemini", "--force"],
    );
  });

  it("probes supportsProfileExchange and caches capability snapshots", async () => {
    const calls: string[][] = [];
    setCcrCommandHooksForTest({
      findBinary: async () => "/usr/local/bin/ccr",
      runCommand: async (_cmd, args) => {
        calls.push(args);
        if (args.join(" ") === "codex auth current --help") {
          return { success: true, stdout: "Usage: ccr codex auth current [--json]", stderr: "", exitCode: 0 };
        }
        if (args.join(" ") === "codex quota --help") {
          return { success: true, stdout: "Usage: ccr codex quota [--json]", stderr: "", exitCode: 0 };
        }
        if (args.join(" ") === "claude profile export --help") {
          return { success: true, stdout: "Options: --stdout --no-secrets", stderr: "", exitCode: 0 };
        }
        if (args.join(" ") === "claude profile import --help") {
          return { success: true, stdout: "Options: --replace --merge --yes", stderr: "", exitCode: 0 };
        }
        return { success: false, stdout: "", stderr: "unexpected", exitCode: 2 };
      },
    });

    assert.equal(await findCcrBinary(), "/usr/local/bin/ccr");
    const snapshot = await detectCcrCapabilities();
    assert.equal(snapshot.binaryPath, "/usr/local/bin/ccr");
    assert.equal(snapshot.supportsCodexAuthCurrentJson, true);
    assert.equal(snapshot.supportsCodexQuotaJson, true);
    assert.equal(snapshot.supportsProfileExchange, true);
    assert.equal(getCachedCcrCapabilitySnapshot(), snapshot);

    const cached = await detectCcrCapabilities();
    assert.equal(cached, snapshot);
    assert.equal(calls.length, 4);

    invalidateCcrCapabilityCache();
    assert.equal(getCachedCcrCapabilitySnapshot(), null);
  });

  it("falls back cleanly on legacy binaries or missing binary and tolerates stderr deprecation warnings", async () => {
    setCcrCommandHooksForTest({
      findBinary: async () => null,
    });
    const missing = await detectCcrCapabilities();
    assert.equal(missing.supportsProfileExchange, false);
    const missingExec = await execCcr(["status"]);
    assert.equal(missingExec.success, false);
    assert.equal(missingExec.exitCode, null);

    const executedArgs: string[][] = [];
    setCcrCommandHooksForTest({
      findBinary: async () => "/usr/local/bin/ccr-legacy",
      runCommand: async (_cmd, args) => {
        executedArgs.push(args);
        if (args[args.length - 1] === "--help") {
          return {
            success: false,
            stdout: "",
            stderr: "error: unrecognized subcommand 'export'",
            exitCode: 2,
          };
        }
        if (args[0] === "import") {
          return {
            success: true,
            stdout: "Imported 2 profiles\n",
            stderr: "⚠ `ccr import` 已弃用，请改用 `ccr claude profile import`。\n",
            exitCode: 0,
          };
        }
        if (args[0] === "json-ok") {
          return { success: true, stdout: "{\"ok\":true}\n", stderr: "", exitCode: 0 };
        }
        if (args[0] === "json-bad") {
          return { success: true, stdout: "not-json", stderr: "", exitCode: 0 };
        }
        return { success: false, stdout: "", stderr: "boom", exitCode: 1 };
      },
    });

    const importResult = await execCatalogImport("claude", "/tmp/claude-edit.toml");
    assert.equal(importResult.success, true);
    assert.equal(importResult.stdout, "Imported 2 profiles");
    assert.match(importResult.stderr, /已弃用/);
    assert.deepEqual(
      executedArgs.find((entry) => entry[0] === "import"),
      ["import", "/tmp/claude-edit.toml", "--platform", "claude", "--force"],
    );

    const parsedJson = await execCcrJson<{ ok: boolean }>(["json-ok"]);
    assert.equal(parsedJson.success, true);
    assert.deepEqual(parsedJson.data, { ok: true });

    const invalidJson = await execCcrJson(["json-bad"]);
    assert.equal(invalidJson.success, false);
    assert.match(invalidJson.stderr, /Failed to parse JSON output/);
  });

  it("uses canonical profile import --replace --yes when capability is supported", async () => {
    const executedArgs: string[][] = [];
    setCcrCommandHooksForTest({
      findBinary: async () => "/usr/local/bin/ccr",
      runCommand: async (_cmd, args) => {
        executedArgs.push(args);
        if (args[args.length - 1] === "--help") {
          return {
            success: true,
            stdout: "--json --stdout --replace",
            stderr: "",
            exitCode: 0,
          };
        }
        return { success: true, stdout: "ok", stderr: "", exitCode: 0 };
      },
    });

    const claudeSave = await execCatalogImport("claude", "/tmp/claude.toml");
    assert.equal(claudeSave.success, true);
    assert.deepEqual(
      executedArgs[executedArgs.length - 1],
      ["claude", "profile", "import", "/tmp/claude.toml", "--replace", "--yes"],
    );

    const grokSave = await execCatalogImport("grok", "/tmp/grok.toml");
    assert.equal(grokSave.success, true);
    assert.deepEqual(
      executedArgs[executedArgs.length - 1],
      ["grok", "profile", "import", "/tmp/grok.toml", "--replace", "--yes"],
    );

    const geminiSave = await execCatalogImport("gemini", "/tmp/gemini.toml");
    assert.equal(geminiSave.success, true);
    assert.deepEqual(
      executedArgs[executedArgs.length - 1],
      ["import", "/tmp/gemini.toml", "--platform", "gemini", "--force"],
    );
  });

  it("invalidateCcrCapabilityCache resets binary path cache so newly installed ccr is detected", async () => {
    let installedBinary: string | null = null;
    setCcrCommandHooksForTest({
      findBinary: async () => installedBinary,
      runCommand: async (_cmd, args) => {
        if (args[args.length - 1] === "--help") {
          return {
            success: true,
            stdout: "--json --stdout --replace",
            stderr: "",
            exitCode: 0,
          };
        }
        return { success: true, stdout: "", stderr: "", exitCode: 0 };
      },
    });

    assert.equal(await findCcrBinary(), null);
    const beforeInstall = await detectCcrCapabilities();
    assert.equal(beforeInstall.binaryPath, undefined);
    assert.equal(beforeInstall.supportsProfileExchange, false);

    // Simulate installing `ccr` into PATH after VS Code startup
    installedBinary = "/opt/homebrew/bin/ccr";
    assert.equal(await findCcrBinary(), null);

    invalidateCcrCapabilityCache();
    assert.equal(await findCcrBinary(), "/opt/homebrew/bin/ccr");
    const afterInstall = await detectCcrCapabilities();
    assert.equal(afterInstall.binaryPath, "/opt/homebrew/bin/ccr");
    assert.equal(afterInstall.supportsProfileExchange, true);
  });
});
