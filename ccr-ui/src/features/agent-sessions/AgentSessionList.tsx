import { memo, useCallback, useRef } from 'react'
import { useVirtualizer } from '@tanstack/react-virtual'
import { EmptyState, SIcon, Spinner, buttonClass, cn } from '@/ui'
import type { AgentSessionListItemDto } from '@/types/generated/agent_sessions/AgentSessionListItemDto'
import type { TranslateFunction } from '@/utils/tf'
import { AGENT_SESSION_ICONS, formatAgentName, formatSessionTime } from './model'

interface SessionRowProps {
  item: AgentSessionListItemDto
  active: boolean
  locale: string
  t: TranslateFunction
  onSelect: (archiveId: string) => void
}

const SessionRow = memo(function SessionRow({ item, active, locale, t, onSelect }: SessionRowProps) {
  const handleSelect = useCallback(() => {
    onSelect(item.archive_id)
  }, [item.archive_id, onSelect])

  return (
    <button
      type="button"
      className={cn(
        'w-full rounded-xl border bg-bg-elevated p-3 text-left transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent-primary',
        active ? 'border-accent-primary/50 bg-accent-primary/10' : 'border-border-default/20 hover:border-border-default/50',
      )}
      onClick={handleSelect}
    >
      <span className="flex items-start justify-between gap-3">
        <span className="min-w-0">
          <span className="flex items-center gap-2">
            <SIcon name={AGENT_SESSION_ICONS[item.agent]} size="w-4 h-4" className="shrink-0 text-text-secondary" />
            <strong className="truncate text-sm text-text-primary">{item.title || item.session_id}</strong>
          </span>
          <span className="mt-1 block truncate font-mono text-xs text-text-ghost">{item.session_id}</span>
        </span>
        <span className="shrink-0 text-xs tabular-nums text-text-muted">
          {formatSessionTime(item.updated_at, locale, t('agentSessions.unknownTime'))}
        </span>
      </span>
      <span className="mt-3 flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-text-muted">
        <span>{formatAgentName(item.agent, t)} · {item.variant}</span>
        <span>{t('agentSessions.messages', { count: item.message_count })}</span>
        <span>{t('agentSessions.tools', { count: item.tool_use_count })}</span>
      </span>
      <span className="mt-2 flex items-center justify-between gap-3 text-xs text-text-ghost">
        <span className="truncate">{item.cwd || '—'}</span>
        <span className="shrink-0">{t(`agentSessions.${item.fidelity}`)}</span>
      </span>
    </button>
  )
})

interface AgentSessionListProps {
  items: AgentSessionListItemDto[]
  selectedArchiveId: string
  locale: string
  pending: boolean
  error?: string
  hasNextPage: boolean
  fetchingNextPage: boolean
  t: TranslateFunction
  onSelect: (archiveId: string) => void
  onLoadMore: () => void
  onRefresh: () => void
}

export const AgentSessionList = memo(function AgentSessionList({
  items,
  selectedArchiveId,
  locale,
  pending,
  error,
  hasNextPage,
  fetchingNextPage,
  t,
  onSelect,
  onLoadMore,
  onRefresh,
}: AgentSessionListProps) {
  const scrollRef = useRef<HTMLDivElement>(null)
  const virtualizer = useVirtualizer({
    count: items.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => 132,
    overscan: 8,
    getItemKey: (index) => items[index]?.archive_id ?? `missing-${index}`,
  })

  if (pending) {
    return (
      <div className="flex min-h-[30rem] flex-col items-center justify-center gap-3 text-sm text-text-muted" role="status">
        <Spinner />
        <span>{t('common.loading')}</span>
      </div>
    )
  }
  if (error) {
    return (
      <EmptyState icon="AlertTriangle" title={t('agentSessions.error')} description={error} actionText={t('common.retry')} onAction={onRefresh} />
    )
  }
  if (items.length === 0) {
    return <EmptyState icon="Inbox" title={t('agentSessions.emptyTitle')} description={t('agentSessions.emptyDescription')} />
  }

  return (
    <>
      <div ref={scrollRef} className="h-[34rem] overflow-y-auto pr-1" data-testid="agent-session-virtual-list">
        <div className="relative w-full" style={{ height: virtualizer.getTotalSize() }}>
          {virtualizer.getVirtualItems().map((virtualRow) => {
            const item = items[virtualRow.index]
            if (!item) return null
            return (
              <div
                key={item.archive_id}
                ref={virtualizer.measureElement}
                data-index={virtualRow.index}
                className="absolute top-0 left-0 w-full pb-2"
                style={{ transform: `translateY(${virtualRow.start}px)` }}
              >
                <SessionRow
                  item={item}
                  active={selectedArchiveId === item.archive_id}
                  locale={locale}
                  t={t}
                  onSelect={onSelect}
                />
              </div>
            )
          })}
        </div>
      </div>
      {hasNextPage ? (
        <button type="button" className={buttonClass({ variant: 'quiet', className: 'mt-3 w-full' })} disabled={fetchingNextPage} onClick={onLoadMore}>
          {fetchingNextPage ? <Spinner size="sm" /> : <SIcon name="ChevronDown" size="w-4 h-4" />}
          {t('agentSessions.loadMore')}
        </button>
      ) : null}
    </>
  )
})
