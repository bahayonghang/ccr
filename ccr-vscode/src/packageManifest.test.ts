import { describe, it } from "node:test";
import assert from "node:assert/strict";
import * as fs from "fs";
import * as path from "path";

describe("package manifest", () => {
  it("exposes the platform metadata and lazy activation surface", () => {
    const manifestPath = path.join(process.cwd(), "package.json");
    const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf-8")) as {
      activationEvents?: string[];
      description?: string;
      keywords?: string[];
      contributes?: {
        commands?: { command: string }[];
      };
    };

    assert.equal(manifest.description, "Manage CCR profiles and platform metadata from the VS Code sidebar");
    assert.equal(manifest.activationEvents, undefined);
    assert.ok(manifest.contributes?.commands?.some((command) => command.command === "ccr.switchProfileForPlatform"));
    assert.ok(manifest.keywords?.includes("gemini"));
    assert.ok(manifest.keywords?.includes("qwen"));
    assert.ok(manifest.keywords?.includes("droid"));
  });
});
