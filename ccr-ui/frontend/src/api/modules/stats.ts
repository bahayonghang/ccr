/**
 * Statistics API Module
 * 
 * 包含成本统计、使用统计、预算管理、定价管理等 API
 */

import { api } from '../core'
import type {
    CostStats,
    DailyCost,
    TopSession,
    StatsSummary,
    UsageRecordsResponse,
    BudgetStatus,
    SetBudgetRequest,
    PricingListResponse,
    SetPricingRequest,
    DailyStatsResponse,
} from '@/types'

// ═══════════════════════════════════════════════════════════
// 📊 成本统计 API (Cost Statistics)
// ═══════════════════════════════════════════════════════════

export const getCostOverview = async (range: string = 'today'): Promise<CostStats> => {
    const response = await api.get<CostStats>(`/stats/cost?range=${range}`)
    return response.data
}

export const getCostToday = async (): Promise<CostStats> => {
    const response = await api.get<CostStats>('/stats/cost/today')
    return response.data
}

export const getCostWeek = async (): Promise<CostStats> => {
    const response = await api.get<CostStats>('/stats/cost/week')
    return response.data
}

export const getCostMonth = async (): Promise<CostStats> => {
    const response = await api.get<CostStats>('/stats/cost/month')
    return response.data
}

export const getCostTrend = async (range: string = 'month'): Promise<DailyCost[]> => {
    const response = await api.get<DailyCost[]>(`/stats/cost/trend?range=${range}`)
    return response.data
}

export const getCostByModel = async (range: string = 'month'): Promise<Record<string, number>> => {
    const response = await api.get<Record<string, number>>(`/stats/cost/by-model?range=${range}`)
    return response.data
}

export const getCostByProject = async (range: string = 'month'): Promise<Record<string, number>> => {
    const response = await api.get<Record<string, number>>(`/stats/cost/by-project?range=${range}`)
    return response.data
}

export const getProviderUsage = async (): Promise<Record<string, number>> => {
    const response = await api.get<Record<string, number>>('/stats/provider-usage')
    return response.data
}

export const getTopSessions = async (limit: number = 10): Promise<TopSession[]> => {
    const response = await api.get<TopSession[]>(`/stats/cost/top-sessions?limit=${limit}`)
    return response.data
}

export const getStatsSummary = async (): Promise<StatsSummary> => {
    const response = await api.get<StatsSummary>('/stats/summary')
    return response.data
}

// 热力图数据响应类型
export interface HeatmapData {
    data: Record<string, number>
    max_value: number
    total_tokens: number
    active_days: number
}

export const getHeatmapData = async (): Promise<HeatmapData> => {
    const response = await api.get<HeatmapData>('/stats/heatmap')
    return response.data
}

// ═══════════════════════════════════════════════════════════
// 💰 预算管理 API (Budget Management)
// ═══════════════════════════════════════════════════════════

export const getBudgetStatus = async (): Promise<BudgetStatus> => {
    const response = await api.get<BudgetStatus>('/budget/status')
    return response.data
}

export const setBudget = async (request: SetBudgetRequest): Promise<void> => {
    await api.post('/budget/set', request)
}

export const resetBudget = async (): Promise<void> => {
    await api.post('/budget/reset')
}

// ═══════════════════════════════════════════════════════════
// 💲 定价管理 API (Pricing Management)
// ═══════════════════════════════════════════════════════════

export const getPricingList = async (): Promise<PricingListResponse> => {
    const response = await api.get<PricingListResponse>('/pricing/list')
    return response.data
}

export const setPricing = async (request: SetPricingRequest): Promise<void> => {
    await api.post('/pricing/set', request)
}

export const removePricing = async (model: string): Promise<void> => {
    await api.delete(`/pricing/remove/${encodeURIComponent(model)}`)
}

export const resetPricing = async (): Promise<void> => {
    await api.post('/pricing/reset')
}

// ═══════════════════════════════════════════════════════════
// 📈 使用分析 API (Usage Analytics)
// ═══════════════════════════════════════════════════════════

export const getUsageRecords = async (
    platform: string = 'claude',
    limit: number = 10000
): Promise<UsageRecordsResponse> => {
    const params = new URLSearchParams()
    params.set('platform', platform)
    params.set('limit', limit.toString())
    const response = await api.get<UsageRecordsResponse>(`/usage/records?${params}`)
    return response.data
}

/**
 * 获取每日使用统计 - 支持 CodMate 风格的三视图切换
 * @param days 查询天数，默认 30
 */
export const getDailyStats = async (days: number = 30): Promise<DailyStatsResponse> => {
    const response = await api.get<DailyStatsResponse>(`/sessions/stats/daily?days=${days}`)
    return response.data
}
