/**
 * VS Code-free CCR CLI process runner and capability detector.
 *
 * Kept independent from `vscode` host APIs so `catalogReader` and Node
 * `node:test` suites can exercise capability probing and command routing directly.
 */

import { execFile } from "child_process";
import type { CcrCapabilitySnapshot } from "../models/types";
import { buildCatalogImportArgs } from "./ccrCliArgs";

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

export type RawCommandRunner = (
  command: string,
  args: string[],
  timeout?: number,
) => Promise<CliResult>;

export type BinaryFinder = () => Promise<string | null>;

function defaultRawCommandRunner(
  command: string,
  args: string[],
  timeout = 30_000,
): Promise<CliResult> {
  return new Promise((resolve) => {
    execFile(command, args, { timeout }, (err, stdout, stderr) => {
      const exitCode = err ? (typeof err.code === "number" ? err.code : 1) : 0;
      resolve({
        success: !err,
        stdout: stdout ?? "",
        stderr: stderr ?? "",
        exitCode,
      });
    });
  });
}

function defaultBinaryFinder(): Promise<string | null> {
  return new Promise((resolve) => {
    const cmd = process.platform === "win32" ? "where" : "which";
    execFile(cmd, ["ccr"], (err, stdout) => {
      if (err || !stdout.trim()) {
        resolve(null);
      } else {
        resolve(stdout.trim().split(/\r?\n/)[0]);
      }
    });
  });
}

let rawCommandRunner: RawCommandRunner = defaultRawCommandRunner;
let binaryFinder: BinaryFinder = defaultBinaryFinder;

let cachedCcrPath: string | null = null;
let ccrChecked = false;
let cachedCapabilities: CcrCapabilitySnapshot | null = null;
let capabilityInflight: Promise<CcrCapabilitySnapshot> | null = null;

/** Override process hooks for deterministic unit testing */
export function setCcrCommandHooksForTest(hooks?: {
  runCommand?: RawCommandRunner;
  findBinary?: BinaryFinder;
}): void {
  rawCommandRunner = hooks?.runCommand ?? defaultRawCommandRunner;
  binaryFinder = hooks?.findBinary ?? defaultBinaryFinder;
  resetCcrCache();
}

/** Check if ccr binary is available in PATH */
export async function findCcrBinary(): Promise<string | null> {
  if (ccrChecked) {
    return cachedCcrPath;
  }

  cachedCcrPath = await binaryFinder();
  ccrChecked = true;
  return cachedCcrPath;
}

/** Reset cached binary path (for testing or after PATH changes) */
export function resetCcrCache(): void {
  cachedCcrPath = null;
  ccrChecked = false;
  cachedCapabilities = null;
  capabilityInflight = null;
}

export async function execRawCommand(
  command: string,
  args: string[],
  timeout = 30_000,
): Promise<CliResult> {
  return rawCommandRunner(command, args, timeout);
}

async function execTrimmedCommand(
  command: string,
  args: string[],
  timeout = 30_000,
): Promise<CliResult> {
  const result = await rawCommandRunner(command, args, timeout);
  return {
    ...result,
    stdout: result.stdout.trim(),
    stderr: result.stderr.trim(),
  };
}

async function probeHelpFlagSupport(command: string, args: string[], flag: string): Promise<boolean> {
  const result = await execTrimmedCommand(command, [...args, "--help"], 10_000);
  if (!result.success) {
    return false;
  }
  const combined = `${result.stdout}\n${result.stderr}`;
  return combined.includes(flag);
}

export function getCachedCcrCapabilitySnapshot(): CcrCapabilitySnapshot | null {
  return cachedCapabilities;
}

export function invalidateCcrCapabilityCache(): void {
  resetCcrCache();
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
        supportsProfileExchange: false,
        checkedAt: Date.now(),
      } satisfies CcrCapabilitySnapshot;
    }

    const [
      supportsCodexAuthCurrentJson,
      supportsCodexQuotaJson,
      supportsProfileExport,
      supportsProfileImport,
    ] = await Promise.all([
      probeHelpFlagSupport(binaryPath, ["codex", "auth", "current"], "--json"),
      probeHelpFlagSupport(binaryPath, ["codex", "quota"], "--json"),
      probeHelpFlagSupport(binaryPath, ["claude", "profile", "export"], "--stdout"),
      probeHelpFlagSupport(binaryPath, ["claude", "profile", "import"], "--replace"),
    ]);

    return {
      binaryPath,
      supportsCodexAuthCurrentJson,
      supportsCodexQuotaJson,
      supportsProfileExchange: supportsProfileExport && supportsProfileImport,
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

  return execTrimmedCommand(ccrPath, args);
}

export async function execCcrJson<T>(args: string[]): Promise<CliJsonResult<T>> {
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

/**
 * Import an edited temporary profile catalog back into CCR, preferring
 * `ccr <platform> profile import <file> --replace --yes` when supported on
 * `claude`/`codex`/`grok` and falling back to legacy `ccr import` otherwise.
 */
export async function execCatalogImport(platformName: string, filePath: string): Promise<CliResult> {
  const capabilities = await detectCcrCapabilities();
  const args = buildCatalogImportArgs(
    platformName,
    filePath,
    capabilities.supportsProfileExchange,
  );
  return execCcr(args);
}
