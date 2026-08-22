import { QueryClient } from '@tanstack/react-query'

/**
 * QueryClient 模块级单例。
 * 默认选项为阶段 1 保守值；新鲜度策略随 08-22-state-logic-port 落地调整。
 */
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      staleTime: 30_000,
      refetchOnWindowFocus: false,
    },
  },
})
