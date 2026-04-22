/**
 * Usage Domain —— usageApi 命名空间门面
 *
 * V2 usage 与 stats 命令实现已合并到 `./stats`（对应 tauri.ts 第 12 分组）。
 * 此文件保留 `usageApi` 命名空间的对外契约子集，业务代码可继续按
 * `import { usageApi } from '@/api'` 使用。
 */

export {
  getUsageSummaryV2,
  getUsageTrendsV2,
  getUsageByModelV2,
  getUsageByProjectV2,
  getUsageLogsV2,
  getUsageDashboardV2,
  getHomeUsageOverviewV2,
  ensureSessionIndexV2,
  getSessionIndexJobStatusV2,
  startUsageImportJobV2,
  getUsageImportJobStatusV2,
  importUsageV2,
  importAllUsageV2,
  getCostOverview,
  getHeatmapData,
  getSessionStats,
  getCostTrend,
  getCostByModel,
  getCostByProject,
  getProviderUsage,
  getTopSessions,
  getStatsSummary,
  getDailyStats,
  type UsageLogsQuery,
} from './stats'
