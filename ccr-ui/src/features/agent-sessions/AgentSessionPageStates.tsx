import { memo } from 'react'
import { SIcon, Spinner } from '@/ui'
import { getErrorMessage } from '@/utils/errorHandler'
import type { AgentSessionListItemDto } from '@/types/generated/agent_sessions/AgentSessionListItemDto'
import type { SessionIndexJobSnapshot } from '@/types/generated/usage/SessionIndexJobSnapshot'
import type { TranslateFunction } from '@/utils/tf'

export const resolveActiveArchiveId = (
  sessions: AgentSessionListItemDto[],
  selectedArchiveId: string,
  skippedArchiveIds: ReadonlySet<string> = new Set(),
): string => {
  const selected = sessions.find((session) => session.archive_id === selectedArchiveId)
  if (selected) return selected.archive_id
  return sessions.find((session) => (
    session.source_state !== 'missing'
    && session.source_state !== 'deleted_by_user'
    && !skippedArchiveIds.has(session.archive_id)
  ))?.archive_id ?? ''
}

export const isRefreshRunning = (status: string | undefined, pending: boolean): boolean =>
  pending || status === 'pending' || status === 'running'

interface AgentRefreshBadgeProps {
  snapshot?: SessionIndexJobSnapshot
  refreshing: boolean
  t: TranslateFunction
}

export const AgentRefreshBadge = memo(function AgentRefreshBadge({
  snapshot,
  refreshing,
  t,
}: AgentRefreshBadgeProps) {
  if (!snapshot) return null
  return (
    <span className="inline-flex items-center gap-2 rounded-full border border-border-default/20 bg-bg-elevated px-3 py-1.5 text-xs text-text-secondary" role="status">
      {refreshing ? <Spinner size="sm" /> : <SIcon name={snapshot.status === 'failed' ? 'AlertTriangle' : 'CheckCircle2'} size="w-4 h-4" />}
      <span>
        {refreshing ? t('agentSessions.refreshing') : t('agentSessions.refreshComplete')}
        {' · '}{snapshot.parsed}/{snapshot.discovered}
      </span>
    </span>
  )
})

interface AgentSessionErrorBannerProps {
  providerError?: Error | null
  refreshError?: Error | null
  statusError?: Error | null
}

export const AgentSessionErrorBanner = memo(function AgentSessionErrorBanner({
  providerError,
  refreshError,
  statusError,
}: AgentSessionErrorBannerProps) {
  const error = providerError ?? refreshError ?? statusError
  if (!error) return null
  return (
    <div className="flex items-start gap-2 rounded-xl border border-accent-danger/20 bg-accent-danger/10 px-4 py-3 text-sm text-accent-danger" role="alert">
      <SIcon name="AlertTriangle" size="w-4 h-4" className="mt-0.5 shrink-0" />
      <span>{getErrorMessage(error)}</span>
    </div>
  )
})

