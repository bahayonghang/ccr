import { describe, it } from "node:test";
import assert from "node:assert/strict";
import type { ProfileEditorDraft, ProfileInfo } from "../models/types";
import { getPanelKey, getPanelTitle, normalizeFieldValue } from "./profileEditorPanel.helpers";

describe("profileEditorPanel helpers", () => {
  it("builds a stable key for edit mode", () => {
    const profile = {
      name: "primary",
      platformName: "claude",
      usageCount: 0,
      enabled: true,
      isCurrent: false,
    } satisfies ProfileInfo;

    assert.equal(getPanelKey("edit", profile), "claude/primary");
    assert.equal(getPanelTitle("edit", profile), "Edit: primary (claude)");
  });

  it("builds a stable key for create mode", () => {
    const draft = {
      name: "",
      platformName: "codex",
      enabled: true,
    } satisfies ProfileEditorDraft;

    assert.equal(getPanelKey("create", draft), "create/codex");
    assert.equal(getPanelTitle("create", draft), "Add Profile: codex");
  });

  it("normalizes comma-separated tags into arrays", () => {
    assert.deepEqual(normalizeFieldValue("tags", " fast, backup , relay "), ["fast", "backup", "relay"]);
  });

  it("returns undefined for empty optional values", () => {
    assert.equal(normalizeFieldValue("model", ""), undefined);
    assert.equal(normalizeFieldValue("tags", ""), undefined);
  });

  it("preserves non-tag values", () => {
    assert.equal(normalizeFieldValue("model", "claude-sonnet"), "claude-sonnet");
  });
});
