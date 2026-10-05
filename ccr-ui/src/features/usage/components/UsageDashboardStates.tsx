import { AsyncStatePanel } from '@/ui'
import type { ReactNode } from 'react'
import type { UsageDashboardController } from '../useUsageDashboard'

interface UsageDashboardStatesProps {
  usage: UsageDashboardController
  hasDashboardData: boolean
  runtimeCopy: { title: string; description: string; actionLabel: string }
  onHome: () => void
  content: ReactNode
}

export function UsageDashboardStates({
  usage,
  hasDashboardData,
  runtimeCopy,
  onHome,
  content,
}: UsageDashboardStatesProps) {
  if (usage.runtimeUnavailable) {
    return (
      <AsyncStatePanel
        state="runtime-unavailable"
        title={runtimeCopy.title}
        description={runtimeCopy.description}
        actionLabel={runtimeCopy.actionLabel}
        actionIcon="ArrowLeft"
        onAction={onHome}
      />
    )
  }
  if (usage.loading && !hasDashboardData) {
    return <AsyncStatePanel state="loading" title={usage.t('usage.states.loading')} compact />
  }
  if (usage.error) {
    return (
      <AsyncStatePanel
        state="error"
        title={usage.t('usage.states.loadFailed')}
        description={usage.error}
        actionLabel={usage.t('common.retry')}
        actionIcon="RefreshCw"
        onAction={usage.onFilterChange}
      />
    )
  }
  if (usage.dashboardUnsupported) {
    return (
      <AsyncStatePanel
        state="empty"
        title={usage.unsupportedStateTitle}
        description={usage.unsupportedStateDescription}
        icon="Database"
        compact
      />
    )
  }
  if (!usage.dashboardReady) {
    return <AsyncStatePanel state="loading" title={usage.t('common.loading')} compact />
  }
  if (usage.showEmptyState) {
    return (
      <AsyncStatePanel
        state="empty"
        title={usage.emptyStateTitle}
        description={usage.emptyStateDescription}
        compact
      />
    )
  }
  return (
    <div className={usage.loading ? 'usage-content usage-content--busy' : 'usage-content'} aria-busy={usage.loading}>
      {content}
    </div>
  )
}
