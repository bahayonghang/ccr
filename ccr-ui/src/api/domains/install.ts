/**
 * Compatibility exports for the registry-generated llmusage install client.
 */

import type { Platform } from '@/types/generated/install/Platform'

export {
  llmusageInstallCancel,
  llmusageInstallCheck,
  llmusageInstallDetect,
  llmusageInstallExecute,
  llmusageInstallManualCatalog,
  llmusageInstallPlan,
  llmusageInstallProbeCapabilities,
  llmusageInstallRecent,
} from '../generated/install'

export type InstallOs = Platform
export type { AttemptId } from '@/types/generated/install/AttemptId'
export type { CancelResult } from '@/types/generated/install/CancelResult'
export type { DetectionResult } from '@/types/generated/install/DetectionResult'
export type { HostCapabilities } from '@/types/generated/install/HostCapabilities'
export type { InstallEvent } from '@/types/generated/install/InstallEvent'
export type { InstallPlanView } from '@/types/generated/install/InstallPlanView'
export type { ManualCatalog } from '@/types/generated/install/ManualCatalog'
export type { PlanId } from '@/types/generated/install/PlanId'
export type { PlanOutcome } from '@/types/generated/install/PlanOutcome'
export type { ProgressStage } from '@/types/generated/install/ProgressStage'
export type { RingBufferSnapshot } from '@/types/generated/install/RingBufferSnapshot'
