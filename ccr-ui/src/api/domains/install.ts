/**
 * llmusage install flow API bindings.
 *
 * Mirrors the Rust types in `ccr_cli::services::install_types` and delegates
 * to Tauri commands registered in `ccr-ui/src-tauri/src/commands/install.rs`.
 */

import { invoke } from '@tauri-apps/api/core'
import type { AttemptId } from '@/types/generated/install/AttemptId'
import type { CancelResult } from '@/types/generated/install/CancelResult'
import type { DetectionResult } from '@/types/generated/install/DetectionResult'
import type { HostCapabilities } from '@/types/generated/install/HostCapabilities'
import type { InstallEvent } from '@/types/generated/install/InstallEvent'
import type { InstallPlanView } from '@/types/generated/install/InstallPlanView'
import type { ManualCatalog } from '@/types/generated/install/ManualCatalog'
import type { PlanId } from '@/types/generated/install/PlanId'
import type { PlanOutcome } from '@/types/generated/install/PlanOutcome'
import type { Platform } from '@/types/generated/install/Platform'
import type { ProgressStage } from '@/types/generated/install/ProgressStage'
import type { RingBufferSnapshot } from '@/types/generated/install/RingBufferSnapshot'

// ──────────────────────────────────────────────────────────────────────────────
// Types
// ──────────────────────────────────────────────────────────────────────────────

export type InstallOs = Platform
export type {
  AttemptId,
  CancelResult,
  DetectionResult,
  HostCapabilities,
  InstallEvent,
  InstallPlanView,
  ManualCatalog,
  PlanId,
  PlanOutcome,
  ProgressStage,
  RingBufferSnapshot,
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
export const llmusageInstallExecute = async (planId: PlanId): Promise<AttemptId> => {
  return invoke('llmusage_install_execute', { planId })
}

/** Cancel the current in-flight install attempt. */
export const llmusageInstallCancel = async (attemptId: AttemptId): Promise<CancelResult> => {
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
