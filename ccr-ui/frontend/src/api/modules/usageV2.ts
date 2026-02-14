/**
 * Usage V2 API Module
 *
 * 对接后端 v2 聚合 API，返回预聚合数据
 */

import { api } from '../core'
import type {
  UsageSummary,
  DailyTrend,
  ModelStat,
  ProjectStat,
  PaginatedLogs,
  HeatmapResponse,
  ImportResult,
} from '@/types/usage'

// 后端 ApiSuccess 包装格式
interface ApiSuccess<T> {
  success: boolean
  data: T
}

const unwrap = <T>(res: { data: ApiSuccess<T> }): T => res.data.data

export const getUsageSummaryV2 = async (
  platform?: string, start?: string, end?: string
): Promise<UsageSummary> => {
  const params = new URLSearchParams()
  if (platform) params.set('platform', platform)
  if (start) params.set('start', start)
  if (end) params.set('end', end)
  return unwrap(await api.get(`/v2/usage/summary?${params}`))
}

export const getUsageTrendsV2 = async (
  platform?: string, start?: string, end?: string
): Promise<DailyTrend[]> => {
  const params = new URLSearchParams()
  if (platform) params.set('platform', platform)
  if (start) params.set('start', start)
  if (end) params.set('end', end)
  return unwrap(await api.get(`/v2/usage/trends?${params}`))
}

export const getUsageByModelV2 = async (
  platform?: string, start?: string, end?: string
): Promise<ModelStat[]> => {
  const params = new URLSearchParams()
  if (platform) params.set('platform', platform)
  if (start) params.set('start', start)
  if (end) params.set('end', end)
  return unwrap(await api.get(`/v2/usage/by-model?${params}`))
}

export const getUsageByProjectV2 = async (
  platform?: string, start?: string, end?: string
): Promise<ProjectStat[]> => {
  const params = new URLSearchParams()
  if (platform) params.set('platform', platform)
  if (start) params.set('start', start)
  if (end) params.set('end', end)
  return unwrap(await api.get(`/v2/usage/by-project?${params}`))
}

export const getUsageHeatmapV2 = async (
  platform?: string, days: number = 365
): Promise<HeatmapResponse> => {
  const params = new URLSearchParams()
  if (platform) params.set('platform', platform)
  params.set('days', days.toString())
  return unwrap(await api.get(`/v2/usage/heatmap?${params}`))
}

export const getUsageLogsV2 = async (
  platform?: string, page: number = 1, pageSize: number = 50, model?: string
): Promise<PaginatedLogs> => {
  const params = new URLSearchParams()
  if (platform) params.set('platform', platform)
  params.set('page', page.toString())
  params.set('page_size', pageSize.toString())
  if (model) params.set('model', model)
  return unwrap(await api.get(`/v2/usage/logs?${params}`))
}

export const importUsageV2 = async (platform: string): Promise<ImportResult> => {
  return unwrap(await api.post(`/v2/usage/import?platform=${platform}`))
}

export const importAllUsageV2 = async (): Promise<ImportResult[]> => {
  const results: ImportResult[] = []
  for (const p of ['claude', 'codex', 'gemini']) {
    results.push(await importUsageV2(p))
  }
  return results
}
