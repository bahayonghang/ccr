/* Generated from commands/handler_registry.rs; do not edit. */

import { invoke } from '@/api/invokeRuntime'
import type { CapabilityReport } from '@/types/generated/usage/CapabilityReport'
import type { DailyTrendDto } from '@/types/generated/usage/DailyTrendDto'
import type { HeatmapResponseDto } from '@/types/generated/usage/HeatmapResponseDto'
import type { HomeUsageOverviewResponse } from '@/types/generated/usage/HomeUsageOverviewResponse'
import type { ImportAllUsageResponse } from '@/types/generated/usage/ImportAllUsageResponse'
import type { ModelStatDto } from '@/types/generated/usage/ModelStatDto'
import type { PaginatedLogsDto } from '@/types/generated/usage/PaginatedLogsDto'
import type { ProjectStatDto } from '@/types/generated/usage/ProjectStatDto'
import type { ProviderBreakdownDto } from '@/types/generated/usage/ProviderBreakdownDto'
import type { SessionIndexJobSnapshot } from '@/types/generated/usage/SessionIndexJobSnapshot'
import type { StartSessionIndexJobResponse } from '@/types/generated/usage/StartSessionIndexJobResponse'
import type { StartUsageImportJobResponse } from '@/types/generated/usage/StartUsageImportJobResponse'
import type { UsageDashboardResponse } from '@/types/generated/usage/UsageDashboardResponse'
import type { UsageImportJobSnapshot } from '@/types/generated/usage/UsageImportJobSnapshot'
import type { UsageImportResultV2 } from '@/types/generated/usage/UsageImportResultV2'
import type { UsageLogsQuery } from '@/types/generated/usage/UsageLogsQuery'
import type { UsageSummaryDto } from '@/types/generated/usage/UsageSummaryDto'

export type { UsageLogsQuery }
export type UsageRangeInput = { platform?: string; startDate?: string; endDate?: string }
export type UsageDashboardInput = UsageRangeInput & { heatmapDays?: number; includeHeatmap?: boolean; provider?: string }

export const getUsageSummaryV2 = (platform?: string, startDate?: string, endDate?: string): Promise<UsageSummaryDto> =>
  invoke('get_usage_summary_v2', { platform, startDate, endDate })
export const getUsageCapabilitiesV2 = (): Promise<CapabilityReport> => invoke('get_usage_capabilities_v2')
export const getUsageTrendsV2 = (platform?: string, startDate?: string, endDate?: string): Promise<DailyTrendDto[]> =>
  invoke('get_usage_trends_v2', { platform, startDate, endDate })
export const getUsageByModelV2 = (platform?: string, startDate?: string, endDate?: string): Promise<ModelStatDto[]> =>
  invoke('get_usage_by_model_v2', { platform, startDate, endDate })
export const getUsageByProviderV2 = (platform?: string, startDate?: string, endDate?: string): Promise<ProviderBreakdownDto[]> =>
  invoke('get_usage_by_provider_v2', { platform, startDate, endDate })
export const getUsageByProjectV2 = (platform?: string, startDate?: string, endDate?: string): Promise<ProjectStatDto[]> =>
  invoke('get_usage_by_project_v2', { platform, startDate, endDate })
export const getUsageHeatmapV2 = (platform?: string, days?: number): Promise<HeatmapResponseDto> =>
  invoke('get_usage_heatmap_v2', { platform, days })
export const getUsageLogsV2 = (platformOrQuery?: string | UsageLogsQuery, page?: number, pageSize?: number, model?: string, cursor?: string, includeTotal?: boolean, mode?: 'cursor' | 'offset'): Promise<PaginatedLogsDto> => {
  const query: UsageLogsQuery = typeof platformOrQuery === 'object'
    ? platformOrQuery
    : { platform: platformOrQuery, page, page_size: pageSize, model, cursor, include_total: includeTotal, mode }
  return invoke('get_usage_logs_v2', { query })
}
export const getUsageDashboardV2 = (platform?: string, startDate?: string, endDate?: string, heatmapDays?: number, includeHeatmap?: boolean, provider?: string): Promise<UsageDashboardResponse> =>
  invoke('get_usage_dashboard_v2', { platform, provider, startDate, endDate, heatmapDays, includeHeatmap })
export const getHomeUsageOverviewV2 = (days?: number): Promise<HomeUsageOverviewResponse> => invoke('get_home_usage_overview_v2', { days })
export const ensureSessionIndexV2 = (): Promise<StartSessionIndexJobResponse> => invoke('ensure_session_index_v2')
export const getSessionIndexJobStatusV2 = (jobId: string): Promise<SessionIndexJobSnapshot> => invoke('get_session_index_job_status_v2', { jobId })
export const startUsageImportJobV2 = (platform?: string, recentDays?: number, resetSources?: boolean): Promise<StartUsageImportJobResponse> =>
  invoke('start_usage_import_job_v2', { platform, recentDays, resetSources })
export const getUsageImportJobStatusV2 = (jobId: string): Promise<UsageImportJobSnapshot> => invoke('get_usage_import_job_status_v2', { jobId })
export const cancelUsageImportJobV2 = (jobId: string): Promise<UsageImportJobSnapshot> => invoke('cancel_usage_import_job_v2', { jobId })
export const importUsageV2 = (platform: string): Promise<UsageImportResultV2> => invoke('import_usage_v2', { platform })
export const importAllUsageV2 = (): Promise<ImportAllUsageResponse> => invoke('import_all_usage_v2')
