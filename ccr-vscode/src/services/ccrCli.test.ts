import { describe, it } from "node:test";
import assert from "node:assert/strict";
import {
  buildCodexAuthUpdateArgs,
  buildClaudeProfileOffArgs,
  buildClaudeProfileSwitchArgs,
  buildCodexProfileOffArgs,
  buildCodexProfileSwitchArgs,
  buildPlatformProfileCreateArgs,
  buildPlatformProfileDeleteArgs,
  buildPlatformProfileDisableArgs,
  buildPlatformProfileEnableArgs,
  buildPlatformProfileOffArgs,
  buildPlatformProfileSetFieldArgs,
  buildPlatformProfileSwitchArgs,
} from "./ccrCliArgs.js";

describe("ccrCli arg builders", () => {
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
});
