import { create } from 'zustand'
import type { UsagePlatform } from '@/types/usage'

// usage 视图偏好 store（08-22-state-logic-port 批次 4）。
// 原 Pinia `stores/usage.ts`（991 行）的数据切片已迁 Query（本目录 queries.ts）；
// 此处只承载父任务 design.md §5 的 `usage` 缓存路由视图态：时间范围 + 平台维度
// （切回路由时筛选条件从 store 恢复）。logs 翻页态随 Query key 承载，不入 store。

export interface UsageTimeRange {
  start?: string
  end?: string
}

interface UsageViewState {
  platform: UsagePlatform | undefined
  timeRange: UsageTimeRange
  setPlatform: (platform: UsagePlatform | undefined) => void
  setTimeRange: (range: UsageTimeRange) => void
  resetFilters: () => void
}

export const useUsageViewStore = create<UsageViewState>()((set) => ({
  platform: undefined,
  timeRange: {},

  setPlatform: (platform) => set({ platform }),
  setTimeRange: (timeRange) => set({ timeRange }),
  resetFilters: () => set({ platform: undefined, timeRange: {} }),
}))
