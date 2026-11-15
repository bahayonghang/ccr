/**
 * CCR CLI interaction service
 *
 * Spawns `ccr` CLI commands for operations that require
 * atomic writes, file locking, and audit trail.
 */

import * as vscode from "vscode";
import type {
  ProfileCreateRequest,
  ProfileCreationPlatform,
} from "../models/types";
import {
  buildCodexAuthUpdateArgs,
  buildPlatformProfileCreateArgs,
  buildPlatformProfileDeleteArgs,
  buildPlatformProfileDisableArgs,
  buildPlatformProfileEnableArgs,
  buildPlatformAuthOffArgs,
  buildPlatformProfileOffArgs,
  buildPlatformProfileSetFieldArgs,
  buildPlatformProfileSwitchArgs,
  buildClaudeProfileOffArgs,
  buildClaudeProfileSwitchArgs,
  buildCodexProfileOffArgs,
  buildCodexProfileSwitchArgs,
  type CodexAuthUpdateData,
  type PlatformAuthOffData,
  type PlatformProfileMutationData,
  type ProfileFieldValue,
} from "./ccrCliArgs";
import {
  detectCcrCapabilities,
  execCatalogImport,
  execCcr,
  execCcrJson,
  findCcrBinary,
  getCachedCcrCapabilitySnapshot,
  invalidateCcrCapabilityCache,
  resetCcrCache,
  type CliJsonResult,
  type CliResult,
} from "./ccrCliCore";

export {
  detectCcrCapabilities,
  execCatalogImport,
  execCcr,
  findCcrBinary,
  getCachedCcrCapabilitySnapshot,
  invalidateCcrCapabilityCache,
  resetCcrCache,
  type CliJsonResult,
  type CliResult,
};

export type {
  CodexAuthUpdateData,
  PlatformAuthOffData,
  PlatformProfileMutationData,
  ProfileFieldValue,
};

/** Check CCR availability and show message if not found */
export async function checkCcrAvailability(): Promise<boolean> {
  const ccrPath = await findCcrBinary();
  if (!ccrPath) {
    vscode.window.showWarningMessage(
      "CCR CLI not found. Profile switching requires `ccr` in your PATH. " +
      "Install with: cargo install --git https://github.com/bahayonghang/ccr ccr",
    );
    return false;
  }
  return true;
}

export async function execClaudeProfileSwitch(profileName: string): Promise<CliResult> {
  return execCcr(buildClaudeProfileSwitchArgs(profileName));
}

export async function execCodexProfileSwitch(profileName: string): Promise<CliResult> {
  return execCcr(buildCodexProfileSwitchArgs(profileName));
}

export async function execClaudeProfileOff(): Promise<CliJsonResult<PlatformProfileMutationData>> {
  return execCcrJson(buildClaudeProfileOffArgs());
}

export async function execCodexProfileOff(): Promise<CliJsonResult<PlatformProfileMutationData>> {
  return execCcrJson(buildCodexProfileOffArgs());
}

/** Execute `ccr <platform> profile switch <name>` */
export async function execProfileSwitch(platformName: string, profileName: string): Promise<CliResult> {
  return execCcr(buildPlatformProfileSwitchArgs(platformName, profileName));
}

/** Execute `ccr <platform> profile off --json` */
export async function execProfileOff(platformName: string): Promise<CliJsonResult<PlatformProfileMutationData>> {
  return execCcrJson(buildPlatformProfileOffArgs(platformName));
}

/** Execute `ccr <platform> auth off --json` */
export async function execAuthOff(platformName: string): Promise<CliJsonResult<PlatformAuthOffData>> {
  return execCcrJson(buildPlatformAuthOffArgs(platformName));
}

/** Execute `ccr codex auth switch <name>` — switch active Codex auth account */
export async function execCodexAuthSwitch(name: string): Promise<CliResult> {
  return execCcr(["codex", "auth", "switch", name]);
}

/** Execute `ccr codex auth delete <name> --force` — delete a saved Codex auth account */
export async function execCodexAuthDelete(name: string): Promise<CliResult> {
  return execCcr(["codex", "auth", "delete", name, "--force"]);
}

/** Execute `ccr codex auth current` — inspect current Codex auth account */
export async function execCodexAuthCurrent(): Promise<CliResult> {
  return execCcr(["codex", "auth", "current"]);
}

/** Execute `ccr codex auth current --json` — inspect current Codex runtime summary */
export async function execCodexAuthCurrentJson<T>(): Promise<CliJsonResult<T>> {
  return execCcrJson(["codex", "auth", "current", "--json"]);
}

/** Execute `ccr codex quota --json` — inspect per-account Codex quota */
export async function execCodexQuotaJson<T>(): Promise<CliJsonResult<T>> {
  return execCcrJson(["codex", "quota", "--json"]);
}

export async function execPlatformProfileCreate(
  platformName: ProfileCreationPlatform,
  profileName: string,
  config: ProfileCreateRequest,
): Promise<CliJsonResult<PlatformProfileMutationData>> {
  return execCcrJson(buildPlatformProfileCreateArgs(platformName, profileName, config));
}

export async function execPlatformProfileSetField(
  platformName: string,
  profileName: string,
  field: string,
  value: ProfileFieldValue,
): Promise<CliJsonResult<PlatformProfileMutationData>> {
  return execCcrJson(buildPlatformProfileSetFieldArgs(platformName, profileName, field, value));
}

export async function execPlatformProfileEnable(
  platformName: string,
  profileName: string,
): Promise<CliJsonResult<PlatformProfileMutationData>> {
  return execCcrJson(buildPlatformProfileEnableArgs(platformName, profileName));
}

export async function execPlatformProfileDisable(
  platformName: string,
  profileName: string,
  force = false,
): Promise<CliJsonResult<PlatformProfileMutationData>> {
  return execCcrJson(buildPlatformProfileDisableArgs(platformName, profileName, force));
}

export async function execPlatformProfileDelete(
  platformName: string,
  profileName: string,
  force = false,
): Promise<CliJsonResult<PlatformProfileMutationData>> {
  return execCcrJson(buildPlatformProfileDeleteArgs(platformName, profileName, force));
}

export async function execCodexAuthUpdate(
  name: string,
  description: string | undefined,
): Promise<CliJsonResult<CodexAuthUpdateData>> {
  return execCcrJson(buildCodexAuthUpdateArgs(name, description));
}
