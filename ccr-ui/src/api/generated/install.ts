/* Generated from commands/handler_registry.rs; do not edit. */

import { invoke } from '@tauri-apps/api/core'
import type { AttemptId } from '@/types/generated/install/AttemptId'
import type { CancelResult } from '@/types/generated/install/CancelResult'
import type { DetectionResult } from '@/types/generated/install/DetectionResult'
import type { HostCapabilities } from '@/types/generated/install/HostCapabilities'
import type { ManualCatalog } from '@/types/generated/install/ManualCatalog'
import type { PlanId } from '@/types/generated/install/PlanId'
import type { PlanOutcome } from '@/types/generated/install/PlanOutcome'
import type { RingBufferSnapshot } from '@/types/generated/install/RingBufferSnapshot'

export const llmusageInstallDetect = (): Promise<DetectionResult> => invoke('llmusage_install_detect')
export const llmusageInstallProbeCapabilities = (): Promise<HostCapabilities> => invoke('llmusage_install_probe_capabilities')
export const llmusageInstallPlan = (detection: DetectionResult, capabilities: HostCapabilities): Promise<PlanOutcome> =>
  invoke('llmusage_install_plan', { detection, capabilities })
export const llmusageInstallExecute = (planId: PlanId): Promise<AttemptId> => invoke('llmusage_install_execute', { planId })
export const llmusageInstallCancel = (attemptId: AttemptId): Promise<CancelResult> => invoke('llmusage_install_cancel', { attemptId })
export const llmusageInstallRecent = (): Promise<RingBufferSnapshot> => invoke('llmusage_install_recent')
export const llmusageInstallManualCatalog = (): Promise<ManualCatalog> => invoke('llmusage_install_manual_catalog')
export const llmusageInstallCheck = (): Promise<[DetectionResult, HostCapabilities]> => invoke('llmusage_install_check')
