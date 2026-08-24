import { QueryClient } from '@tanstack/react-query'

/** Unused query results stay this long after the last observer unmounts. */
export const QUERY_GC_TIME_MS = 120_000

/**
 * QueryClient 模块级单例。
 * 默认选项为阶段 1 保守值；新鲜度策略随 08-22-state-logic-port 落地调整。
 * gcTime 必须有限，否则跨路由 soak 会把 heatmap / monitoring 等缓存一直留在堆上。
 */
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      staleTime: 30_000,
      gcTime: QUERY_GC_TIME_MS,
      refetchOnWindowFocus: false,
    },
  },
})

if (typeof window !== 'undefined') {
  Object.defineProperty(window, '__CCR_SOAK_STATS', {
    configurable: true,
    value: () => {
      const queries = queryClient.getQueryCache().getAll()
      return {
        queries: queries.length,
        active: queries.filter((query) => query.getObserversCount() > 0).length,
        mutations: queryClient.getMutationCache().getAll().length,
        historyLength: window.history.length,
      }
    },
  })
}
