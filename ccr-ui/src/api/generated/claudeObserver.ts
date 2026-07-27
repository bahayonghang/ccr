/* Generated from commands/handler_registry.rs; do not edit. */

import { invoke } from '@tauri-apps/api/core'
import type { BreakdownRow } from '@/types/generated/claude_observer/BreakdownRow'
import type { CacheStatsDto } from '@/types/generated/claude_observer/CacheStatsDto'
import type { DailyPoint } from '@/types/generated/claude_observer/DailyPoint'
import type { HeatmapCell } from '@/types/generated/claude_observer/HeatmapCell'
import type { InsightDto } from '@/types/generated/claude_observer/InsightDto'
import type { SessionRow } from '@/types/generated/claude_observer/SessionRow'
import type { SubscriptionDto } from '@/types/generated/claude_observer/SubscriptionDto'
import type { TopToolRow } from '@/types/generated/claude_observer/TopToolRow'

export const claudeObserver = {
  getInsight: (range?: 'today' | 'month' | 'all'): Promise<InsightDto> => invoke('claude_observer_get_insight', { range }),
  dailyTrend: (days?: number): Promise<DailyPoint[]> => invoke('claude_observer_daily_trend', { days }),
  costBreakdown: (dim: 'project' | 'model', days?: number, limit?: number): Promise<BreakdownRow[]> => invoke('claude_observer_cost_breakdown', { dim, days, limit }),
  cacheStats: (): Promise<CacheStatsDto> => invoke('claude_observer_cache_stats'),
  topSessions: (limit?: number, by?: 'cost' | 'calls'): Promise<SessionRow[]> => invoke('claude_observer_top_sessions', { limit, by }),
  toolHeatmap: (days?: number): Promise<HeatmapCell[]> => invoke('claude_observer_tool_heatmap', { days }),
  topTools: (days?: number, limit?: number): Promise<TopToolRow[]> => invoke('claude_observer_top_tools', { days, limit }),
  subscriptionGet: (): Promise<SubscriptionDto> => invoke('claude_observer_subscription_get'),
  subscriptionSet: (mode: string, plan: string, monthlyUsd: number): Promise<SubscriptionDto> =>
    invoke('claude_observer_subscription_set', { mode, plan, monthlyUsd }),
}
