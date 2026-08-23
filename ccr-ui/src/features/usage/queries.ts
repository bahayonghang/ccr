import { useQuery, useQueryClient } from '@tanstack/react-query'
import {
  getHomeUsageOverviewV2,
  getUsageByModelV2,
  getUsageByProjectV2,
  getUsageByProviderV2,
  getUsageCapabilitiesV2,
  getUsageDashboardV2,
  getUsageHeatmapV2,
  getUsageImportJobStatusV2,
  getUsageLogsV2,
  getUsageSummaryV2,
  getUsageTrendsV2,
} from '@/api'
import type { UsageLogsQuery } from '@/api/generated/usageV2'

// usage 域 Query 层（08-22-state-logic-port 批次 2）。
// 原 Pinia `stores/usage.ts`（991 行）的数据切片按 state-disposition.md 迁到
// TanStack Query；视图偏好（platform/timeRange/logsPage/logsModelFilter）入
// Zustand（见同目录 stores.ts，批次 4）。staleTime 沿用原 30s TTL 语义；
// capabilities 变更频率低，取 5min。事件失效见 shell/eventBridge.ts。

export const usageKeys = {
  all: ['usage'] as const,
  capabilities: () => [...usageKeys.all, 'capabilities'] as const,
  dashboard: (platform?: string, startDate?: string, endDate?: string) =>
    [...usageKeys.all, 'dashboard', platform ?? null, startDate ?? null, endDate ?? null] as const,
  summary: (platform?: string, startDate?: string, endDate?: string) =>
    [...usageKeys.all, 'summary', platform ?? null, startDate ?? null, endDate ?? null] as const,
  trends: (platform?: string, startDate?: string, endDate?: string) =>
    [...usageKeys.all, 'trends', platform ?? null, startDate ?? null, endDate ?? null] as const,
  byModel: (platform?: string, startDate?: string, endDate?: string) =>
    [...usageKeys.all, 'by-model', platform ?? null, startDate ?? null, endDate ?? null] as const,
  byProject: (platform?: string, startDate?: string, endDate?: string) =>
    [...usageKeys.all, 'by-project', platform ?? null, startDate ?? null, endDate ?? null] as const,
  byProvider: (platform?: string, startDate?: string, endDate?: string) =>
    [...usageKeys.all, 'by-provider', platform ?? null, startDate ?? null, endDate ?? null] as const,
  heatmap: (platform?: string, days?: number) =>
    [...usageKeys.all, 'heatmap', platform ?? null, days ?? null] as const,
  logs: (query: UsageLogsQueryKey) =>
    [...usageKeys.all, 'logs', query.platform ?? null, query.page ?? 1, query.model ?? null, query.cursor ?? null] as const,
  importJob: (jobId: string) => [...usageKeys.all, 'import-job', jobId] as const,
}

export const homeUsageKeys = {
  all: ['home-usage'] as const,
  overview: (days?: number) => [...homeUsageKeys.all, 'overview', days ?? null] as const,
}

export interface UsageLogsQueryKey {
  platform?: string
  page?: number
  model?: string
  cursor?: string
}

/** staleTime 取值记录（批次 2）：数据切片 30s（原 TTL）；capabilities 5min。 */
const USAGE_STALE_TIME = 30_000
const CAPABILITIES_STALE_TIME = 300_000

export function useUsageCapabilities() {
  return useQuery({
    queryKey: usageKeys.capabilities(),
    queryFn: () => getUsageCapabilitiesV2(),
    staleTime: CAPABILITIES_STALE_TIME,
  })
}

export function useUsageDashboard(platform?: string, startDate?: string, endDate?: string) {
  return useQuery({
    queryKey: usageKeys.dashboard(platform, startDate, endDate),
    queryFn: () => getUsageDashboardV2(platform, startDate, endDate),
    staleTime: USAGE_STALE_TIME,
  })
}

export function useUsageSummary(platform?: string, startDate?: string, endDate?: string) {
  return useQuery({
    queryKey: usageKeys.summary(platform, startDate, endDate),
    queryFn: () => getUsageSummaryV2(platform, startDate, endDate),
    staleTime: USAGE_STALE_TIME,
  })
}

export function useUsageTrends(platform?: string, startDate?: string, endDate?: string) {
  return useQuery({
    queryKey: usageKeys.trends(platform, startDate, endDate),
    queryFn: () => getUsageTrendsV2(platform, startDate, endDate),
    staleTime: USAGE_STALE_TIME,
  })
}

export function useUsageByModel(platform?: string, startDate?: string, endDate?: string) {
  return useQuery({
    queryKey: usageKeys.byModel(platform, startDate, endDate),
    queryFn: () => getUsageByModelV2(platform, startDate, endDate),
    staleTime: USAGE_STALE_TIME,
  })
}

export function useUsageByProject(platform?: string, startDate?: string, endDate?: string) {
  return useQuery({
    queryKey: usageKeys.byProject(platform, startDate, endDate),
    queryFn: () => getUsageByProjectV2(platform, startDate, endDate),
    staleTime: USAGE_STALE_TIME,
  })
}

export function useUsageByProvider(platform?: string, startDate?: string, endDate?: string) {
  return useQuery({
    queryKey: usageKeys.byProvider(platform, startDate, endDate),
    queryFn: () => getUsageByProviderV2(platform, startDate, endDate),
    staleTime: USAGE_STALE_TIME,
  })
}

export function useUsageHeatmap(platform?: string, days?: number) {
  return useQuery({
    queryKey: usageKeys.heatmap(platform, days),
    queryFn: () => getUsageHeatmapV2(platform, days),
    staleTime: USAGE_STALE_TIME,
  })
}

export function useUsageLogs(query: UsageLogsQuery) {
  return useQuery({
    queryKey: usageKeys.logs({
      platform: query.platform,
      page: query.page,
      model: query.model,
      cursor: query.cursor,
    }),
    queryFn: () => getUsageLogsV2(query),
    staleTime: USAGE_STALE_TIME,
    placeholderData: (previous) => previous,
  })
}

/** 导入任务状态轮询：jobId 存在时启用，2s 间隔（原 store 轮询节奏）。 */
export function useUsageImportJob(jobId: string | null) {
  return useQuery({
    queryKey: usageKeys.importJob(jobId ?? 'none'),
    queryFn: () => getUsageImportJobStatusV2(jobId as string),
    enabled: jobId !== null,
    refetchInterval: 2_000,
  })
}

export function useHomeUsageOverview(days?: number) {
  return useQuery({
    queryKey: homeUsageKeys.overview(days),
    queryFn: () => getHomeUsageOverviewV2(days),
    staleTime: USAGE_STALE_TIME,
  })
}

/** 触发用量数据失效（事件桥接与导入完成后的通用出口）。 */
export function useInvalidateUsage() {
  const queryClient = useQueryClient()
  return () => queryClient.invalidateQueries({ queryKey: usageKeys.all })
}
