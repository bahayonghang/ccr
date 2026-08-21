/**
 * CCR CLI interaction service
 *
 * Spawns `ccr` CLI commands for operations that require
 * atomic writes, file locking, and audit trail.
 */

import { execFile } from "child_process";
import * as vscode from "vscode";
import type {
  CcrCapabilitySnapshot,
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

let cachedCcrPath: string | null = null;
let ccrChecked = false;
let cachedCapabilities: CcrCapabilitySnapshot | null = null;
let capabilityInflight: Promise<CcrCapabilitySnapshot> | null = null;

/** Check if ccr binary is available in PATH */
export async function findCcrBinary(): Promise<string | null> {
  if (ccrChecked) {
    return cachedCcrPath;
  }

  return new Promise((resolve) => {
    const cmd = process.platform === "win32" ? "where" : "which";
    execFile(cmd, ["ccr"], (err, stdout) => {
      ccrChecked = true;
      if (err || !stdout.trim()) {
        cachedCcrPath = null;
        resolve(null);
      } else {
        cachedCcrPath = stdout.trim().split(/\r?\n/)[0];
        resolve(cachedCcrPath);
      }
    });
  });
}

/** Reset cached binary path (for testing or after PATH changes) */
export function resetCcrCache(): void {
  cachedCcrPath = null;
  ccrChecked = false;
  cachedCapabilities = null;
  capabilityInflight = null;
}

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

/** Result of a CLI command execution */
export interface CliResult {
  success: boolean;
  stdout: string;
  stderr: string;
  exitCode: number | null;
}

export interface CliJsonResult<T> extends CliResult {
  data?: T;
}

export type { CodexAuthUpdateData, PlatformAuthOffData, PlatformProfileMutationData, ProfileFieldValue };

function execFileCommand(
  command: string,
  args: string[],
  timeout = 30_000,
): Promise<CliResult> {
  return new Promise((resolve) => {
    execFile(command, args, { timeout }, (err, stdout, stderr) => {
      const exitCode = err ? (typeof err.code === "number" ? err.code : 1) : 0;
      resolve({
        success: !err,
        stdout: stdout?.trim() ?? "",
        stderr: stderr?.trim() ?? "",
        exitCode,
      });
    });
  });
}

async function probeHelpFlagSupport(command: string, args: string[], flag: string): Promise<boolean> {
  const result = await execFileCommand(command, [...args, "--help"], 10_000);
  const combined = `${result.stdout}\n${result.stderr}`;
  return combined.includes(flag);
}

export function getCachedCcrCapabilitySnapshot(): CcrCapabilitySnapshot | null {
  return cachedCapabilities;
}

export function invalidateCcrCapabilityCache(): void {
  cachedCapabilities = null;
  capabilityInflight = null;
}

export async function detectCcrCapabilities(force = false): Promise<CcrCapabilitySnapshot> {
  if (!force && cachedCapabilities) {
    return cachedCapabilities;
  }
  if (!force && capabilityInflight) {
    return capabilityInflight;
  }

  capabilityInflight = (async () => {
    const binaryPath = await findCcrBinary();
    if (!binaryPath) {
      return {
        supportsCodexAuthCurrentJson: false,
        supportsCodexQuotaJson: false,
        checkedAt: Date.now(),
      } satisfies CcrCapabilitySnapshot;
    }

    const [supportsCodexAuthCurrentJson, supportsCodexQuotaJson] = await Promise.all([
      probeHelpFlagSupport(binaryPath, ["codex", "auth", "current"], "--json"),
      probeHelpFlagSupport(binaryPath, ["codex", "quota"], "--json"),
    ]);

    return {
      binaryPath,
      supportsCodexAuthCurrentJson,
      supportsCodexQuotaJson,
      checkedAt: Date.now(),
    } satisfies CcrCapabilitySnapshot;
  })()
    .then((snapshot) => {
      cachedCapabilities = snapshot;
      return snapshot;
    })
    .finally(() => {
      capabilityInflight = null;
    });

  return capabilityInflight;
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

/** Execute an arbitrary ccr command */
export async function execCcr(args: string[]): Promise<CliResult> {
  const ccrPath = await findCcrBinary();
  if (!ccrPath) {
    return {
      success: false,
      stdout: "",
      stderr: "CCR CLI not found. Please install CCR and ensure it is in your PATH.",
      exitCode: null,
    };
  }

  return execFileCommand(ccrPath, args);
}

async function execCcrJson<T>(args: string[]): Promise<CliJsonResult<T>> {
  const result = await execCcr(args);
  if (!result.success) {
    return result;
  }

  try {
    return {
      ...result,
      data: result.stdout ? JSON.parse(result.stdout) as T : undefined,
    };
  } catch (error) {
    return {
      ...result,
      success: false,
      stderr: `Failed to parse JSON output: ${error}`,
    };
  }
}
