/**
 * Stats Domain —— 统计与用量 API
 *
 * 对应后端 commands::stats::* 与 commands::usage::* 命令。
 * 真迁移自 tauri.ts 第 12 分组（含 V2 usage 子节与 Stats 扩展）。
 *
 * V2 usage 系列与 Stats 系列命令在业务上都属于"数据/统计"域，
 * 这里合并为一个 domain；`domains/usage.ts` 保留作为 `usageApi` 命名空间门面，
 * 从本文件 re-export 子集以维持历史契约。
 */

import { invoke } from '@tauri-apps/api/core'
import { asRecord, type UnknownRecord } from '../_shared'

// ── 费用与热力图 ──

/** 获取费用概览 */
export const getCostOverview = async <T = UnknownRecord>(period?: string): Promise<T> => {
  return invoke('get_cost_overview', { period })
}

/** 获取热力图数据 */
export const getHeatmapData = async <T = UnknownRecord>(
  platform?: string,
  days?: number,
): Promise<T> => {
  return invoke('get_heatmap_data', { platform, days })
}

// ── V2 Usage ──

/** V2: 获取使用量汇总 */
export const getUsageCapabilitiesV2 = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('get_usage_capabilities_v2')
}

/** V2: 获取使用量汇总 */
export const getUsageSummaryV2 = async <T = UnknownRecord>(
  platform?: string,
  startDate?: string,
  endDate?: string,
): Promise<T> => {
  return invoke('get_usage_summary_v2', { platform, startDate, endDate })
}

/** V2: 获取每日趋势 */
export const getUsageTrendsV2 = async <T = UnknownRecord>(
  platform?: string,
  startDate?: string,
  endDate?: string,
): Promise<T> => {
  return invoke('get_usage_trends_v2', { platform, startDate, endDate })
}

/** V2: 获取模型统计 */
export const getUsageByModelV2 = async <T = UnknownRecord>(
  platform?: string,
  startDate?: string,
  endDate?: string,
): Promise<T> => {
  return invoke('get_usage_by_model_v2', { platform, startDate, endDate })
}

/** V2: 获取项目统计 */
export const getUsageByProjectV2 = async <T = UnknownRecord>(
  platform?: string,
  startDate?: string,
  endDate?: string,
): Promise<T> => {
  return invoke('get_usage_by_project_v2', { platform, startDate, endDate })
}

/** V2: 获取热力图（兼容映射到现有命令） */
export const getUsageHeatmapV2 = async <T = UnknownRecord>(
  platform?: string,
  days?: number,
): Promise<T> => {
  return invoke('get_usage_heatmap_v2', { platform, days })
}

/** V2 getUsageLogsV2 查询参数 */
export interface UsageLogsQuery {
  platform?: string
  model?: string
  start_date?: string
  end_date?: string
  page?: number
  page_size?: number
  cursor?: string
  include_total?: boolean
  mode?: 'cursor' | 'offset'
}

/** V2: 获取日志（支持 cursor 与 offset 两种分页模式） */
export const getUsageLogsV2 = async <T = UnknownRecord>(
  platformOrQuery?: string | UsageLogsQuery,
  page?: number,
  pageSize?: number,
  model?: string,
  cursor?: string,
  includeTotal?: boolean,
  mode?: 'cursor' | 'offset',
): Promise<T> => {
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
export const getUsageDashboardV2 = async <T = UnknownRecord>(
  platform?: string,
  startDate?: string,
  endDate?: string,
  heatmapDays?: number,
  includeHeatmap?: boolean,
): Promise<T> => {
  return invoke('get_usage_dashboard_v2', {
    platform,
    startDate,
    endDate,
    heatmapDays,
    includeHeatmap,
  })
}

/** V2: 启动 usage 后台导入任务 */
export const startUsageImportJobV2 = async <T = UnknownRecord>(
  platform?: string,
  recentDays?: number,
  resetSources?: boolean,
): Promise<T> => {
  return invoke('start_usage_import_job_v2', { platform, recentDays, resetSources })
}

/** V2: 后台确保 session 索引可用 */
export const ensureSessionIndexV2 = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('ensure_session_index_v2')
}

/** V2: 查询 session 索引后台任务状态 */
export const getSessionIndexJobStatusV2 = async <T = UnknownRecord>(jobId: string): Promise<T> => {
  return invoke('get_session_index_job_status_v2', { jobId })
}

/** V2: 查询 usage 后台导入任务状态 */
export const getUsageImportJobStatusV2 = async <T = UnknownRecord>(jobId: string): Promise<T> => {
  return invoke('get_usage_import_job_status_v2', { jobId })
}

/** V2: 取消 usage 后台导入任务 */
export const cancelUsageImportJobV2 = async <T = UnknownRecord>(jobId: string): Promise<T> => {
  return invoke('cancel_usage_import_job_v2', { jobId })
}

/** V2: 导入单平台 usage */
export const importUsageV2 = async <T = UnknownRecord>(platform: string): Promise<T> => {
  return invoke('import_usage_v2', { platform })
}

/** V2: 导入全部 usage */
export const importAllUsageV2 = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('import_all_usage_v2')
}

/** V2: 首页工作区概览 */
export const getHomeUsageOverviewV2 = async <T = UnknownRecord>(days?: number): Promise<T> => {
  return invoke('get_home_usage_overview_v2', { days })
}

/** 获取会话统计 */
export const getSessionStats = async <T = UnknownRecord>(platform?: string): Promise<T> => {
  return invoke('get_session_stats', { platform })
}

// ── Stats 扩展 ──

/** 获取费用趋势 */
export const getCostTrend = async <T = UnknownRecord>(period?: string): Promise<T> => {
  return invoke('get_cost_trend', { period })
}

/** 按模型统计费用（period 参数保留向后兼容，后端已忽略） */
export const getCostByModel = async <T = UnknownRecord>(_period?: string): Promise<T> => {
  return invoke('get_cost_by_model')
}

/** 按项目统计费用（period 参数保留向后兼容） */
export const getCostByProject = async <T = UnknownRecord>(_period?: string): Promise<T> => {
  return invoke('get_cost_by_project')
}

/** 获取提供商使用量 */
export const getProviderUsage = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('get_provider_usage')
}

/** 获取 Top Sessions */
export const getTopSessions = async <T = UnknownRecord>(limit?: number): Promise<T> => {
  return invoke('get_top_sessions', { limit })
}

/** 获取统计摘要 */
export const getStatsSummary = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('get_stats_summary')
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

/** 获取每日统计 */
export const getDailyStats = async <T = UnknownRecord>(days?: number): Promise<T> => {
  return invoke('get_daily_stats', { days })
}
