import { memo, useMemo, useRef } from 'react'
import { useVirtualizer } from '@tanstack/react-virtual'
import { useForm } from 'react-hook-form'
import { EmptyState, SIcon, Spinner, buttonClass, cn } from '@/ui'
import type { AgentSessionDetailDto } from '@/types/generated/agent_sessions/AgentSessionDetailDto'
import type { AgentSessionListItemDto } from '@/types/generated/agent_sessions/AgentSessionListItemDto'
import type { AgentSessionMessageDto } from '@/types/generated/agent_sessions/AgentSessionMessageDto'
import type { TranslateFunction } from '@/utils/tf'
import { formatAgentName, formatSessionTime, resolveAgentSessionDetailError } from './model'

interface MessageRowProps {
  message: AgentSessionMessageDto
  locale: string
  t: TranslateFunction
}

const MessageRow = memo(function MessageRow({ message, locale, t }: MessageRowProps) {
  const roleKey = message.role === 'user' || message.role === 'tool' ? message.role : 'assistant'
  return (
    <article className={cn(
      'rounded-xl border border-border-default/20 bg-bg-elevated p-4',
      roleKey === 'user' && 'border-accent-primary/25',
    )}>
      <header className="mb-3 flex flex-wrap items-center justify-between gap-2 text-xs">
        <span className="flex items-center gap-2 font-semibold text-text-secondary">
          <SIcon name={roleKey === 'user' ? 'User' : roleKey === 'tool' ? 'Wrench' : 'Bot'} size="w-4 h-4" />
          {t(`agentSessions.roles.${roleKey}`)}
          {message.tool_name ? <span className="font-mono font-normal text-text-muted">{message.tool_name}</span> : null}
        </span>
        <span className="text-text-ghost">
          {message.timestamp ? formatSessionTime(message.timestamp, locale, t('agentSessions.unknownTime')) : `#${message.ordinal}`}
        </span>
      </header>
      <p className="whitespace-pre-wrap break-words text-sm leading-6 text-text-secondary">{message.content}</p>
      {message.clipped ? (
        <p className="mt-3 flex items-start gap-2 text-xs text-accent-warning">
          <SIcon name="Scissors" size="w-4 h-4" className="mt-0.5 shrink-0" />
          <span>{t('agentSessions.clippedNotice')}</span>
        </p>
      ) : null}
    </article>
  )
})

interface AgentSessionTranscriptProps {
  session?: AgentSessionListItemDto
  details: AgentSessionDetailDto[]
  locale: string
  pending: boolean
  error?: string
  hasOlder: boolean
  fetchingOlder: boolean
  t: TranslateFunction
  onLoadOlder: () => void
  onRetry: () => void
}

export const AgentSessionTranscript = memo(function AgentSessionTranscript({
  session,
  details,
  locale,
  pending,
  error,
  hasOlder,
  fetchingOlder,
  t,
  onLoadOlder,
  onRetry,
}: AgentSessionTranscriptProps) {
  const findForm = useForm({ defaultValues: { q: '' } })
  const findQuery = findForm.watch('q').trim().toLocaleLowerCase()
  const messages = useMemo(() => {
    const unique = new Map<string, AgentSessionMessageDto>()
    details.forEach((page) => {
      if (Array.isArray(page?.messages)) {
        page.messages.forEach((message) => unique.set(message.key, message))
      }
    })
    return [...unique.values()].sort((left, right) => left.ordinal - right.ordinal)
  }, [details])
  const visibleMessages = useMemo(() => {
    if (!findQuery) return messages
    return messages.filter((message) => [message.content, message.tool_name ?? '', message.role]
      .some((value) => value.toLocaleLowerCase().includes(findQuery)))
  }, [findQuery, messages])
  const scrollRef = useRef<HTMLDivElement>(null)
  const virtualizer = useVirtualizer({
    count: visibleMessages.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => 148,
    overscan: 6,
    getItemKey: (index) => visibleMessages[index]?.key ?? `missing-${index}`,
  })

  if (!session) {
    return <EmptyState icon="MessagesSquare" title={t('agentSessions.selectTitle')} description={t('agentSessions.selectDescription')} />
  }
  if (pending) {
    return (
      <div className="flex min-h-[30rem] flex-col items-center justify-center gap-3 text-sm text-text-muted" role="status">
        <Spinner />
        <span>{t('common.loading')}</span>
      </div>
    )
  }
  if (error) {
    const empty = resolveAgentSessionDetailError(error, t)
    return <EmptyState icon="AlertTriangle" title={empty.title} description={empty.description} actionText={t('common.retry')} onAction={onRetry} />
  }

  const fidelity = details[0]?.fidelity ?? session.fidelity
  return (
    <>
      <div className="mb-4 border-b border-border-default/20 pb-4">
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div className="min-w-0">
            <h2 className="truncate text-lg font-semibold text-text-primary">{session.title || session.session_id}</h2>
            <p className="mt-1 truncate font-mono text-xs text-text-ghost">{session.session_id}</p>
          </div>
          <span className="rounded-full border border-border-default/20 bg-bg-elevated px-3 py-1 text-xs text-text-secondary">
            {formatAgentName(session.agent, t)} · {session.variant}
          </span>
        </div>
        <p className="mt-3 text-sm text-text-muted">{t('agentSessions.detailHint')}</p>
        {fidelity === 'partial' ? (
          <p className="mt-3 flex items-start gap-2 rounded-lg bg-accent-warning/10 p-3 text-sm text-accent-warning">
            <SIcon name="AlertTriangle" size="w-4 h-4" className="mt-0.5 shrink-0" />
            {t('agentSessions.partialNotice')}
          </p>
        ) : null}
        {fidelity === 'locked' ? (
          <p className="mt-3 flex items-start gap-2 rounded-lg bg-accent-warning/10 p-3 text-sm text-accent-warning">
            <SIcon name="Lock" size="w-4 h-4" className="mt-0.5 shrink-0" />
            {t('agentSessions.lockedNotice')}
          </p>
        ) : null}
      </div>

      <label className="mb-3 block">
        <span className="mb-1 block text-xs font-medium text-text-muted">{t('agentSessions.loadedFind')}</span>
        <span className="relative block">
          <SIcon name="Search" size="w-4 h-4" className="absolute left-3 top-1/2 -translate-y-1/2 text-text-muted" />
          <input
            type="search"
            className="w-full rounded-xl border border-border-default/30 bg-bg-elevated py-2 pr-3 pl-9 text-sm text-text-primary"
            placeholder={t('agentSessions.localFindHint')}
            {...findForm.register('q')}
          />
        </span>
      </label>

      {hasOlder ? (
        <button type="button" className={buttonClass({ variant: 'quiet', className: 'mb-3 w-full' })} disabled={fetchingOlder} onClick={onLoadOlder}>
          {fetchingOlder ? <Spinner size="sm" /> : <SIcon name="ChevronUp" size="w-4 h-4" />}
          {t('agentSessions.loadOlder')}
        </button>
      ) : null}

      {visibleMessages.length === 0 ? (
        <EmptyState icon="SearchX" title={t('agentSessions.noTranscriptMatches')} description={t('agentSessions.localFindHint')} />
      ) : (
        <div ref={scrollRef} className="h-[38rem] overflow-y-auto pr-1" data-testid="agent-transcript-virtual-list">
          <div className="relative w-full" style={{ height: virtualizer.getTotalSize() }}>
            {virtualizer.getVirtualItems().map((virtualRow) => {
              const message = visibleMessages[virtualRow.index]
              if (!message) return null
              return (
                <div
                  key={message.key}
                  ref={virtualizer.measureElement}
                  data-index={virtualRow.index}
                  className="absolute top-0 left-0 w-full pb-3"
                  style={{ transform: `translateY(${virtualRow.start}px)` }}
                >
                  <MessageRow message={message} locale={locale} t={t} />
                </div>
              )
            })}
          </div>
        </div>
      )}
    </>
  )
})
