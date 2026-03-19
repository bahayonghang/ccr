/**
 * CCR VSCode Extension — entry point
 *
 * Registers all providers, commands, and watchers.
 */

import * as vscode from "vscode";
import { CodexAuthNode, PlatformNode, ProfileNode, ProfileTreeProvider, SectionNode } from "./providers/profileTreeProvider";
import { StatusBarProvider } from "./providers/statusBarProvider";
import { CcrWatcher } from "./services/ccrWatcher";
import {
  checkCcrAvailability,
  execCodexAuthDelete,
  execCodexAuthSwitch,
  execPlatformSwitch,
  execProfileSwitch,
} from "./services/ccrCli";
import { readCodexAuthAccounts, writeCodexAuthDescription } from "./services/codexAuthReader";
import { createProfile, deleteProfile, readProfiles, readRegistry, toggleProfileEnabled, writeProfileField } from "./services/tomlReader";
import { getProfilesPath } from "./services/ccrPaths";
import {
  EDITABLE_FIELDS,
  getEditableProfileFields,
  isProfileCreationPlatform,
  type ProfileCreateRequest,
  type ProfileCreationPlatform,
} from "./models/types";
import { ProfileEditorPanel } from "./providers/profileEditorPanel";

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  const treeProvider = new ProfileTreeProvider();
  const treeView = vscode.window.createTreeView("ccr-profiles", {
    treeDataProvider: treeProvider,
    showCollapseAll: true,
  });
  context.subscriptions.push(treeView);

  const statusBar = new StatusBarProvider();
  context.subscriptions.push(statusBar);

  const refreshAll = () => {
    treeProvider.refresh();
    statusBar.update();
  };

  const watcher = new CcrWatcher();
  watcher.onChange(refreshAll);
  context.subscriptions.push(watcher);

  context.subscriptions.push(
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (event.affectsConfiguration("ccr")) {
        refreshAll();
      }
    }),
  );

  checkCcrAvailability();

  context.subscriptions.push(
    vscode.commands.registerCommand("ccr.refreshProfiles", refreshAll),
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("ccr.switchProfile", async (node?: ProfileNode) => {
      if (node instanceof ProfileNode) {
        if (node.profile.isCurrent) {
          return;
        }
        await doSwitch(node.profile.platformName, node.profile.name, refreshAll);
        return;
      }

      await showSwitchQuickPick(refreshAll);
    }),
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("ccr.switchProfileForPlatform", async (platformName?: string) => {
      if (typeof platformName === "string" && platformName.length > 0) {
        await showSwitchQuickPick(refreshAll, platformName);
        return;
      }
      await showSwitchQuickPick(refreshAll);
    }),
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("ccr.addProfile", async (node?: PlatformNode | SectionNode) => {
      await showAddProfileFlow(context, refreshAll, node);
    }),
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("ccr.addProfileForPlatform", async (node?: PlatformNode | SectionNode | string) => {
      await showAddProfileFlow(context, refreshAll, node);
    }),
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("ccr.editProfileField", async (node?: ProfileNode) => {
      if (!(node instanceof ProfileNode)) {
        vscode.window.showWarningMessage("Please select a profile to edit.");
        return;
      }
      await editProfileField(node, refreshAll);
    }),
  );

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

  context.subscriptions.push(
    vscode.commands.registerCommand("ccr.deleteProfile", async (node?: ProfileNode) => {
      if (!(node instanceof ProfileNode)) {
        vscode.window.showWarningMessage("Please select a profile to delete.");
        return;
      }

      const confirm = await vscode.window.showWarningMessage(
        `Delete profile '${node.profile.name}' from ${node.profile.platformName}?`,
        { modal: true },
        "Delete",
      );
      if (confirm !== "Delete") {
        return;
      }

      try {
        const nextCurrent = await deleteProfile(node.profile.platformName, node.profile.name);
        vscode.window.showInformationMessage(
          nextCurrent
            ? `Deleted '${node.profile.name}'. Current profile is now '${nextCurrent}'.`
            : `Deleted '${node.profile.name}'. No profiles remain.`,
        );
        refreshAll();
      } catch (err) {
        vscode.window.showErrorMessage(`Failed to delete profile: ${err}`);
      }
    }),
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("ccr.editProfileVisual", async (node?: ProfileNode) => {
      if (!(node instanceof ProfileNode)) {
        vscode.window.showWarningMessage("Please select a profile to edit.");
        return;
      }
      ProfileEditorPanel.createOrShow(context.extensionUri, node.profile, refreshAll);
    }),
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("ccr.switchCodexAuth", async (node?: CodexAuthNode) => {
      if (node instanceof CodexAuthNode) {
        if (node.auth.isCurrent) {
          return;
        }
        await doSwitchCodexAuth(node.auth.name, refreshAll);
        return;
      }

      await showCodexAuthQuickPick(refreshAll);
    }),
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("ccr.editCodexAuth", async (node?: CodexAuthNode) => {
      if (!(node instanceof CodexAuthNode)) {
        vscode.window.showWarningMessage("Please select a Codex auth account to edit.");
        return;
      }

      const newDescription = await vscode.window.showInputBox({
        prompt: `Edit description for Codex auth '${node.auth.name}'`,
        value: node.auth.description ?? "",
        placeHolder: "Optional description",
      });

      if (newDescription === undefined) {
        return;
      }

      try {
        await writeCodexAuthDescription(node.auth.name, newDescription || undefined);
        vscode.window.showInformationMessage(`Updated Codex auth '${node.auth.name}'.`);
        refreshAll();
      } catch (err) {
        vscode.window.showErrorMessage(`Failed to update Codex auth: ${err}`);
      }
    }),
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("ccr.deleteCodexAuth", async (node?: CodexAuthNode) => {
      if (!(node instanceof CodexAuthNode)) {
        vscode.window.showWarningMessage("Please select a Codex auth account to delete.");
        return;
      }

      const confirm = await vscode.window.showWarningMessage(
        `Delete Codex auth '${node.auth.name}'?`,
        { modal: true },
        "Delete",
      );
      if (confirm !== "Delete") {
        return;
      }

      const available = await checkCcrAvailability();
      if (!available) {
        return;
      }

      const result = await execCodexAuthDelete(node.auth.name);
      if (result.success) {
        vscode.window.showInformationMessage(`Deleted Codex auth '${node.auth.name}'.`);
        refreshAll();
      } else {
        vscode.window.showErrorMessage(`Failed to delete Codex auth: ${result.stderr || "Unknown error"}`);
      }
    }),
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("ccr.openProfilesFile", async (node?: PlatformNode | ProfileNode) => {
      let platformName: string | undefined;
      if (node instanceof PlatformNode) {
        platformName = node.platform.name;
      } else if (node instanceof ProfileNode) {
        platformName = node.profile.platformName;
      }

      if (!platformName) {
        const registry = readRegistry();
        if (!registry || registry.platforms.length === 0) {
          vscode.window.showWarningMessage("No platforms available.");
          return;
        }
        const picked = await vscode.window.showQuickPick(
          registry.platforms.map((platform) => ({
            label: `${platform.icon} ${platform.displayName}`,
            platformName: platform.name,
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

  context.subscriptions.push(
    vscode.commands.registerCommand("ccr.selectStatusBarPlatform", async () => {
      const registry = readRegistry();
      if (!registry || registry.platforms.length === 0) {
        vscode.window.showWarningMessage("No platforms available.");
        return;
      }

      const picked = await vscode.window.showQuickPick(
        registry.platforms.map((platform) => ({
          label: `${platform.icon} ${platform.displayName}`,
          description: platform.currentProfile ? `current: ${platform.currentProfile}` : undefined,
          platformName: platform.name,
        })),
        {
          placeHolder: "Select platform to pin in the CCR status bar",
        },
      );

      if (!picked) {
        return;
      }

      const config = vscode.workspace.getConfiguration("ccr");
      await config.update("statusBar.platform", picked.platformName, vscode.ConfigurationTarget.Global);
      await config.update("statusBar.mode", "pinned", vscode.ConfigurationTarget.Global);
      refreshAll();
      vscode.window.showInformationMessage(
        `Pinned CCR status bar to ${picked.platformName}.`,
      );
    }),
  );
}

export function deactivate(): void {
  ProfileEditorPanel.disposeAll();
}

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

async function doSwitchCodexAuth(name: string, refreshAll: () => void): Promise<void> {
  const available = await checkCcrAvailability();
  if (!available) return;

  const confirmEnabled = vscode.workspace.getConfiguration("ccr").get<boolean>("confirmBeforeSwitch", true);
  if (confirmEnabled) {
    const confirm = await vscode.window.showWarningMessage(
      `Switch Codex auth to "${name}"?`,
      { modal: true },
      "Yes",
    );
    if (confirm !== "Yes") return;
  }

  const result = await execCodexAuthSwitch(name);
  if (result.success) {
    vscode.window.showInformationMessage(`Switched Codex auth to '${name}'.`);
    refreshAll();
  } else {
    vscode.window.showErrorMessage(`Codex auth switch failed: ${result.stderr || "Unknown error"}`);
  }
}

async function showSwitchQuickPick(
  refreshAll: () => void,
  platformOverride?: string,
): Promise<void> {
  const registry = readRegistry();
  if (!registry || registry.platforms.length === 0) {
    vscode.window.showWarningMessage("No platforms available.");
    return;
  }

  let platformName: string;
  if (platformOverride) {
    const exists = registry.platforms.some((platform) => platform.name === platformOverride);
    if (!exists) {
      vscode.window.showWarningMessage(`Platform '${platformOverride}' is not available.`);
      return;
    }
    platformName = platformOverride;
  } else if (registry.platforms.length === 1) {
    platformName = registry.platforms[0].name;
  } else {
    const picked = await vscode.window.showQuickPick(
      registry.platforms.map((platform) => ({
        label: `${platform.icon} ${platform.displayName}`,
        description: platform.currentProfile ? `current: ${platform.currentProfile}` : undefined,
        platformName: platform.name,
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

  const items = profiles.map((profile) => ({
    label: profile.isCurrent ? `$(check) ${profile.name}` : `     ${profile.name}`,
    description: [profile.provider, profile.model].filter(Boolean).join(" | ") || undefined,
    detail: profile.description,
    profileName: profile.name,
    isCurrent: profile.isCurrent,
  }));

  const picked = await vscode.window.showQuickPick(items, {
    placeHolder: `Select profile for ${platformName}`,
  });

  if (!picked || picked.isCurrent) return;

  await doSwitch(platformName, picked.profileName, refreshAll);
}

async function showCodexAuthQuickPick(refreshAll: () => void): Promise<void> {
  const accounts = readCodexAuthAccounts();
  if (accounts.length === 0) {
    vscode.window.showWarningMessage("No Codex auth accounts available.");
    return;
  }

  const picked = await vscode.window.showQuickPick(
    accounts.map((auth) => ({
      label: auth.isCurrent ? `$(check) ${auth.name}` : auth.name,
      description: auth.email,
      detail: auth.description,
      name: auth.name,
      isCurrent: auth.isCurrent,
    })),
    { placeHolder: "Select Codex auth account" },
  );

  if (!picked || picked.isCurrent) {
    return;
  }

  await doSwitchCodexAuth(picked.name, refreshAll);
}

async function showAddProfileFlow(
  context: vscode.ExtensionContext,
  refreshAll: () => void,
  source?: PlatformNode | SectionNode | ProfileCreationPlatform | string,
): Promise<void> {
  const platformName = await resolveProfileCreationPlatform(source);
  if (!platformName) {
    return;
  }

  ProfileEditorPanel.createForNewProfile(context.extensionUri, platformName, async (draft) => {
    try {
      const trimmedName = draft.name.trim();
      if (!trimmedName) {
        throw new Error("Profile name cannot be empty.");
      }

      const config: ProfileCreateRequest = {
        description: normalizeOptionalText(draft.description),
        model: normalizeOptionalText(draft.model),
        small_fast_model: normalizeOptionalText(draft.smallFastModel),
        provider: normalizeOptionalText(draft.provider),
        provider_type: normalizeOptionalText(draft.providerType),
        account: normalizeOptionalText(draft.account),
        tags: normalizeTags(draft.tags),
        enabled: draft.enabled,
      };

      if (platformName === "claude") {
        config.base_url = normalizeOptionalText(draft.baseUrl);
        config.auth_token = normalizeOptionalText(draft.authToken);
      }

      await createProfile(platformName, trimmedName, config);
      refreshAll();
      vscode.window.showInformationMessage(`Created ${platformName} profile '${trimmedName}'.`);
    } catch (err) {
      throw err;
    }
  });
}

async function resolveProfileCreationPlatform(
  source?: PlatformNode | SectionNode | ProfileCreationPlatform | string,
): Promise<ProfileCreationPlatform | undefined> {
  if (source instanceof PlatformNode) {
    return isProfileCreationPlatform(source.platform.name) ? source.platform.name : undefined;
  }

  if (source instanceof SectionNode) {
    return source.section.kind === "profiles" && isProfileCreationPlatform(source.section.platformName)
      ? source.section.platformName
      : undefined;
  }

  if (typeof source === "string" && isProfileCreationPlatform(source)) {
    return source;
  }

  const registry = readRegistry();
  const items = (registry?.platforms ?? [])
    .filter((platform): platform is typeof platform & { name: ProfileCreationPlatform } => (
      platform.enabled && isProfileCreationPlatform(platform.name)
    ))
    .map((platform) => ({
      label: `${platform.icon} ${platform.displayName}`,
      description: platform.currentProfile ? `current: ${platform.currentProfile}` : undefined,
      platformName: platform.name,
    }));

  if (items.length === 0) {
    vscode.window.showWarningMessage("Claude and Codex profile creation is not available.");
    return undefined;
  }

  if (items.length === 1) {
    return items[0].platformName;
  }

  const picked = await vscode.window.showQuickPick(items, {
    placeHolder: "Select platform for the new profile",
  });

  return picked?.platformName;
}

function normalizeOptionalText(value: string | undefined): string | undefined {
  const trimmed = value?.trim();
  return trimmed ? trimmed : undefined;
}

function normalizeTags(tags: string[] | undefined): string[] | undefined {
  if (!tags || tags.length === 0) {
    return undefined;
  }

  const normalized = tags.map((tag) => tag.trim()).filter(Boolean);
  return normalized.length > 0 ? normalized : undefined;
}

async function editProfileField(
  node: ProfileNode,
  refreshAll: () => void,
): Promise<void> {
  const profile = node.profile;

  const editableFieldKeys = getEditableProfileFields(profile.platformName);
  const fieldItems = EDITABLE_FIELDS
    .filter((field) => editableFieldKeys.includes(field.key))
    .map((field) => {
      const currentVal = profile[field.key as keyof typeof profile];
      return {
        label: field.label,
        description: typeof currentVal === "string"
          ? (field.key === "authToken" ? "****" : currentVal)
          : currentVal !== undefined
            ? String(currentVal)
            : "(empty)",
        field,
      };
    });

  const picked = await vscode.window.showQuickPick(fieldItems, {
    placeHolder: `Edit field for profile '${profile.name}'`,
  });
  if (!picked) return;

  const currentValue = profile[picked.field.key as keyof typeof profile];
  const currentStr = typeof currentValue === "string"
    ? currentValue
    : currentValue !== undefined
      ? String(currentValue)
      : "";

  const newValue = await vscode.window.showInputBox({
    prompt: `Edit ${picked.field.label} for '${profile.name}'`,
    value: currentStr,
    placeHolder: `Enter new value for ${picked.field.label}`,
  });

  if (newValue === undefined) return;

  try {
    await writeProfileField(profile.platformName, profile.name, picked.field.tomlKey, newValue || undefined);
    vscode.window.showInformationMessage(
      `Updated ${picked.field.label} for '${profile.name}'.`,
    );
    refreshAll();
  } catch (err) {
    vscode.window.showErrorMessage(`Failed to update profile: ${err}`);
  }
}
