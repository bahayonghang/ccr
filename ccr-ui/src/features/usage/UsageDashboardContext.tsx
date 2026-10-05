import { createContext, useContext, type ReactNode } from 'react'
import type { UsageDashboardController } from './useUsageDashboard'

const UsageDashboardContext = createContext<UsageDashboardController | null>(null)

export function UsageDashboardProvider({
  value,
  children,
}: {
  value: UsageDashboardController
  children: ReactNode
}) {
  return (
    <UsageDashboardContext.Provider value={value}>
      {children}
    </UsageDashboardContext.Provider>
  )
}

export function useUsageDashboardContext(): UsageDashboardController {
  const context = useContext(UsageDashboardContext)
  if (!context) {
    throw new Error('useUsageDashboardContext 必须在 UsageDashboardView 的 provide 作用域内调用')
  }
  return context
}
