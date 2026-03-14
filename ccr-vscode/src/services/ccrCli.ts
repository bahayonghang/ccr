/**
 * CCR CLI interaction service
 *
 * Spawns `ccr` CLI commands for operations that require
 * atomic writes, file locking, and audit trail.
 */

import { execFile } from "child_process";
import * as vscode from "vscode";

let cachedCcrPath: string | null = null;
let ccrChecked = false;

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

/** Execute `ccr platform switch <name>` — switch active platform */
export async function execPlatformSwitch(platformName: string): Promise<CliResult> {
  return execCcr(["platform", "switch", platformName]);
}

/** Execute `ccr switch <name>` — switch profile within current platform */
export async function execProfileSwitch(profileName: string): Promise<CliResult> {
  return execCcr(["switch", profileName]);
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

  return new Promise((resolve) => {
    execFile(ccrPath, args, { timeout: 30000 }, (err, stdout, stderr) => {
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
