/**
 * Claude Observer domain API
 *
 * 对应 ccr-ui/src-tauri/src/commands/claude_observer.rs 的 9 个命令。
 * 数据源：llmusage（token/cost 维度）+ ccr-db `claude_tool_calls`（工具调用维度）。
 * 返回类型全部来自 ts-rs 生成绑定（src/types/generated/claude_observer/），
 * 契约见 .trellis/spec/ccr/backend/typed-ipc-bindings.md。
 */

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
  /** 一次性拉首屏 Hero 三卡 + 订阅 banner 数据 */
  getInsight: async (range?: 'today' | 'month' | 'all'): Promise<InsightDto> => {
    return invoke('claude_observer_get_insight', { range })
  },

  /** 最近 N 天每日趋势（claude 平台过滤） */
  dailyTrend: async (days?: number): Promise<DailyPoint[]> => {
    return invoke('claude_observer_daily_trend', { days })
  },

  /** 按 project 或 model 维度 Top N 拆分 */
  costBreakdown: async (
    dim: 'project' | 'model',
    days?: number,
    limit?: number
  ): Promise<BreakdownRow[]> => {
    return invoke('claude_observer_cost_breakdown', { dim, days, limit })
  },

  /** 缓存效率：命中率 + 4 个 token 总量 */
  cacheStats: async (): Promise<CacheStatsDto> => {
    return invoke('claude_observer_cache_stats')
  },

  /** Top sessions（来自 claude_tool_calls，by ∈ cost | calls） */
  topSessions: async (limit?: number, by?: 'cost' | 'calls'): Promise<SessionRow[]> => {
    return invoke('claude_observer_top_sessions', { limit, by })
  },

  /** 周×小时工具调用热力图 */
  toolHeatmap: async (days?: number): Promise<HeatmapCell[]> => {
    return invoke('claude_observer_tool_heatmap', { days })
  },

  /** Top tools 排行（按调用次数） */
  topTools: async (days?: number, limit?: number): Promise<TopToolRow[]> => {
    return invoke('claude_observer_top_tools', { days, limit })
  },

  /** 读取订阅设置 */
  subscriptionGet: async (): Promise<SubscriptionDto> => {
    return invoke('claude_observer_subscription_get')
  },

  /** 写入订阅设置 */
  subscriptionSet: async (
    mode: string,
    plan: string,
    monthlyUsd: number
  ): Promise<SubscriptionDto> => {
    return invoke('claude_observer_subscription_set', {
      mode,
      plan,
      monthlyUsd,
    })
  },
}
