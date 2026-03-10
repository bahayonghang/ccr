/**
 * Status bar provider for CCR current profile indicator
 *
 * Displays current platform and profile in the bottom status bar.
 * Click opens QuickPick for profile switching.
 */

import * as vscode from "vscode";
import { ccrRootExists } from "../services/ccrPaths";
import { readRegistry, readProfiles } from "../services/tomlReader";

export class StatusBarProvider implements vscode.Disposable {
  private statusBarItem: vscode.StatusBarItem;

  constructor() {
    this.statusBarItem = vscode.window.createStatusBarItem(
      vscode.StatusBarAlignment.Left,
      50,
    );
    this.statusBarItem.command = "ccr.switchProfile";
    this.statusBarItem.name = "CCR Profile";
    this.update();
    this.statusBarItem.show();
  }

  /** Update status bar text from current config state */
  update(): void {
    if (!ccrRootExists()) {
      this.statusBarItem.text = "$(gear) CCR: Not configured";
      this.statusBarItem.tooltip = "CCR is not initialized. Run 'ccr init' to get started.";
      return;
    }

    const registry = readRegistry();
    if (!registry || registry.platforms.length === 0) {
      this.statusBarItem.text = "$(gear) CCR: No platforms";
      this.statusBarItem.tooltip = "No platforms configured in CCR.";
      return;
    }

    // Find the first platform (sorted with current first by readRegistry)
    const primary = registry.platforms[0];
    if (!primary) {
      this.statusBarItem.text = "$(gear) CCR";
      return;
    }

    // Read current profile name
    const profiles = readProfiles(primary.name);
    const current = profiles.find((p) => p.isCurrent);
    const profileName = current?.name ?? primary.currentProfile ?? "none";

    this.statusBarItem.text = `${primary.icon} ${primary.displayName}: ${profileName}`;
    this.statusBarItem.tooltip = new vscode.MarkdownString(
      [
        `**CCR Profile Status**`,
        `Platform: ${primary.displayName}`,
        `Profile: ${profileName}`,
        current?.model ? `Model: ${current.model}` : null,
        current?.provider ? `Provider: ${current.provider}` : null,
        ``,
        `_Click to switch profile_`,
      ]
        .filter(Boolean)
        .join("  \n"),
    );
  }

  dispose(): void {
    this.statusBarItem.dispose();
  }
}
