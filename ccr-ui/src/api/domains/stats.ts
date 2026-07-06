/**
 * Stats Domain —— 统计与用量 API
 *
 * 对应后端 commands::usage::* 与 commands::pricing::* 命令。
 * 真迁移自 tauri.ts 第 12 分组。
 *
 * legacy CostTracker 系 stats 命令（get_cost_overview 等 10 条）已随
 * usage-family-absorb 任务下线，统计数据统一走 V2 usage（llmusage 投影）；
 * `domains/usage.ts` 保留作为 `usageApi` 命名空间门面，
 * 从本文件 re-export 子集以维持历史契约。
 */

import { invoke } from '@tauri-apps/api/core'
import { asRecord, type UnknownRecord } from '../_shared'
import type { CapabilityReport } from '../../types/generated/usage/CapabilityReport'
import type { DailyTrendDto } from '../../types/generated/usage/DailyTrendDto'
import type { HeatmapResponseDto } from '../../types/generated/usage/HeatmapResponseDto'
import type { HomeUsageOverviewResponse } from '../../types/generated/usage/HomeUsageOverviewResponse'
import type { ImportAllUsageResponse } from '../../types/generated/usage/ImportAllUsageResponse'
import type { ModelStatDto } from '../../types/generated/usage/ModelStatDto'
import type { PaginatedLogsDto } from '../../types/generated/usage/PaginatedLogsDto'
import type { ProjectStatDto } from '../../types/generated/usage/ProjectStatDto'
import type { ProviderBreakdownDto } from '../../types/generated/usage/ProviderBreakdownDto'
import type { SessionIndexJobSnapshot } from '../../types/generated/usage/SessionIndexJobSnapshot'
import type { StartSessionIndexJobResponse } from '../../types/generated/usage/StartSessionIndexJobResponse'
import type { StartUsageImportJobResponse } from '../../types/generated/usage/StartUsageImportJobResponse'
import type { UsageDashboardResponse } from '../../types/generated/usage/UsageDashboardResponse'
import type { UsageImportJobSnapshot } from '../../types/generated/usage/UsageImportJobSnapshot'
import type { UsageImportResultV2 } from '../../types/generated/usage/UsageImportResultV2'
import type { UsageLogsQuery } from '../../types/generated/usage/UsageLogsQuery'
import type { UsageSummaryDto } from '../../types/generated/usage/UsageSummaryDto'

// ── V2 Usage ──
// 返回类型为 ts-rs 生成的 DTO（src/types/generated/usage/），与 Rust 命令签名同源；
// 该组 wrapper 不再接受调用方泛型注入，shape 漂移在编译期暴露。

/** V2: 查询 llmusage 能力矩阵 */
export const getUsageCapabilitiesV2 = async (): Promise<CapabilityReport> => {
  return invoke('get_usage_capabilities_v2')
}

/** V2: 获取使用量汇总 */
export const getUsageSummaryV2 = async (
  platform?: string,
  startDate?: string,
  endDate?: string
): Promise<UsageSummaryDto> => {
  return invoke('get_usage_summary_v2', { platform, startDate, endDate })
}

/** V2: 获取每日趋势 */
export const getUsageTrendsV2 = async (
  platform?: string,
  startDate?: string,
  endDate?: string
): Promise<DailyTrendDto[]> => {
  return invoke('get_usage_trends_v2', { platform, startDate, endDate })
}

/** V2: 获取模型统计 */
export const getUsageByModelV2 = async (
  platform?: string,
  startDate?: string,
  endDate?: string
): Promise<ModelStatDto[]> => {
  return invoke('get_usage_by_model_v2', { platform, startDate, endDate })
}

/** V2: 获取 provider 统计 */
export const getUsageByProviderV2 = async (
  platform?: string,
  startDate?: string,
  endDate?: string
): Promise<ProviderBreakdownDto[]> => {
  return invoke('get_usage_by_provider_v2', { platform, startDate, endDate })
}

/** V2: 获取项目统计 */
export const getUsageByProjectV2 = async (
  platform?: string,
  startDate?: string,
  endDate?: string
): Promise<ProjectStatDto[]> => {
  return invoke('get_usage_by_project_v2', { platform, startDate, endDate })
}

/** V2: 获取热力图（兼容映射到现有命令） */
export const getUsageHeatmapV2 = async (
  platform?: string,
  days?: number
): Promise<HeatmapResponseDto> => {
  return invoke('get_usage_heatmap_v2', { platform, days })
}

export type { UsageLogsQuery }

/** V2: 获取日志（支持 cursor 与 offset 两种分页模式） */
export const getUsageLogsV2 = async (
  platformOrQuery?: string | UsageLogsQuery,
  page?: number,
  pageSize?: number,
  model?: string,
  cursor?: string,
  includeTotal?: boolean,
  mode?: 'cursor' | 'offset'
): Promise<PaginatedLogsDto> => {
  const query: UsageLogsQuery =
    typeof platformOrQuery === 'object'
      ? platformOrQuery
      : {
          platform: platformOrQuery,
          page,
          page_size: pageSize,
          model,
          cursor,
          include_total: includeTotal,
          mode,
        }
  return invoke('get_usage_logs_v2', { query })
}

/** V2: 获取仪表盘聚合 */
export const getUsageDashboardV2 = async (
  platform?: string,
  startDate?: string,
  endDate?: string,
  heatmapDays?: number,
  includeHeatmap?: boolean,
  provider?: string
): Promise<UsageDashboardResponse> => {
  return invoke('get_usage_dashboard_v2', {
    platform,
    provider,
    startDate,
    endDate,
    heatmapDays,
    includeHeatmap,
  })
}

/** V2: 启动 usage 后台导入任务 */
export const startUsageImportJobV2 = async (
  platform?: string,
  recentDays?: number,
  resetSources?: boolean
): Promise<StartUsageImportJobResponse> => {
  return invoke('start_usage_import_job_v2', { platform, recentDays, resetSources })
}

/** V2: 后台确保 session 索引可用 */
export const ensureSessionIndexV2 = async (): Promise<StartSessionIndexJobResponse> => {
  return invoke('ensure_session_index_v2')
}

/** V2: 查询 session 索引后台任务状态 */
export const getSessionIndexJobStatusV2 = async (
  jobId: string
): Promise<SessionIndexJobSnapshot> => {
  return invoke('get_session_index_job_status_v2', { jobId })
}

/** V2: 查询 usage 后台导入任务状态 */
export const getUsageImportJobStatusV2 = async (jobId: string): Promise<UsageImportJobSnapshot> => {
  return invoke('get_usage_import_job_status_v2', { jobId })
}

/** V2: 取消 usage 后台导入任务 */
export const cancelUsageImportJobV2 = async (jobId: string): Promise<UsageImportJobSnapshot> => {
  return invoke('cancel_usage_import_job_v2', { jobId })
}

/** V2: 导入单平台 usage */
export const importUsageV2 = async (platform: string): Promise<UsageImportResultV2> => {
  return invoke('import_usage_v2', { platform })
}

/** V2: 导入全部 usage */
export const importAllUsageV2 = async (): Promise<ImportAllUsageResponse> => {
  return invoke('import_all_usage_v2')
}

/** V2: 首页工作区概览 */
export const getHomeUsageOverviewV2 = async (days?: number): Promise<HomeUsageOverviewResponse> => {
  return invoke('get_home_usage_overview_v2', { days })
}

// ── 定价 ──

/** 设置定价（兼容 data.model / data.name 两种命名） */
export const setPricing = async <T = UnknownRecord>(data: unknown): Promise<T> => {
  const source = asRecord(data)
  const model = String(source.model ?? source.name ?? '')
  return invoke('set_pricing', { model, pricing: data })
}

/** 获取定价列表 */
export const getPricingList = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('get_pricing_list')
}

/** 移除指定模型定价 */
export const removePricing = async <T = UnknownRecord>(model: string): Promise<T> => {
  return invoke('remove_pricing', { model })
}

/** 重置全部定价 */
export const resetPricing = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('reset_pricing')
}
