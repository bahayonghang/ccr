/**
 * TreeDataProvider for CCR profiles sidebar
 *
 * Three-level hierarchy:
 * - Platform nodes (top-level)
 *   - Section nodes (Profiles/Auth)
 *     - Profile/Auth leaf nodes
 */

import * as vscode from "vscode";
import { ccrRootExists, getPlatformCodiconId } from "../services/ccrPaths";
import { readRegistry, readProfiles, maskToken } from "../services/tomlReader";
import { readCodexAuthAccounts } from "../services/codexAuthReader";
import type { CodexAuthInfo, PlatformInfo, ProfileInfo, TreeSectionInfo, TreeSectionKind } from "../models/types";

/** Escape user-controlled strings to prevent Markdown injection */
function escapeMarkdown(str: string): string {
  return str.replace(/([\\`*_{}[\]()#+\-.!|~])/g, "\\$1");
}

/** Platform-specific ThemeColor IDs for codicon icons */
const PLATFORM_THEME_COLORS: Record<string, string> = {
  claude: "charts.orange",
  codex: "charts.green",
};

function getSectionInfo(platformName: string, kind: TreeSectionKind): TreeSectionInfo {
  if (platformName === "claude") {
    return {
      kind,
      platformName,
      label: "Claude Profiles",
      description: "Switch and manage Claude profiles",
    };
  }

  if (kind === "auth") {
    return {
      kind,
      platformName,
      label: "Codex Auth",
      description: "Switch and manage saved Codex auth accounts",
    };
  }

  return {
    kind,
    platformName,
    label: "Codex Profiles",
    description: "Switch and manage Codex profiles",
  };
}

function sortProfiles(profiles: ProfileInfo[]): ProfileInfo[] {
  return [...profiles].sort((a, b) => {
    if (a.isCurrent !== b.isCurrent) return a.isCurrent ? -1 : 1;
    if (a.enabled !== b.enabled) return a.enabled ? -1 : 1;
    return a.name.localeCompare(b.name);
  });
}

function sortAuthAccounts(accounts: CodexAuthInfo[]): CodexAuthInfo[] {
  return [...accounts].sort((a, b) => {
    if (a.isCurrent !== b.isCurrent) return a.isCurrent ? -1 : 1;
    if (a.isVirtual !== b.isVirtual) return a.isVirtual ? -1 : 1;
    return a.name.localeCompare(b.name);
  });
}

export class PlatformNode extends vscode.TreeItem {
  constructor(public readonly platform: PlatformInfo) {
    super(
      platform.displayName,
      platform.enabled
        ? vscode.TreeItemCollapsibleState.Expanded
        : vscode.TreeItemCollapsibleState.None,
    );
    this.contextValue = platform.enabled && (platform.name === "claude" || platform.name === "codex")
      ? "platform-create-supported"
      : platform.enabled
        ? "platform"
        : "platform-disabled";

    const codiconId = getPlatformCodiconId(platform.name);
    if (!platform.enabled) {
      this.iconPath = new vscode.ThemeIcon("eye-closed", new vscode.ThemeColor("disabledForeground"));
    } else {
      const themeColor = PLATFORM_THEME_COLORS[platform.name] ?? "foreground";
      this.iconPath = new vscode.ThemeIcon(codiconId, new vscode.ThemeColor(themeColor));
    }

    if (!platform.enabled) {
      this.description = "(disabled)";
    } else if (platform.currentProfile) {
      this.description = `▸ ${platform.currentProfile}`;
    }

    const md = new vscode.MarkdownString();
    md.appendMarkdown(`### $(${codiconId}) ${escapeMarkdown(platform.displayName)}\n\n`);
    md.appendMarkdown(`**Status:** ${platform.enabled ? "Enabled" : "Disabled"}\n\n`);
    if (platform.currentProfile) {
      md.appendMarkdown(`**Current Profile:** \`${escapeMarkdown(platform.currentProfile)}\`\n\n`);
    }
    md.appendMarkdown(`---\n\n`);
    md.appendMarkdown(`*Expand to manage grouped resources*`);
    this.tooltip = md;
  }
}

export class SectionNode extends vscode.TreeItem {
  constructor(public readonly section: TreeSectionInfo) {
    super(section.label, vscode.TreeItemCollapsibleState.Expanded);
    this.contextValue = section.kind === "profiles" && (section.platformName === "claude" || section.platformName === "codex")
      ? "section-profiles-create-supported"
      : `section-${section.kind}`;
    this.description = section.kind === "auth" ? "saved accounts" : "profiles";
    this.iconPath = new vscode.ThemeIcon(
      section.kind === "auth" ? "key" : "files",
      new vscode.ThemeColor(section.kind === "auth" ? "charts.green" : "foreground"),
    );
    this.tooltip = new vscode.MarkdownString(`**${escapeMarkdown(section.label)}**\n\n${escapeMarkdown(section.description)}`);
  }
}

export class ProfileNode extends vscode.TreeItem {
  constructor(public readonly profile: ProfileInfo) {
    super(profile.name, vscode.TreeItemCollapsibleState.None);

    this.contextValue = profile.isCurrent ? "profile-current" : "profile";

    const parts: string[] = [];
    if (profile.provider) parts.push(profile.provider);
    if (profile.model) parts.push(profile.model);
    this.description = parts.join(" · ") || undefined;

    if (!profile.enabled) {
      this.description = `${this.description ?? ""} — disabled`.trim();
    }

    if (profile.isCurrent) {
      this.iconPath = new vscode.ThemeIcon("pass-filled", new vscode.ThemeColor("testing.iconPassed"));
    } else if (!profile.enabled) {
      this.iconPath = new vscode.ThemeIcon("eye-closed", new vscode.ThemeColor("disabledForeground"));
    } else {
      this.iconPath = new vscode.ThemeIcon("circle-large-outline", new vscode.ThemeColor("foreground"));
    }

    const md = new vscode.MarkdownString();
    const statusIcon = profile.isCurrent
      ? "$(pass-filled)"
      : profile.enabled
        ? "$(circle-large-outline)"
        : "$(eye-closed)";
    md.appendMarkdown(`### ${statusIcon} ${escapeMarkdown(profile.name)}\n\n`);

    if (profile.description) {
      md.appendMarkdown(`*${escapeMarkdown(profile.description)}*\n\n`);
    }

    if (profile.baseUrl || profile.authToken) {
      md.appendMarkdown(`**Connection**\n\n`);
      if (profile.baseUrl) md.appendMarkdown(`- Base URL: \`${escapeMarkdown(profile.baseUrl)}\`\n`);
      if (profile.authToken) md.appendMarkdown(`- Auth Token: \`${escapeMarkdown(maskToken(profile.authToken))}\`\n`);
      md.appendMarkdown(`\n`);
    }

    if (profile.model || profile.smallFastModel) {
      md.appendMarkdown(`**Model**\n\n`);
      if (profile.model) md.appendMarkdown(`- Model: \`${escapeMarkdown(profile.model)}\`\n`);
      if (profile.smallFastModel) md.appendMarkdown(`- Small/Fast: \`${escapeMarkdown(profile.smallFastModel)}\`\n`);
      md.appendMarkdown(`\n`);
    }

    if (profile.provider || profile.providerType || profile.account) {
      md.appendMarkdown(`**Identity**\n\n`);
      if (profile.provider) md.appendMarkdown(`- Provider: ${escapeMarkdown(profile.provider)}\n`);
      if (profile.providerType) md.appendMarkdown(`- Type: ${escapeMarkdown(profile.providerType)}\n`);
      if (profile.account) md.appendMarkdown(`- Account: ${escapeMarkdown(profile.account)}\n`);
      md.appendMarkdown(`\n`);
    }

    md.appendMarkdown(`---\n\n`);
    const status = profile.isCurrent ? "Active" : profile.enabled ? "Available" : "Disabled";
    md.appendMarkdown(`**Status:** ${status}`);
    if (profile.usageCount > 0) md.appendMarkdown(` · **Usage:** ${profile.usageCount}`);
    if (profile.tags && profile.tags.length > 0) {
      md.appendMarkdown(` · **Tags:** ${profile.tags.map((t) => escapeMarkdown(t)).join(", ")}`);
    }
    md.appendMarkdown(`\n\n`);
    md.appendMarkdown(`*Click to switch · Right-click for more*`);

    this.tooltip = md;
    this.command = {
      command: "ccr.switchProfile",
      title: "Switch Profile",
      arguments: [this],
    };
  }
}

export class CodexAuthNode extends vscode.TreeItem {
  constructor(public readonly auth: CodexAuthInfo) {
    super(auth.name, vscode.TreeItemCollapsibleState.None);

    this.contextValue = auth.isCurrent ? "codex-auth-current" : "codex-auth";
    this.description = [auth.email, auth.description].filter(Boolean).join(" · ") || undefined;

    if (auth.isCurrent) {
      this.iconPath = new vscode.ThemeIcon("pass-filled", new vscode.ThemeColor("testing.iconPassed"));
    } else {
      this.iconPath = new vscode.ThemeIcon("key", new vscode.ThemeColor("charts.green"));
    }

    const md = new vscode.MarkdownString();
    md.appendMarkdown(`### $(key) ${escapeMarkdown(auth.name)}\n\n`);
    if (auth.description) {
      md.appendMarkdown(`*${escapeMarkdown(auth.description)}*\n\n`);
    }
    if (auth.email) {
      md.appendMarkdown(`- Email: ${escapeMarkdown(auth.email)}\n`);
    }
    if (auth.savedAt) {
      md.appendMarkdown(`- Saved At: \`${escapeMarkdown(auth.savedAt)}\`\n`);
    }
    if (auth.expiresAt) {
      md.appendMarkdown(`- Expires At: \`${escapeMarkdown(auth.expiresAt)}\`\n`);
    }
    md.appendMarkdown(`\n---\n\n`);
    md.appendMarkdown(`**Status:** ${auth.isCurrent ? "Active" : "Saved"}`);
    if (auth.isVirtual) {
      md.appendMarkdown(` · **Type:** Virtual`);
    }
    md.appendMarkdown(`\n\n*Click to switch auth account · Right-click for more*`);
    this.tooltip = md;

    this.command = {
      command: "ccr.switchCodexAuth",
      title: "Switch Codex Auth",
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

export type TreeNode = PlatformNode | SectionNode | ProfileNode | CodexAuthNode | MessageNode;

export class ProfileTreeProvider implements vscode.TreeDataProvider<TreeNode> {
  private _onDidChangeTreeData = new vscode.EventEmitter<TreeNode | undefined | null>();
  readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

  refresh(): void {
    this._onDidChangeTreeData.fire(undefined);
  }

  getTreeItem(element: TreeNode): vscode.TreeItem {
    return element;
  }

  async getChildren(element?: TreeNode): Promise<TreeNode[]> {
    if (!ccrRootExists()) {
      if (!element) {
        return [new MessageNode("CCR is not initialized. Run 'ccr init' to get started.")];
      }
      return [];
    }

    if (!element) {
      const registry = await readRegistry();
      if (!registry || registry.platforms.length === 0) {
        return [new MessageNode("No platforms configured.")];
      }
      return registry.platforms.map((platform) => new PlatformNode(platform));
    }

    if (element instanceof PlatformNode) {
      if (element.platform.name === "codex") {
        return [
          new SectionNode(getSectionInfo("codex", "profiles")),
          new SectionNode(getSectionInfo("codex", "auth")),
        ];
      }

      return [new SectionNode(getSectionInfo(element.platform.name, "profiles"))];
    }

    if (element instanceof SectionNode) {
      if (element.section.kind === "profiles") {
        const profiles = sortProfiles(await readProfiles(element.section.platformName));
        return profiles.length > 0
          ? profiles.map((profile) => new ProfileNode(profile))
          : [new MessageNode("No profiles configured.")];
      }

      const authAccounts = sortAuthAccounts(await readCodexAuthAccounts());
      return authAccounts.length > 0
        ? authAccounts.map((auth) => new CodexAuthNode(auth))
        : [new MessageNode("No Codex auth accounts configured.")];
    }

    return [];
  }
}
