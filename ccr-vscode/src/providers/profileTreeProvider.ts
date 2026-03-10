/**
 * TreeDataProvider for CCR profiles sidebar
 *
 * Two-level hierarchy:
 * - Platform nodes (top-level, collapsible) with codicon icons
 *   - Profile nodes (children, leaf) with semantic status icons
 */

import * as vscode from "vscode";
import { ccrRootExists, getPlatformCodiconId } from "../services/ccrPaths";
import { readRegistry, readProfiles, maskToken } from "../services/tomlReader";
import type { PlatformInfo, ProfileInfo } from "../models/types";

/** Escape user-controlled strings to prevent Markdown injection */
function escapeMarkdown(str: string): string {
  return str.replace(/([\\`*_{}[\]()#+\-.!|~])/g, "\\$1");
}

/** Platform-specific ThemeColor IDs for codicon icons */
const PLATFORM_THEME_COLORS: Record<string, string> = {
  claude: "charts.orange",
  codex: "charts.green",
  gemini: "charts.blue",
  qwen: "foreground",
  iflow: "charts.yellow",
  droid: "charts.purple",
};

// ── Tree item types ──

export class PlatformNode extends vscode.TreeItem {
  constructor(public readonly platform: PlatformInfo) {
    super(platform.displayName, vscode.TreeItemCollapsibleState.Expanded);
    this.contextValue = "platform";

    // Codicon icon with platform-specific color
    const codiconId = getPlatformCodiconId(platform.name);
    const themeColor = PLATFORM_THEME_COLORS[platform.name] ?? "foreground";
    this.iconPath = new vscode.ThemeIcon(codiconId, new vscode.ThemeColor(themeColor));

    // Description: arrow indicator + current profile name
    if (!platform.enabled) {
      this.description = "(disabled)";
    } else if (platform.currentProfile) {
      this.description = `\u25B8 ${platform.currentProfile}`;
    }

    // Rich markdown tooltip
    const md = new vscode.MarkdownString();
    md.appendMarkdown(`### $(${codiconId}) ${escapeMarkdown(platform.displayName)}\n\n`);
    md.appendMarkdown(`**Status:** ${platform.enabled ? "Enabled" : "Disabled"}\n\n`);
    if (platform.currentProfile) {
      md.appendMarkdown(`**Current Profile:** \`${escapeMarkdown(platform.currentProfile)}\`\n\n`);
    }
    md.appendMarkdown(`---\n\n`);
    md.appendMarkdown(`*Click to expand \u00B7 Right-click to open config file*`);
    this.tooltip = md;
  }
}

export class ProfileNode extends vscode.TreeItem {
  constructor(public readonly profile: ProfileInfo) {
    super(profile.name, vscode.TreeItemCollapsibleState.None);

    // Context value for menu contributions
    this.contextValue = profile.isCurrent ? "profile-current" : "profile";

    // Description: provider · model (middle dot separator)
    const parts: string[] = [];
    if (profile.provider) parts.push(profile.provider);
    if (profile.model) parts.push(profile.model);
    this.description = parts.join(" \u00B7 ") || undefined;

    // Append disabled indicator with em dash
    if (!profile.enabled) {
      this.description = `${this.description ?? ""} \u2014 disabled`.trim();
    }

    // Icons with clear semantic meaning
    if (profile.isCurrent) {
      this.iconPath = new vscode.ThemeIcon("pass-filled", new vscode.ThemeColor("testing.iconPassed"));
    } else if (!profile.enabled) {
      this.iconPath = new vscode.ThemeIcon("eye-closed", new vscode.ThemeColor("disabledForeground"));
    } else {
      this.iconPath = new vscode.ThemeIcon("circle-large-outline", new vscode.ThemeColor("foreground"));
    }

    // Rich markdown tooltip with structured sections
    const md = new vscode.MarkdownString();

    // Header with status icon
    const statusIcon = profile.isCurrent
      ? "$(pass-filled)"
      : profile.enabled
        ? "$(circle-large-outline)"
        : "$(eye-closed)";
    md.appendMarkdown(`### ${statusIcon} ${escapeMarkdown(profile.name)}\n\n`);

    if (profile.description) {
      md.appendMarkdown(`*${escapeMarkdown(profile.description)}*\n\n`);
    }

    // Connection section
    if (profile.baseUrl || profile.authToken) {
      md.appendMarkdown(`**Connection**\n\n`);
      if (profile.baseUrl) md.appendMarkdown(`- Base URL: \`${escapeMarkdown(profile.baseUrl)}\`\n`);
      if (profile.authToken) md.appendMarkdown(`- Auth Token: \`${escapeMarkdown(maskToken(profile.authToken))}\`\n`);
      md.appendMarkdown(`\n`);
    }

    // Model section
    if (profile.model || profile.smallFastModel) {
      md.appendMarkdown(`**Model**\n\n`);
      if (profile.model) md.appendMarkdown(`- Model: \`${escapeMarkdown(profile.model)}\`\n`);
      if (profile.smallFastModel) md.appendMarkdown(`- Small/Fast: \`${escapeMarkdown(profile.smallFastModel)}\`\n`);
      md.appendMarkdown(`\n`);
    }

    // Identity section
    if (profile.provider || profile.providerType || profile.account) {
      md.appendMarkdown(`**Identity**\n\n`);
      if (profile.provider) md.appendMarkdown(`- Provider: ${escapeMarkdown(profile.provider)}\n`);
      if (profile.providerType) md.appendMarkdown(`- Type: ${escapeMarkdown(profile.providerType)}\n`);
      if (profile.account) md.appendMarkdown(`- Account: ${escapeMarkdown(profile.account)}\n`);
      md.appendMarkdown(`\n`);
    }

    // Footer: status, usage, tags
    md.appendMarkdown(`---\n\n`);
    const status = profile.isCurrent ? "Active" : profile.enabled ? "Available" : "Disabled";
    md.appendMarkdown(`**Status:** ${status}`);
    if (profile.usageCount > 0) md.appendMarkdown(` \u00B7 **Usage:** ${profile.usageCount}`);
    if (profile.tags && profile.tags.length > 0) {
      md.appendMarkdown(` \u00B7 **Tags:** ${profile.tags.map((t) => escapeMarkdown(t)).join(", ")}`);
    }
    md.appendMarkdown(`\n\n`);
    md.appendMarkdown(`*Click to switch \u00B7 Right-click for more*`);

    this.tooltip = md;

    // Click to switch
    this.command = {
      command: "ccr.switchProfile",
      title: "Switch Profile",
      arguments: [this],
    };
  }
}

export class MessageNode extends vscode.TreeItem {
  constructor(message: string) {
    super(message, vscode.TreeItemCollapsibleState.None);
    this.contextValue = "message";
    this.iconPath = new vscode.ThemeIcon("info", new vscode.ThemeColor("charts.blue"));
  }
}

type TreeNode = PlatformNode | ProfileNode | MessageNode;

// ── Provider ──

export class ProfileTreeProvider implements vscode.TreeDataProvider<TreeNode> {
  private _onDidChangeTreeData = new vscode.EventEmitter<TreeNode | undefined | null>();
  readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

  refresh(): void {
    this._onDidChangeTreeData.fire(undefined);
  }

  getTreeItem(element: TreeNode): vscode.TreeItem {
    return element;
  }

  getChildren(element?: TreeNode): TreeNode[] {
    // Check if CCR is initialized
    if (!ccrRootExists()) {
      if (!element) {
        return [new MessageNode("CCR is not initialized. Run 'ccr init' to get started.")];
      }
      return [];
    }

    // Top level: platform nodes
    if (!element) {
      const registry = readRegistry();
      if (!registry || registry.platforms.length === 0) {
        return [new MessageNode("No platforms configured.")];
      }
      return registry.platforms
        .filter((p) => p.enabled)
        .map((p) => new PlatformNode(p));
    }

    // Second level: profile nodes under a platform
    if (element instanceof PlatformNode) {
      const profiles = readProfiles(element.platform.name);
      if (profiles.length === 0) {
        return [new MessageNode("No profiles configured.")];
      }
      // Sort: current first, then enabled, then alphabetical
      profiles.sort((a, b) => {
        if (a.isCurrent !== b.isCurrent) return a.isCurrent ? -1 : 1;
        if (a.enabled !== b.enabled) return a.enabled ? -1 : 1;
        return a.name.localeCompare(b.name);
      });
      return profiles.map((p) => new ProfileNode(p));
    }

    return [];
  }
}
