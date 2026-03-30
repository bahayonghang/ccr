import { describe, it } from "node:test";
import assert from "node:assert/strict";
import {
  buildCodexAuthUpdateArgs,
  buildPlatformProfileCreateArgs,
  buildPlatformProfileDeleteArgs,
  buildPlatformProfileDisableArgs,
  buildPlatformProfileEnableArgs,
  buildPlatformProfileSetFieldArgs,
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
      "platform", "profile", "create", "claude", "work", "--json",
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
      ["platform", "profile", "set-field", "claude", "work", "model", "--json", "--value", "claude-opus"],
    );

    assert.deepEqual(
      buildPlatformProfileSetFieldArgs("codex", "prod", "tags", ["prod", "shared"]),
      ["platform", "profile", "set-field", "codex", "prod", "tags", "--json", "--value-json", "[\"prod\",\"shared\"]"],
    );

    assert.deepEqual(
      buildPlatformProfileSetFieldArgs("claude", "work", "model", undefined),
      ["platform", "profile", "set-field", "claude", "work", "model", "--json", "--clear"],
    );
  });

  it("builds enable/disable/delete args with force when requested", () => {
    assert.deepEqual(
      buildPlatformProfileEnableArgs("claude", "work"),
      ["platform", "profile", "enable", "claude", "work", "--json"],
    );
    assert.deepEqual(
      buildPlatformProfileDisableArgs("claude", "work", true),
      ["platform", "profile", "disable", "claude", "work", "--json", "--force"],
    );
    assert.deepEqual(
      buildPlatformProfileDeleteArgs("codex", "old", true),
      ["platform", "profile", "delete", "codex", "old", "--json", "--force"],
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
