// V2 Usage Analytics Types —— 由 ts-rs 从 Rust DTO 生成（src/types/generated/usage/）。
// 本文件是兼容 shim：旧导出名 → 生成类型别名，既有 `@/types/usage` 导入面不变。
// 改字段请改 Rust 侧（src-tauri services/usage.rs 等），然后 `just tauri-bindings` 再生成；
// 手写仅保留纯视图类型（不上 IPC wire 的部分）。

export type { UsageSummaryDto as UsageSummary } from './generated/usage/UsageSummaryDto'
export type { DailyTrendDto as DailyTrend } from './generated/usage/DailyTrendDto'
export type { ModelStatDto as ModelStat } from './generated/usage/ModelStatDto'
export type { ProjectStatDto as ProjectStat } from './generated/usage/ProjectStatDto'
export type { SourceBreakdownDto as SourceBreakdown } from './generated/usage/SourceBreakdownDto'
export type { ProviderBreakdownDto as ProviderBreakdown } from './generated/usage/ProviderBreakdownDto'
export type { UsageRecordDto as UsageRecordV2 } from './generated/usage/UsageRecordDto'
export type { PaginatedLogsDto as PaginatedLogs } from './generated/usage/PaginatedLogsDto'
export type { UsageLogsQuery } from './generated/usage/UsageLogsQuery'
export type { HeatmapResponseDto as HeatmapResponse } from './generated/usage/HeatmapResponseDto'

export type { UsageArchiveDiagnostics } from './generated/usage/UsageArchiveDiagnostics'
export type { UsageFreshnessState } from './generated/usage/UsageFreshnessState'
export type { UsageFreshnessProjection } from './generated/usage/UsageFreshnessProjection'
export type { UsageSourceHealthState } from './generated/usage/UsageSourceHealthState'
export type { UsageSourceHealth } from './generated/usage/UsageSourceHealth'
export type { UsageReadinessState } from './generated/usage/UsageReadinessState'
export type { UsageReadinessProjection } from './generated/usage/UsageReadinessProjection'
export type { UsageDrilldownProjection } from './generated/usage/UsageDrilldownProjection'
export type { UsageSnapshotProjection } from './generated/usage/UsageSnapshotProjection'
export type { UsageDashboardResponse } from './generated/usage/UsageDashboardResponse'

export type { UnsupportedReason as UsageUnsupportedReason } from './generated/usage/UnsupportedReason'
export type { FeatureCapability as UsageFeatureCapability } from './generated/usage/FeatureCapability'
export type { CapabilityReport as UsageCapabilityReport } from './generated/usage/CapabilityReport'

export type { HomeOverviewPlatformStats } from './generated/usage/HomeOverviewPlatformStats'
export type { HomeOverviewSeriesItem } from './generated/usage/HomeOverviewSeriesItem'
export type { HomeOverviewSummary } from './generated/usage/HomeOverviewSummary'
export type { HomeOverviewBootstrap } from './generated/usage/HomeOverviewBootstrap'
export type { HomeUsageOverviewResponse } from './generated/usage/HomeUsageOverviewResponse'

export type { UsageImportResultV2 as UsageImportResult } from './generated/usage/UsageImportResultV2'
export type { UsageImportSummary } from './generated/usage/UsageImportSummary'
export type { ImportAllUsageResponse } from './generated/usage/ImportAllUsageResponse'
export type { UsageImportJobStatus } from './generated/usage/UsageImportJobStatus'
export type { UsageImportJobStage } from './generated/usage/UsageImportJobStage'
export type { UsageImportJobSnapshot } from './generated/usage/UsageImportJobSnapshot'
export type { StartUsageImportJobResponse } from './generated/usage/StartUsageImportJobResponse'
export type { SessionIndexJobStatus } from './generated/usage/SessionIndexJobStatus'
export type { SessionIndexJobStage } from './generated/usage/SessionIndexJobStage'
export type { SessionIndexJobSnapshot } from './generated/usage/SessionIndexJobSnapshot'
export type { StartSessionIndexJobResponse } from './generated/usage/StartSessionIndexJobResponse'

// ── 以下为纯视图/事件类型，不在生成范围 ──

/** 首页概览视图模式 */
export type HomeOverviewViewMode = 'sessions' | 'requests' | 'tokens'

/** 平台类型（视图层收窄；wire 上是 string） */
export type UsagePlatform = 'claude' | 'codex' | 'gemini' | 'opencode'

/** Tauri 事件 `usage:snapshot-updated` 的 payload（事件不走命令返回，不在生成范围） */
export interface UsageSnapshotUpdatedPayload {
  reason: string
  platform_scope: string
  job_id?: string | null
  imported_records: number
  generated_at: string
}
