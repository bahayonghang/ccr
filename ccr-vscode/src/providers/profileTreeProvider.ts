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
import {
  ensureCodexQuotaSnapshot,
  getCachedCodexQuotaByAccount,
  getCodexQuotaError,
} from "../services/codexQuotaReader";
import {
  ensureCodexRuntimeSnapshot,
  getCachedCodexRuntimeSnapshot,
  getCodexRuntimeError,
} from "../services/codexRuntimeReader";
import type {
  CodexAuthInfo,
  CodexAuthQuotaInfo,
  CodexRuntimeSnapshot,
  PlatformInfo,
  ProfileInfo,
  TreeSectionInfo,
  TreeSectionKind,
} from "../models/types";
import {
  buildCodexAuthDetailDescriptors,
  buildCodexRuntimeDetails,
  formatCodexAuthDescription,
  formatQuotaReset,
  getCodexPlatformDescription,
  getQuotaTone,
  getSectionInfo,
} from "./profileTreePresentation";
import {
  getPlatformNodeContextValue,
  getProfileNodeContextValue,
  getSectionNodeContextValue,
  supportsProfileMutation,
} from "./profileTreeVisibility";

/** Escape user-controlled strings to prevent Markdown injection */
function escapeMarkdown(str: string): string {
  return str.replace(/([\\`*_{}[\]()#+\-.!|~])/g, "\\$1");
}

/** Platform-specific ThemeColor IDs for codicon icons */
const PLATFORM_THEME_COLORS: Record<string, string> = {
  claude: "charts.orange",
  codex: "charts.green",
};

function getQuotaToneThemeColor(tone: "success" | "warning" | "danger" | "neutral"): string {
  switch (tone) {
    case "danger":
      return "problemsErrorIcon.foreground";
    case "warning":
      return "problemsWarningIcon.foreground";
    case "success":
      return "charts.green";
    default:
      return "foreground";
  }
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
  constructor(
    public readonly platform: PlatformInfo,
    runtimeSnapshot?: CodexRuntimeSnapshot | null,
    runtimeError?: string | null,
  ) {
    super(
      platform.displayName,
      platform.enabled
        ? vscode.TreeItemCollapsibleState.Expanded
        : vscode.TreeItemCollapsibleState.None,
    );
    const supportsMutation = supportsProfileMutation(platform.name);
    this.contextValue = getPlatformNodeContextValue(platform);

    const codiconId = getPlatformCodiconId(platform.name);
    if (!platform.enabled) {
      this.iconPath = new vscode.ThemeIcon("eye-closed", new vscode.ThemeColor("disabledForeground"));
    } else {
      const themeColor = PLATFORM_THEME_COLORS[platform.name] ?? "foreground";
      this.iconPath = new vscode.ThemeIcon(codiconId, new vscode.ThemeColor(themeColor));
    }

    if (!platform.enabled) {
      this.description = "(disabled)";
    } else if (platform.name === "codex") {
      this.description = getCodexPlatformDescription(platform, runtimeSnapshot ?? undefined);
    } else if (platform.currentProfile) {
      this.description = `▸ ${platform.currentProfile}`;
    }

    const md = new vscode.MarkdownString();
    md.appendMarkdown(`### $(${codiconId}) ${escapeMarkdown(platform.displayName)}\n\n`);
    md.appendMarkdown(`**Status:** ${platform.enabled ? "Enabled" : "Disabled"}\n\n`);
    if (platform.name === "codex" && runtimeSnapshot) {
      md.appendMarkdown(`**Runtime:** \`${escapeMarkdown(runtimeSnapshot.runtimeSummary.profileLabel)}\`\n\n`);
      md.appendMarkdown(`**Auth:** \`${escapeMarkdown(runtimeSnapshot.runtimeSummary.authLabel)}\`\n\n`);
      md.appendMarkdown(`**Source:** ${escapeMarkdown(runtimeSnapshot.dataSource)}\n\n`);
      if (runtimeSnapshot.authSidecarLabel) {
        md.appendMarkdown(`**Sidecar:** ${escapeMarkdown(runtimeSnapshot.authSidecarLabel)}\n\n`);
      }
      if (runtimeSnapshot.binaryPath) {
        md.appendMarkdown(`**Binary:** \`${escapeMarkdown(runtimeSnapshot.binaryPath)}\`\n\n`);
      }
    } else if (platform.currentProfile) {
      md.appendMarkdown(`**Current Profile:** \`${escapeMarkdown(platform.currentProfile)}\`\n\n`);
    }
    if (platform.name === "codex" && runtimeError) {
      md.appendMarkdown(`**Runtime Sync:** ${escapeMarkdown(runtimeError)}\n\n`);
    }
    md.appendMarkdown(`---\n\n`);
    md.appendMarkdown(platform.enabled && supportsMutation
      ? `*Expand to manage grouped resources*`
      : `*Expand to browse grouped resources*`);
    this.tooltip = md;
  }
}

export class SectionNode extends vscode.TreeItem {
  constructor(public readonly section: TreeSectionInfo) {
    super(section.label, vscode.TreeItemCollapsibleState.Expanded);
    this.contextValue = getSectionNodeContextValue(section);
    this.description = section.kind === "auth"
      ? "quota & accounts"
      : section.kind === "runtime"
        ? "live runtime"
        : "profiles";
    this.iconPath = new vscode.ThemeIcon(
      section.kind === "auth" ? "key" : section.kind === "runtime" ? "pulse" : "files",
      new vscode.ThemeColor(
        section.kind === "auth"
          ? "charts.green"
          : section.kind === "runtime"
            ? "charts.blue"
            : "foreground",
      ),
    );
    this.tooltip = new vscode.MarkdownString(`**${escapeMarkdown(section.label)}**\n\n${escapeMarkdown(section.description)}`);
  }
}

export class ProfileNode extends vscode.TreeItem {
  constructor(public readonly profile: ProfileInfo) {
    super(profile.name, vscode.TreeItemCollapsibleState.None);

    const supportsMutation = supportsProfileMutation(profile.platformName);
    this.contextValue = getProfileNodeContextValue(profile);

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
    md.appendMarkdown(supportsMutation
      ? `*Click to switch · Right-click for more*`
      : `*Read-only profile · Right-click for more*`);

    this.tooltip = md;
    if (supportsMutation) {
      this.command = {
        command: "ccr.switchProfile",
        title: "Switch Profile",
        arguments: [this],
      };
    }
  }
}

export class CodexAuthNode extends vscode.TreeItem {
  constructor(
    public readonly auth: CodexAuthInfo,
    public readonly quota?: CodexAuthQuotaInfo,
    public readonly quotaFetchError?: string | null,
  ) {
    super(auth.name, vscode.TreeItemCollapsibleState.Expanded);

    this.id = `codex-auth:${auth.name}`;
    this.contextValue = auth.isCurrent ? "codex-auth-current" : "codex-auth";
    const summaryDescription = formatCodexAuthDescription(quota, quotaFetchError);
    this.description = summaryDescription !== undefined
      ? summaryDescription
      : quota?.quota
        ? undefined
        : quotaFetchError
          ? "quota unavailable"
          : "loading quota…";

    if (auth.isCurrent) {
      this.iconPath = new vscode.ThemeIcon("pass-filled", new vscode.ThemeColor("testing.iconPassed"));
    } else {
      const tone = getQuotaTone(quota);
      this.iconPath = new vscode.ThemeIcon("key", new vscode.ThemeColor(getQuotaToneThemeColor(tone)));
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
    if (auth.lastRefresh) {
      md.appendMarkdown(`- Last Refresh: \`${escapeMarkdown(auth.lastRefresh)}\`\n`);
    }
    if (auth.expiresAt) {
      md.appendMarkdown(`- Expires At: \`${escapeMarkdown(auth.expiresAt)}\`\n`);
    }
    if (quota?.quota) {
      md.appendMarkdown(`- 5h Remaining: **${quota.quota.hourlyPercentage}%**\n`);
      md.appendMarkdown(`- 7d Remaining: **${quota.quota.weeklyPercentage}%**\n`);
      if (quota.quota.planType) {
        md.appendMarkdown(`- Plan: ${escapeMarkdown(quota.quota.planType)}\n`);
      }
      const hourlyReset = formatQuotaReset(quota.quota.hourlyResetTime);
      const weeklyReset = formatQuotaReset(quota.quota.weeklyResetTime);
      if (hourlyReset) {
        md.appendMarkdown(`- 5h Reset: \`${escapeMarkdown(hourlyReset)}\`\n`);
      }
      if (weeklyReset) {
        md.appendMarkdown(`- 7d Reset: \`${escapeMarkdown(weeklyReset)}\`\n`);
      }
    } else if (quota?.error) {
      md.appendMarkdown(`- Quota: ${escapeMarkdown(quota.error)}\n`);
    } else if (quotaFetchError) {
      md.appendMarkdown(`- Quota: ${escapeMarkdown(quotaFetchError)}\n`);
    } else {
      md.appendMarkdown(`- Quota: loading...\n`);
    }
    md.appendMarkdown(`\n---\n\n`);
    md.appendMarkdown(`**Status:** ${auth.isCurrent ? "Active" : "Saved"}`);
    if (auth.isVirtual) {
      md.appendMarkdown(` · **Type:** Virtual`);
    }
    md.appendMarkdown(`\n\n*Click label to switch auth account · Expand to inspect quota windows*`);
    this.tooltip = md;

    this.command = {
      command: "ccr.switchCodexAuth",
      title: "Switch Codex Auth",
      arguments: [this],
    };
  }
}

export class CodexAuthDetailNode extends vscode.TreeItem {
  constructor(
    authName: string,
    public readonly detail: { key: string; label: string; description?: string; icon: string; tone: "success" | "warning" | "danger" | "neutral"; tooltip: string },
  ) {
    super(detail.label, vscode.TreeItemCollapsibleState.None);
    this.id = `codex-auth:${authName}:detail:${detail.key}`;
    this.contextValue = "codex-auth-detail";
    this.description = detail.description;
    this.iconPath = new vscode.ThemeIcon(detail.icon, new vscode.ThemeColor(getQuotaToneThemeColor(detail.tone)));
    this.tooltip = new vscode.MarkdownString(
      `**${escapeMarkdown(detail.label)}**\n\n${escapeMarkdown(detail.tooltip)}`,
    );
  }
}

export class RuntimeDetailNode extends vscode.TreeItem {
  constructor(
    public readonly detail: { label: string; description: string; icon: string; tooltip: string },
  ) {
    super(detail.label, vscode.TreeItemCollapsibleState.None);
    this.contextValue = "runtime-detail";
    this.description = detail.description;
    this.iconPath = new vscode.ThemeIcon(detail.icon, new vscode.ThemeColor("charts.blue"));
    this.tooltip = new vscode.MarkdownString(
      `**${escapeMarkdown(detail.label)}**\n\n${escapeMarkdown(detail.tooltip)}`,
    );
  }
}

export class MessageNode extends vscode.TreeItem {
  constructor(message: string) {
    super(message, vscode.TreeItemCollapsibleState.None);
    this.contextValue = "message";
    this.iconPath = new vscode.ThemeIcon("info", new vscode.ThemeColor("charts.blue"));
  }
}

export type TreeNode =
  | PlatformNode
  | SectionNode
  | ProfileNode
  | CodexAuthNode
  | CodexAuthDetailNode
  | RuntimeDetailNode
  | MessageNode;

export class ProfileTreeProvider implements vscode.TreeDataProvider<TreeNode> {
  private _onDidChangeTreeData = new vscode.EventEmitter<TreeNode | undefined | null>();
  readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

  constructor(private readonly onAsyncDataChanged?: () => void) {}

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
      ensureCodexRuntimeSnapshot(this.onAsyncDataChanged);
      const runtimeSnapshot = getCachedCodexRuntimeSnapshot();
      const runtimeError = getCodexRuntimeError();
      return registry.platforms.map((platform) => new PlatformNode(
        platform,
        platform.name === "codex" ? runtimeSnapshot : undefined,
        platform.name === "codex" ? runtimeError : undefined,
      ));
    }

    if (element instanceof PlatformNode) {
      if (element.platform.name === "codex") {
        return [
          new SectionNode(getSectionInfo("codex", "runtime")),
          new SectionNode(getSectionInfo("codex", "profiles")),
          new SectionNode(getSectionInfo("codex", "auth")),
        ];
      }

      return [new SectionNode(getSectionInfo(element.platform.name, "profiles"))];
    }

    if (element instanceof SectionNode) {
      if (element.section.kind === "runtime") {
        ensureCodexRuntimeSnapshot(this.onAsyncDataChanged);
        const runtimeSnapshot = getCachedCodexRuntimeSnapshot();
        const runtimeError = getCodexRuntimeError();
        return buildCodexRuntimeDetails(element.section.platformName === "codex"
          ? {
              name: "codex",
              displayName: "Codex",
              icon: "",
              enabled: true,
              currentProfile: runtimeSnapshot?.runtimeSummary.currentProfileName,
            }
          : {
              name: element.section.platformName,
              displayName: element.section.platformName,
              icon: "",
              enabled: true,
            }, runtimeSnapshot, runtimeError)
          .map((detail) => new RuntimeDetailNode(detail));
      }

      if (element.section.kind === "profiles") {
        const profiles = sortProfiles(await readProfiles(element.section.platformName));
        return profiles.length > 0
          ? profiles.map((profile) => new ProfileNode(profile))
          : [new MessageNode("No profiles configured.")];
      }

      const runtimeSnapshot = getCachedCodexRuntimeSnapshot();
      ensureCodexQuotaSnapshot(this.onAsyncDataChanged);
      const quotaByAccount = getCachedCodexQuotaByAccount();
      const quotaError = getCodexQuotaError();
      const authAccounts = sortAuthAccounts((await readCodexAuthAccounts()).map((auth) => ({
        ...auth,
        isCurrent: runtimeSnapshot?.runtimeSummary.currentAuthName === auth.name,
      })));
      return authAccounts.length > 0
        ? authAccounts.map((auth) => new CodexAuthNode(auth, quotaByAccount[auth.name], quotaError))
        : [new MessageNode("No Codex auth accounts configured.")];
    }

    if (element instanceof CodexAuthNode) {
      return buildCodexAuthDetailDescriptors(element.quota, element.quotaFetchError)
        .map((detail) => new CodexAuthDetailNode(element.auth.name, detail));
    }

    return [];
  }
}
