import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { claudeObserver } from '@/api'

// claude 域 Query 层（08-22-state-logic-port 批次 2）。
// 原 `stores/claudeObserver.ts` 的 9 个数据切片全部来自 claude_observer_* IPC，
// 由 `claude_observer:updated` 事件驱动 refetch（原 store 语义）→ 事件桥接层
// 失效（shell/eventBridge.ts，批次 3）。订阅/面板 UI 态入 Zustand（批次 4）。
// staleTime 30s 为兜底新鲜度；主新鲜度来源是事件失效。

export const claudeObserverKeys = {
  all: ['claude-observer'] as const,
  insight: (range?: 'today' | 'month' | 'all') =>
    [...claudeObserverKeys.all, 'insight', range ?? null] as const,
  dailyTrend: (days?: number) => [...claudeObserverKeys.all, 'daily-trend', days ?? null] as const,
  costBreakdown: (dim: 'project' | 'model', days?: number, limit?: number) =>
    [...claudeObserverKeys.all, 'cost-breakdown', dim, days ?? null, limit ?? null] as const,
  cacheStats: () => [...claudeObserverKeys.all, 'cache-stats'] as const,
  topSessions: (limit?: number, by?: 'cost' | 'calls') =>
    [...claudeObserverKeys.all, 'top-sessions', limit ?? null, by ?? null] as const,
  toolHeatmap: (days?: number) =>
    [...claudeObserverKeys.all, 'tool-heatmap', days ?? null] as const,
  topTools: (days?: number, limit?: number) =>
    [...claudeObserverKeys.all, 'top-tools', days ?? null, limit ?? null] as const,
  subscription: () => [...claudeObserverKeys.all, 'subscription'] as const,
}

const OBSERVER_STALE_TIME = 30_000

export function useClaudeObserverInsight(range?: 'today' | 'month' | 'all') {
  return useQuery({
    queryKey: claudeObserverKeys.insight(range),
    queryFn: () => claudeObserver.getInsight(range),
    staleTime: OBSERVER_STALE_TIME,
  })
}

export function useClaudeObserverDailyTrend(days?: number) {
  return useQuery({
    queryKey: claudeObserverKeys.dailyTrend(days),
    queryFn: () => claudeObserver.dailyTrend(days),
    staleTime: OBSERVER_STALE_TIME,
  })
}

export function useClaudeObserverCostBreakdown(dim: 'project' | 'model', days?: number, limit?: number) {
  return useQuery({
    queryKey: claudeObserverKeys.costBreakdown(dim, days, limit),
    queryFn: () => claudeObserver.costBreakdown(dim, days, limit),
    staleTime: OBSERVER_STALE_TIME,
  })
}

export function useClaudeObserverCacheStats() {
  return useQuery({
    queryKey: claudeObserverKeys.cacheStats(),
    queryFn: () => claudeObserver.cacheStats(),
    staleTime: OBSERVER_STALE_TIME,
  })
}

export function useClaudeObserverTopSessions(limit?: number, by?: 'cost' | 'calls') {
  return useQuery({
    queryKey: claudeObserverKeys.topSessions(limit, by),
    queryFn: () => claudeObserver.topSessions(limit, by),
    staleTime: OBSERVER_STALE_TIME,
  })
}

export function useClaudeObserverToolHeatmap(days?: number) {
  return useQuery({
    queryKey: claudeObserverKeys.toolHeatmap(days),
    queryFn: () => claudeObserver.toolHeatmap(days),
    staleTime: OBSERVER_STALE_TIME,
  })
}

export function useClaudeObserverTopTools(days?: number, limit?: number) {
  return useQuery({
    queryKey: claudeObserverKeys.topTools(days, limit),
    queryFn: () => claudeObserver.topTools(days, limit),
    staleTime: OBSERVER_STALE_TIME,
  })
}

export function useClaudeObserverSubscription() {
  return useQuery({
    queryKey: claudeObserverKeys.subscription(),
    queryFn: () => claudeObserver.subscriptionGet(),
    staleTime: OBSERVER_STALE_TIME,
  })
}

/** 订阅计划写入：成功后失效 subscription 切片。 */
export function useSetClaudeObserverSubscription() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (input: { mode: string; plan: string; monthlyUsd: number }) =>
      claudeObserver.subscriptionSet(input.mode, input.plan, input.monthlyUsd),
    onSuccess: () =>
      queryClient.invalidateQueries({ queryKey: claudeObserverKeys.subscription() }),
  })
}
