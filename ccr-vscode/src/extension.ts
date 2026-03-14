/**
 * CCR VSCode Extension — entry point
 *
 * Registers all providers, commands, and watchers.
 */

import * as vscode from "vscode";
import { ProfileTreeProvider, ProfileNode, PlatformNode } from "./providers/profileTreeProvider";
import { StatusBarProvider } from "./providers/statusBarProvider";
import { CcrWatcher } from "./services/ccrWatcher";
import { checkCcrAvailability, execPlatformSwitch, execProfileSwitch } from "./services/ccrCli";
import { readProfiles, readRegistry, writeProfileField, toggleProfileEnabled } from "./services/tomlReader";
import { getProfilesPath } from "./services/ccrPaths";
import { EDITABLE_FIELDS } from "./models/types";
import { ProfileEditorPanel } from "./providers/profileEditorPanel";

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  // ── Providers ──
  const treeProvider = new ProfileTreeProvider();
  const treeView = vscode.window.createTreeView("ccr-profiles", {
    treeDataProvider: treeProvider,
    showCollapseAll: true,
  });
  context.subscriptions.push(treeView);

  const statusBar = new StatusBarProvider();
  context.subscriptions.push(statusBar);

  /** Refresh both tree and status bar */
  const refreshAll = () => {
    treeProvider.refresh();
    statusBar.update();
  };

  // ── File watcher ──
  const watcher = new CcrWatcher();
  watcher.onChange(refreshAll);
  context.subscriptions.push(watcher);

  // ── Check CCR availability (non-blocking) ──
  checkCcrAvailability();

  // ── Commands ──

  // Refresh profiles
  context.subscriptions.push(
    vscode.commands.registerCommand("ccr.refreshProfiles", refreshAll),
  );

  // Switch profile (via TreeView click or QuickPick)
  context.subscriptions.push(
    vscode.commands.registerCommand("ccr.switchProfile", async (node?: ProfileNode) => {
      if (node instanceof ProfileNode) {
        // Called from TreeView click
        if (node.profile.isCurrent) {
          return; // Already current, no-op
        }
        await doSwitch(node.profile.platformName, node.profile.name, refreshAll);
      } else {
        // Called from command palette or status bar — show QuickPick
        await showSwitchQuickPick(refreshAll);
      }
    }),
  );

  // Edit profile field
  context.subscriptions.push(
    vscode.commands.registerCommand("ccr.editProfileField", async (node?: ProfileNode) => {
      if (!(node instanceof ProfileNode)) {
        vscode.window.showWarningMessage("Please select a profile to edit.");
        return;
      }
      await editProfileField(node, refreshAll);
    }),
  );

  // Toggle profile enabled
  context.subscriptions.push(
    vscode.commands.registerCommand("ccr.toggleProfileEnabled", async (node?: ProfileNode) => {
      if (!(node instanceof ProfileNode)) {
        vscode.window.showWarningMessage("Please select a profile to toggle.");
        return;
      }
      try {
        const newState = await toggleProfileEnabled(node.profile.platformName, node.profile.name);
        vscode.window.showInformationMessage(
          `Profile '${node.profile.name}' ${newState ? "enabled" : "disabled"}.`,
        );
        refreshAll();
      } catch (err) {
        vscode.window.showErrorMessage(`Failed to toggle profile: ${err}`);
      }
    }),
  );

  // Visual profile editor (Webview)
  context.subscriptions.push(
    vscode.commands.registerCommand("ccr.editProfileVisual", async (node?: ProfileNode) => {
      if (!(node instanceof ProfileNode)) {
        vscode.window.showWarningMessage("Please select a profile to edit.");
        return;
      }
      ProfileEditorPanel.createOrShow(context.extensionUri, node.profile, refreshAll);
    }),
  );

  // Open profiles.toml in editor
  context.subscriptions.push(
    vscode.commands.registerCommand("ccr.openProfilesFile", async (node?: PlatformNode | ProfileNode) => {
      let platformName: string | undefined;
      if (node instanceof PlatformNode) {
        platformName = node.platform.name;
      } else if (node instanceof ProfileNode) {
        platformName = node.profile.platformName;
      }

      if (!platformName) {
        // Fallback: ask user to pick a platform
        const registry = readRegistry();
        if (!registry || registry.platforms.length === 0) {
          vscode.window.showWarningMessage("No platforms available.");
          return;
        }
        const picked = await vscode.window.showQuickPick(
          registry.platforms.map((p) => ({
            label: `${p.icon} ${p.displayName}`,
            platformName: p.name,
          })),
          { placeHolder: "Select platform to open profiles.toml" },
        );
        if (!picked) return;
        platformName = picked.platformName;
      }

      const filePath = getProfilesPath(platformName);
      try {
        const doc = await vscode.workspace.openTextDocument(vscode.Uri.file(filePath));
        await vscode.window.showTextDocument(doc);
      } catch {
        vscode.window.showErrorMessage(`Cannot open ${filePath}. File may not exist.`);
      }
    }),
  );
}

export function deactivate(): void {
  ProfileEditorPanel.disposeAll();
}

// ── Helpers ──

async function doSwitch(
  platform: string,
  profileName: string,
  refreshAll: () => void,
): Promise<void> {
  const confirmEnabled = vscode.workspace.getConfiguration("ccr").get<boolean>("confirmBeforeSwitch", true);
  if (confirmEnabled) {
    const confirm = await vscode.window.showWarningMessage(
      `Switch to profile "${profileName}" on ${platform}?`,
      { modal: true },
      "Yes",
    );
    if (confirm !== "Yes") return;
  }

  const available = await checkCcrAvailability();
  if (!available) return;

  await vscode.window.withProgress(
    {
      location: vscode.ProgressLocation.Notification,
      title: `Switching to ${profileName}...`,
      cancellable: false,
    },
    async () => {
      // Issue 1: Correct CLI syntax — switch platform first if needed
      const registry = readRegistry();
      const currentPlatform = registry?.currentPlatform ?? "";

      if (currentPlatform !== platform) {
        const platResult = await execPlatformSwitch(platform);
        if (!platResult.success) {
          vscode.window.showErrorMessage(
            `Platform switch failed: ${platResult.stderr || "Unknown error"}`,
          );
          return;
        }
      }

      const result = await execProfileSwitch(profileName);
      if (result.success) {
        vscode.window.showInformationMessage(`Switched to profile '${profileName}'.`);
      } else {
        vscode.window.showErrorMessage(
          `Switch failed: ${result.stderr || "Unknown error"}`,
        );
      }
    },
  );

  refreshAll();
}

async function showSwitchQuickPick(
  refreshAll: () => void,
): Promise<void> {
  const registry = readRegistry();
  if (!registry || registry.platforms.length === 0) {
    vscode.window.showWarningMessage("No platforms available.");
    return;
  }

  // If only one platform, skip platform selection
  let platformName: string;
  if (registry.platforms.length === 1) {
    platformName = registry.platforms[0].name;
  } else {
    const picked = await vscode.window.showQuickPick(
      registry.platforms.map((p) => ({
        label: `${p.icon} ${p.displayName}`,
        description: p.currentProfile ? `current: ${p.currentProfile}` : undefined,
        platformName: p.name,
      })),
      { placeHolder: "Select platform" },
    );
    if (!picked) return;
    platformName = picked.platformName;
  }

  const profiles = readProfiles(platformName);
  if (profiles.length === 0) {
    vscode.window.showWarningMessage(`No profiles for ${platformName}.`);
    return;
  }

  const items = profiles.map((p) => ({
    label: p.isCurrent ? `$(check) ${p.name}` : `     ${p.name}`,
    description: [p.provider, p.model].filter(Boolean).join(" | ") || undefined,
    detail: p.description,
    profileName: p.name,
    isCurrent: p.isCurrent,
  }));

  const picked = await vscode.window.showQuickPick(items, {
    placeHolder: `Select profile for ${platformName}`,
  });

  if (!picked || picked.isCurrent) return;

  await doSwitch(platformName, picked.profileName, refreshAll);
}

async function editProfileField(
  node: ProfileNode,
  refreshAll: () => void,
): Promise<void> {
  const profile = node.profile;

  // Pick which field to edit
  const fieldItems = EDITABLE_FIELDS.map((f) => {
    const currentVal = profile[f.key as keyof typeof profile];
    return {
      label: f.label,
      description: typeof currentVal === "string"
        ? (f.key === "authToken" ? "****" : currentVal)
        : currentVal !== undefined
          ? String(currentVal)
          : "(empty)",
      field: f,
    };
  });

  const picked = await vscode.window.showQuickPick(fieldItems, {
    placeHolder: `Edit field for profile '${profile.name}'`,
  });
  if (!picked) return;

  // Get current value using camelCase key
  const currentValue = profile[picked.field.key as keyof typeof profile];
  const currentStr = typeof currentValue === "string"
    ? currentValue
    : currentValue !== undefined
      ? String(currentValue)
      : "";

  // Input new value
  const newValue = await vscode.window.showInputBox({
    prompt: `Edit ${picked.field.label} for '${profile.name}'`,
    value: currentStr,
    placeHolder: `Enter new value for ${picked.field.label}`,
  });

  if (newValue === undefined) return; // Cancelled

  try {
    // Write using snake_case TOML key
    await writeProfileField(profile.platformName, profile.name, picked.field.tomlKey, newValue || undefined);
    vscode.window.showInformationMessage(
      `Updated ${picked.field.label} for '${profile.name}'.`,
    );
    refreshAll();
  } catch (err) {
    vscode.window.showErrorMessage(`Failed to update profile: ${err}`);
  }
}
