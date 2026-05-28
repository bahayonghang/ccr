/**
 * llmusage install flow API bindings.
 *
 * Mirrors the Rust types in `ccr_cli::services::install_types` and delegates
 * to Tauri commands registered in `ccr-ui/src-tauri/src/commands/install.rs`.
 */

import { invoke } from '@tauri-apps/api/core'

// ──────────────────────────────────────────────────────────────────────────────
// Types
// ──────────────────────────────────────────────────────────────────────────────

export type Platform = 'macos' | 'linux' | 'windows'
export type PackageManager = 'cargo' | 'homebrew' | 'scoop' | 'winget'
export type DurationClass = 'fast' | 'medium' | 'slow'
export type LogStream = 'stdout' | 'stderr'
export type ProgressStage = 'resolving' | 'downloading' | 'compiling' | 'installing' | 'finalizing'
export type FailureKind = 'spawn_failed' | 'non_zero_exit' | 'internal_error'
export type UnsupportedReason = 'no_package_manager' | 'elevation_required'
export type PostInstallHint = 'reopen_app_for_path'

export interface AttemptId {
  /** UUID string */
  [key: string]: string
}

export interface HostCapabilities {
  platform: Platform
  has_cargo: boolean
  has_homebrew: boolean
  has_scoop: boolean
  has_winget: boolean
  cargo_path: string | null
  homebrew_path: string | null
}

export interface DataRootWarning {
  kind: 'data_root_missing'
  path: string
}

export type DetectionResult =
  | {
      status: 'available'
      path: string
      version: string | null
      data_root_warning: DataRootWarning | null
    }
  | {
      status: 'absent'
      reason: AbsentReason
      data_root_warning: DataRootWarning | null
    }

export type AbsentReason =
  | { kind: 'not_on_path' }
  | { kind: 'not_executable'; exit_code: number | null; stderr_excerpt: string }

export interface InstallPlan {
  platform: Platform
  package_manager: PackageManager
  command: string
  args: string[]
  envs: Record<string, string>
  elevation_required: boolean
  duration_class: DurationClass
  plan_id: string
}

export type PlanOutcome =
  | { kind: 'plan' } & InstallPlan
  | { kind: 'unsupported'; reason: UnsupportedReason }

export type InstallEvent =
  | { type: 'started'; attempt_id: string; plan: InstallPlan }
  | { type: 'log'; attempt_id: string; stream: LogStream; line: string; seq: number }
  | { type: 'progress'; attempt_id: string; stage: ProgressStage; detail: string | null }
  | { type: 'succeeded'; attempt_id: string; duration_ms: number; installed_version: string | null }
  | { type: 'failed'; attempt_id: string; failure_kind: FailureKind; exit_code: number | null; stderr_excerpt: string | null; error_message: string }
  | { type: 'cancelled'; attempt_id: string; requested_at_ms: number }

export type CancelResult =
  | { kind: 'cancelled'; attempt_id: string; requested_at_ms: number }
  | { kind: 'not_running' }
  | { kind: 'already_terminal'; attempt_id: string }

export interface RingBufferSnapshot {
  attempt_id: string | null
  logs: InstallEvent[]
  terminal: InstallEvent | null
}

export interface ManualCommand {
  platform: Platform
  package_manager: PackageManager | null
  title: string
  command_line: string
  notes: string | null
}

export interface ManualCatalog {
  entries: ManualCommand[]
  docs_url: string
}

// ──────────────────────────────────────────────────────────────────────────────
// API Functions
// ──────────────────────────────────────────────────────────────────────────────

/** Detect whether llmusage is available on the host. */
export const llmusageInstallDetect = async (): Promise<DetectionResult> => {
  return invoke('llmusage_install_detect')
}

/** Probe the host for available package managers and platform info. */
export const llmusageInstallProbeCapabilities = async (): Promise<HostCapabilities> => {
  return invoke('llmusage_install_probe_capabilities')
}

/** Generate an install plan for the current host. */
export const llmusageInstallPlan = async (
  detection: DetectionResult,
  capabilities: HostCapabilities,
): Promise<PlanOutcome> => {
  return invoke('llmusage_install_plan', { detection, capabilities })
}

/** Start an install attempt. Returns the attempt ID. */
export const llmusageInstallExecute = async (plan: InstallPlan): Promise<string> => {
  return invoke('llmusage_install_execute', { plan })
}

/** Cancel the current in-flight install attempt. */
export const llmusageInstallCancel = async (attemptId: string): Promise<CancelResult> => {
  return invoke('llmusage_install_cancel', { attemptId })
}

/** Read the most recent events from the ring buffer. */
export const llmusageInstallRecent = async (): Promise<RingBufferSnapshot> => {
  return invoke('llmusage_install_recent')
}

/** Get the manual install catalog (copy-able commands + docs link). */
export const llmusageInstallManualCatalog = async (): Promise<ManualCatalog> => {
  return invoke('llmusage_install_manual_catalog')
}

/** Convenience: detect + probe capabilities in one call. */
export const llmusageInstallCheck = async (): Promise<[DetectionResult, HostCapabilities]> => {
  return invoke('llmusage_install_check')
}
