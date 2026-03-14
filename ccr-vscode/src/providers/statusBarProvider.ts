/**
 * Status bar provider for CCR current profile indicator
 *
 * Displays either a pinned platform, the current platform, or hides itself
 * based on user settings.
 */

import * as vscode from "vscode";
import { ccrRootExists } from "../services/ccrPaths";
import { readRegistry, readProfiles } from "../services/tomlReader";
import { resolveStatusBarTarget } from "./statusBarTarget";

export class StatusBarProvider implements vscode.Disposable {
  private readonly statusBarItem: vscode.StatusBarItem;

  constructor() {
    this.statusBarItem = vscode.window.createStatusBarItem(
      vscode.StatusBarAlignment.Left,
      50,
    );
    this.statusBarItem.name = "CCR Profile";
    this.update();
  }

  /** Update status bar text from current config state */
  update(): void {
    const config = vscode.workspace.getConfiguration("ccr");
    const mode = config.get<string>("statusBar.mode", "pinned");
    const pinnedPlatform = config.get<string>("statusBar.platform", "");

    if (mode === "hidden") {
      this.statusBarItem.hide();
      return;
    }

    if (!ccrRootExists()) {
      this.statusBarItem.command = "ccr.switchProfile";
      this.statusBarItem.text = "$(gear) CCR: Not configured";
      this.statusBarItem.tooltip = "CCR is not initialized. Run 'ccr init' to get started.";
      this.statusBarItem.show();
      return;
    }

    const registry = readRegistry();
    if (!registry || registry.platforms.length === 0) {
      this.statusBarItem.command = "ccr.switchProfile";
      this.statusBarItem.text = "$(gear) CCR: No platforms";
      this.statusBarItem.tooltip = "No platforms configured in CCR.";
      this.statusBarItem.show();
      return;
    }

    const target = resolveStatusBarTarget({
      platforms: registry.platforms,
      currentPlatform: registry.currentPlatform,
      mode,
      pinnedPlatform,
    });

    if (!target.visible) {
      this.statusBarItem.hide();
      return;
    }

    const platform = target.platform;
    if (!platform) {
      this.statusBarItem.command = "ccr.switchProfile";
      this.statusBarItem.text = "$(gear) CCR";
      this.statusBarItem.tooltip = "CCR status bar target could not be resolved.";
      this.statusBarItem.show();
      return;
    }

    const profiles = readProfiles(platform.name);
    const current = profiles.find((profile) => profile.isCurrent);
    const profileName = current?.name ?? platform.currentProfile ?? "none";

    this.statusBarItem.command = target.mode === "pinned"
      ? {
          command: "ccr.switchProfileForPlatform",
          title: "Switch Profile",
          arguments: [platform.name],
        }
      : "ccr.switchProfile";

    this.statusBarItem.text = `${platform.icon} ${platform.displayName}: ${profileName}`;

    const tooltipLines = [
      `**CCR Profile Status**`,
      `Mode: ${target.mode === "pinned" ? "Pinned platform" : "Current platform"}`,
      `Platform: ${platform.displayName}`,
      `Profile: ${profileName}`,
      current?.model ? `Model: ${current.model}` : null,
      current?.provider ? `Provider: ${current.provider}` : null,
      target.warning ? `$(warning) ${target.warning}` : null,
      ``,
      target.mode === "pinned"
        ? `_Click to switch profiles for ${platform.displayName}_`
        : `_Click to switch profile_`,
    ]
      .filter(Boolean)
      .join("  \n");

    this.statusBarItem.tooltip = new vscode.MarkdownString(tooltipLines);
    this.statusBarItem.show();
  }

  dispose(): void {
    this.statusBarItem.dispose();
  }
}
