/**
 * Status bar provider for CCR current profile indicator
 *
 * Displays Claude Code and Codex as two independent status bar items.
 */

import * as vscode from "vscode";
import type { PlatformInfo } from "../models/types";
import { ccrRootExists } from "../services/ccrPaths";
import { readRegistry, readProfiles } from "../services/tomlReader";
import {
  ensureCodexRuntimeSnapshot,
  getCachedCodexRuntimeSnapshot,
} from "../services/codexRuntimeReader";
import { buildStatusBarText, buildStatusBarTooltipLines } from "./statusBarPresentation";
import {
  type StatusBarPlatformName,
  resolveStatusBarItems,
} from "./statusBarTarget";

const STATUS_BAR_PRIORITIES: Record<StatusBarPlatformName, number> = {
  claude: 51,
  codex: 50,
};

export class StatusBarProvider implements vscode.Disposable {
  private readonly statusBarItems: Record<StatusBarPlatformName, vscode.StatusBarItem>;
  private readonly fallbackItem: vscode.StatusBarItem;
  private updateVersion = 0;

  constructor() {
    this.statusBarItems = {
      claude: this.createPlatformItem("claude"),
      codex: this.createPlatformItem("codex"),
    };
    this.fallbackItem = vscode.window.createStatusBarItem(
      vscode.StatusBarAlignment.Left,
      52,
    );
    this.fallbackItem.name = "CCR Profile";
    this.update();
  }

  /** Update status bar text from current config state */
  update(): void {
    void this.updateAsync();
  }

  private createPlatformItem(platformName: StatusBarPlatformName): vscode.StatusBarItem {
    const item = vscode.window.createStatusBarItem(
      vscode.StatusBarAlignment.Left,
      STATUS_BAR_PRIORITIES[platformName],
    );
    item.name = `CCR ${platformName}`;
    return item;
  }

  private hideAllItems(): void {
    this.fallbackItem.hide();
    this.statusBarItems.claude.hide();
    this.statusBarItems.codex.hide();
  }

  private showFallback(text: string, tooltip: string): void {
    this.statusBarItems.claude.hide();
    this.statusBarItems.codex.hide();
    this.fallbackItem.command = "ccr.switchProfile";
    this.fallbackItem.text = text;
    this.fallbackItem.tooltip = tooltip;
    this.fallbackItem.show();
  }

  private async updatePlatformItem(
    updateVersion: number,
    platform: PlatformInfo,
  ): Promise<void> {
    const item = this.statusBarItems[platform.name as StatusBarPlatformName];
    if (!item) {
      return;
    }

    const profiles = await readProfiles(platform.name);
    if (updateVersion !== this.updateVersion) {
      return;
    }

    const current = profiles.find((profile) => profile.isCurrent);
    const profileName = current?.name ?? platform.currentProfile ?? "none";
    const runtimeSnapshot = platform.name === "codex"
      ? getCachedCodexRuntimeSnapshot()
      : null;

    if (platform.name === "codex") {
      ensureCodexRuntimeSnapshot(() => this.update());
    }

    item.command = {
      command: "ccr.switchProfileForPlatform",
      title: "Switch Profile",
      arguments: [platform.name],
    };
    item.text = buildStatusBarText(platform, profileName, runtimeSnapshot);
    item.tooltip = new vscode.MarkdownString(
      buildStatusBarTooltipLines("pinned", platform, profileName, current, runtimeSnapshot)
        .join("  \n"),
    );
    item.show();
  }

  private async updateAsync(): Promise<void> {
    const updateVersion = ++this.updateVersion;
    const config = vscode.workspace.getConfiguration("ccr");
    const mode = config.get<string>("statusBar.mode", "pinned");
    const showClaude = config.get<boolean>("statusBar.showClaude", true);
    const showCodex = config.get<boolean>("statusBar.showCodex", true);

    if (mode === "hidden") {
      this.hideAllItems();
      return;
    }

    if (!ccrRootExists()) {
      this.showFallback("$(gear) CCR: Not configured", "CCR is not initialized. Run 'ccr init' to get started.");
      return;
    }

    const registry = await readRegistry();
    if (updateVersion !== this.updateVersion) {
      return;
    }

    if (!registry || registry.platforms.length === 0) {
      this.showFallback("$(gear) CCR: No platforms", "No platforms configured in CCR.");
      return;
    }

    this.fallbackItem.hide();
    const targets = resolveStatusBarItems({
      platforms: registry.platforms,
      currentPlatform: registry.currentPlatform,
      mode,
      showClaude,
      showCodex,
    });

    const visiblePlatforms = new Set(targets.map((target) => target.platform?.name).filter(Boolean));
    for (const [platformName, item] of Object.entries(this.statusBarItems) as [StatusBarPlatformName, vscode.StatusBarItem][]) {
      if (!visiblePlatforms.has(platformName)) {
        item.hide();
      }
    }

    if (targets.length === 0) {
      this.hideAllItems();
      return;
    }

    for (const target of targets) {
      if (!target.platform) {
        continue;
      }
      await this.updatePlatformItem(updateVersion, target.platform);
      if (updateVersion !== this.updateVersion) {
        return;
      }
    }
  }

  dispose(): void {
    this.fallbackItem.dispose();
    this.statusBarItems.claude.dispose();
    this.statusBarItems.codex.dispose();
  }
}
