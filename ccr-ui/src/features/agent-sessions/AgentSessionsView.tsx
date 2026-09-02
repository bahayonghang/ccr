import { useCallback, useDeferredValue, useEffect, useMemo, useRef, useState, type ReactNode } from 'react'
import { useInfiniteQuery, useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useForm } from 'react-hook-form'
import { Link } from 'react-router'
import { agentSessionsApi, getCurrentEnvironment } from '@/api'
import { useAppLocale, useAppT } from '@/i18n'
import { getErrorMessage } from '@/utils/errorHandler'
import { EmptyState, PageHeader, PageShell, SIcon, Spinner, buttonClass } from '@/ui'
import type { AgentSessionAgentDto } from '@/types/generated/agent_sessions/AgentSessionAgentDto'
import type { AgentSessionListRequestDto } from '@/types/generated/agent_sessions/AgentSessionListRequestDto'
import type { SessionIndexJobSnapshot } from '@/types/generated/usage/SessionIndexJobSnapshot'
import type { TranslateFunction } from '@/utils/tf'
import { AgentSessionList } from './AgentSessionList'
import {
  AgentRefreshBadge,
  AgentSessionErrorBanner,
  isRefreshRunning,
  resolveActiveArchiveId,
} from './AgentSessionPageStates'
import { AgentSessionProviderStrip } from './AgentSessionProviderStrip'
import { AgentSessionTranscript } from './AgentSessionTranscript'
import {
  DEFAULT_AGENT_SESSION_FILTERS,
  dateBoundary,
  isUnreadableAgentSessionError,
  type AgentSessionFilterValues,
} from './model'
import { agentSessionKeys } from './queries'

const LIST_LIMIT = 80
const DETAIL_LIMIT = 100

const isLocalEnvironment = (ready: boolean, environmentType?: string): boolean =>
  ready ? environmentType === 'local' : false

const canReadAgentSessions = (localEnvironment: boolean, locator: string): boolean =>
  localEnvironment ? Boolean(locator) : false

const canStartAgentSessionRefresh = (localEnvironment: boolean, refreshing: boolean): boolean =>
  localEnvironment ? !refreshing : false

interface AgentSessionEnvironmentGateProps {
  pending: boolean
  error: unknown
  localEnvironment: boolean
  t: ReturnType<typeof useAppT>
  children: ReactNode
}

const buildAgentSessionListRequest = (
  filters: AgentSessionFilterValues,
  selectedAgents: AgentSessionAgentDto[],
): AgentSessionListRequestDto => ({
  agents: selectedAgents.length > 0 ? [...selectedAgents].sort() : undefined,
  query: filters.q.trim() || undefined,
  cwd_prefix: filters.cwd.trim() || undefined,
  started_at: dateBoundary(filters.startedAt, false),
  ended_at: dateBoundary(filters.endedAt, true),
  source_state: filters.sourceState === 'all' ? undefined : filters.sourceState,
  fidelity: filters.fidelity === 'all' ? undefined : filters.fidelity,
  limit: LIST_LIMIT,
})

function AgentSessionEnvironmentGate({
  pending,
  error,
  localEnvironment,
  t,
  children,
}: AgentSessionEnvironmentGateProps) {
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
      <EmptyState
        icon="AlertTriangle"
        title={t('agentSessions.environmentErrorTitle')}
        description={getErrorMessage(error)}
      />
    )
  }
  if (!localEnvironment) {
    return (
      <EmptyState
        icon="MonitorOff"
        title={t('agentSessions.localOnlyTitle')}
        description={t('agentSessions.localOnlyDescription')}
      />
    )
  }
  return children
}

interface AgentSessionsHeaderProps {
  localEnvironment: boolean
  refreshing: boolean
  snapshot?: SessionIndexJobSnapshot
  t: TranslateFunction
  onRefresh: () => void
}

function AgentSessionsHeader({
  localEnvironment,
  refreshing,
  snapshot,
  t,
  onRefresh,
}: AgentSessionsHeaderProps) {
  return (
    <PageHeader
      title={t('agentSessions.title')}
      description={t('agentSessions.subtitle')}
      leading={
        <div className="flex h-10 w-10 items-center justify-center rounded-xl border border-accent-primary/25 bg-accent-primary/10">
          <SIcon name="MessagesSquare" size="w-6 h-6" className="text-accent-primary" />
        </div>
      }
      status={<AgentRefreshBadge snapshot={snapshot} refreshing={refreshing} t={t} />}
      actions={
        <>
          <Link to="/usage" className={buttonClass({ variant: 'secondary' })}>
            <SIcon name="Activity" size="w-4 h-4" />
            {t('agentSessions.usage')}
          </Link>
          <button type="button" className={buttonClass({ variant: 'primary' })} disabled={!canStartAgentSessionRefresh(localEnvironment, refreshing)} onClick={onRefresh}>
            <SIcon name="RefreshCw" size="w-4 h-4" className={refreshing ? 'animate-spin' : undefined} />
            {refreshing ? t('agentSessions.refreshing') : t('agentSessions.refresh')}
          </button>
        </>
      }
    />
  )
}

export function AgentSessionsView() {
  const t = useAppT()
  const locale = useAppLocale()
  const queryClient = useQueryClient()
  const filterForm = useForm<AgentSessionFilterValues>({ defaultValues: DEFAULT_AGENT_SESSION_FILTERS })
  const deferredFilters = useDeferredValue(filterForm.watch())
  const [selectedAgents, setSelectedAgents] = useState<AgentSessionAgentDto[]>([])
  const [selectedArchiveId, setSelectedArchiveId] = useState('')
  const [skippedArchiveIds, setSkippedArchiveIds] = useState<string[]>([])
  const [refreshJobId, setRefreshJobId] = useState('')
  const invalidatedJobRef = useRef('')
  const bootstrapRefreshRef = useRef(false)
  const environmentQuery = useQuery({
    queryKey: ['current-environment'],
    queryFn: getCurrentEnvironment,
    staleTime: 0,
  })
  const environmentId = environmentQuery.data?.id ?? null
  const localEnvironment = isLocalEnvironment(
    environmentQuery.isSuccess,
    environmentQuery.data?.env_type,
  )

  const listRequest = useMemo(
    () => buildAgentSessionListRequest(deferredFilters, selectedAgents),
    [deferredFilters, selectedAgents],
  )

  const providersQuery = useQuery({
    queryKey: agentSessionKeys.providers(environmentId),
    queryFn: agentSessionsApi.agentSessionsGetProviderStatus,
    enabled: localEnvironment,
  })
  const listQuery = useInfiniteQuery({
    queryKey: agentSessionKeys.list(environmentId, listRequest),
    initialPageParam: undefined as string | undefined,
    queryFn: ({ pageParam }) => agentSessionsApi.agentSessionsList({ ...listRequest, cursor: pageParam }),
    getNextPageParam: (lastPage) => lastPage.next_cursor,
    enabled: localEnvironment,
  })
  const sessions = useMemo(
    () => listQuery.data?.pages.flatMap((page) => Array.isArray(page?.items) ? page.items : []) ?? [],
    [listQuery.data],
  )
  const skippedArchiveIdSet = useMemo(() => new Set(skippedArchiveIds), [skippedArchiveIds])
  const activeArchiveId = resolveActiveArchiveId(sessions, selectedArchiveId, skippedArchiveIdSet)
  const activeSession = sessions.find((session) => session.archive_id === activeArchiveId)

  const detailQuery = useInfiniteQuery({
    queryKey: agentSessionKeys.detail(environmentId, activeArchiveId),
    initialPageParam: undefined as number | undefined,
    queryFn: ({ pageParam }) => agentSessionsApi.agentSessionsGetDetail({
      archive_id: activeArchiveId,
      before_cursor: pageParam,
      limit: DETAIL_LIMIT,
    }),
    getNextPageParam: (lastPage) => lastPage.has_older ? lastPage.next_before : undefined,
    enabled: canReadAgentSessions(localEnvironment, activeArchiveId),
  })

  const refreshMutation = useMutation({
    mutationFn: agentSessionsApi.agentSessionsStartRefresh,
    onSuccess: (response) => {
      setRefreshJobId(response.job_id)
    },
  })
  const refreshStatusQuery = useQuery({
    queryKey: agentSessionKeys.refresh(environmentId, refreshJobId),
    queryFn: () => agentSessionsApi.agentSessionsGetRefreshStatus(refreshJobId),
    enabled: canReadAgentSessions(localEnvironment, refreshJobId),
    refetchInterval: (query) => {
      const status = query.state.data?.status
      return status === 'finished' || status === 'failed' ? false : 800
    },
  })
  const refreshStatus = refreshStatusQuery.data?.status ?? refreshMutation.data?.snapshot.status
  const refreshing = isRefreshRunning(refreshStatus, refreshMutation.isPending)

  useEffect(() => {
    if (!localEnvironment || bootstrapRefreshRef.current) return
    bootstrapRefreshRef.current = true
    refreshMutation.mutate()
  }, [localEnvironment, refreshMutation])

  useEffect(() => {
    if (!refreshJobId || invalidatedJobRef.current === refreshJobId) return
    if (refreshStatus !== 'finished' && refreshStatus !== 'failed') return
    invalidatedJobRef.current = refreshJobId
    setSkippedArchiveIds([])
    void queryClient.invalidateQueries({ queryKey: agentSessionKeys.all })
  }, [queryClient, refreshJobId, refreshStatus])

  useEffect(() => {
    if (selectedArchiveId || !activeArchiveId || !detailQuery.isError) return
    if (!isUnreadableAgentSessionError(getErrorMessage(detailQuery.error))) return
    setSkippedArchiveIds((current) => (
      current.includes(activeArchiveId) ? current : [...current, activeArchiveId]
    ))
  }, [selectedArchiveId, activeArchiveId, detailQuery.error, detailQuery.isError])

  const handleToggleAgent = useCallback((agent: AgentSessionAgentDto) => {
    setSelectedAgents((current) => current.includes(agent)
      ? current.filter((item) => item !== agent)
      : [...current, agent])
  }, [])
  const handleSelectSession = useCallback((archiveId: string) => {
    setSelectedArchiveId(archiveId)
  }, [])
  const handleRefresh = useCallback(() => {
    refreshMutation.mutate()
  }, [refreshMutation])
  const handleRetryList = useCallback(() => {
    void listQuery.refetch()
  }, [listQuery])
  const handleLoadMore = useCallback(() => {
    void listQuery.fetchNextPage()
  }, [listQuery])
  const handleRetryDetail = useCallback(() => {
    void detailQuery.refetch()
  }, [detailQuery])
  const handleLoadOlder = useCallback(() => {
    void detailQuery.fetchNextPage()
  }, [detailQuery])
  const handleResetFilters = useCallback(() => {
    filterForm.reset(DEFAULT_AGENT_SESSION_FILTERS)
    setSelectedAgents([])
  }, [filterForm])

  const jobSnapshot = refreshStatusQuery.data ?? refreshMutation.data?.snapshot
  return (
    <PageShell
      className="agent-sessions-view"
      header={
        <AgentSessionsHeader
          localEnvironment={localEnvironment}
          refreshing={refreshing}
          snapshot={jobSnapshot}
          t={t}
          onRefresh={handleRefresh}
        />
      }
    >
      <div className="space-y-4">
        <AgentSessionEnvironmentGate
          pending={environmentQuery.isPending}
          error={environmentQuery.error}
          localEnvironment={localEnvironment}
          t={t}
        >
          <>
        <div className="rounded-2xl border border-border-default/20 bg-bg-surface p-4">
          <AgentSessionProviderStrip
            statuses={providersQuery.data ?? []}
            selectedAgents={selectedAgents}
            pending={providersQuery.isPending}
            failed={Boolean(providersQuery.error)}
            t={t}
            onToggle={handleToggleAgent}
          />
        </div>

        <section className="rounded-2xl border border-border-default/20 bg-bg-surface p-4" aria-labelledby="agent-session-filter-title">
          <div className="mb-3 flex items-center justify-between gap-3">
            <h2 id="agent-session-filter-title" className="text-sm font-semibold text-text-primary">{t('agentSessions.filters')}</h2>
            <button type="button" className={buttonClass({ variant: 'quiet', size: 'sm' })} onClick={handleResetFilters}>
              <SIcon name="RotateCcw" size="w-4 h-4" />
              {t('agentSessions.clearFilters')}
            </button>
          </div>
          <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-[minmax(16rem,2fr)_minmax(12rem,1.5fr)_repeat(4,minmax(8rem,1fr))]">
            <label className="block">
              <span className="mb-1 block text-xs font-medium text-text-muted">{t('agentSessions.search')}</span>
              <input type="search" className="w-full rounded-xl border border-border-default/30 bg-bg-elevated px-3 py-2 text-sm text-text-primary" placeholder={t('agentSessions.searchPlaceholder')} {...filterForm.register('q')} />
            </label>
            <label className="block">
              <span className="mb-1 block text-xs font-medium text-text-muted">{t('agentSessions.cwd')}</span>
              <input type="text" className="w-full rounded-xl border border-border-default/30 bg-bg-elevated px-3 py-2 text-sm text-text-primary" placeholder={t('agentSessions.cwdPlaceholder')} {...filterForm.register('cwd')} />
            </label>
            <label className="block">
              <span className="mb-1 block text-xs font-medium text-text-muted">{t('agentSessions.startedAt')}</span>
              <input type="date" className="w-full rounded-xl border border-border-default/30 bg-bg-elevated px-3 py-2 text-sm text-text-primary" {...filterForm.register('startedAt')} />
            </label>
            <label className="block">
              <span className="mb-1 block text-xs font-medium text-text-muted">{t('agentSessions.endedAt')}</span>
              <input type="date" className="w-full rounded-xl border border-border-default/30 bg-bg-elevated px-3 py-2 text-sm text-text-primary" {...filterForm.register('endedAt')} />
            </label>
            <label className="block">
              <span className="mb-1 block text-xs font-medium text-text-muted">{t('agentSessions.state')}</span>
              <select className="w-full rounded-xl border border-border-default/30 bg-bg-elevated px-3 py-2 text-sm text-text-primary" {...filterForm.register('sourceState')}>
                <option value="all">{t('agentSessions.all')}</option>
                <option value="live">{t('agentSessions.live')}</option>
                <option value="missing">{t('agentSessions.missing')}</option>
                <option value="deleted_by_user">{t('agentSessions.deleted')}</option>
              </select>
            </label>
            <label className="block">
              <span className="mb-1 block text-xs font-medium text-text-muted">{t('agentSessions.fidelity')}</span>
              <select className="w-full rounded-xl border border-border-default/30 bg-bg-elevated px-3 py-2 text-sm text-text-primary" {...filterForm.register('fidelity')}>
                <option value="all">{t('agentSessions.all')}</option>
                <option value="full">{t('agentSessions.full')}</option>
                <option value="partial">{t('agentSessions.partial')}</option>
                <option value="locked">{t('agentSessions.locked')}</option>
              </select>
            </label>
          </div>
        </section>

        <AgentSessionErrorBanner providerError={providersQuery.error} refreshError={refreshMutation.error} statusError={refreshStatusQuery.error} />

        <div className="grid gap-4 min-[56.25rem]:grid-cols-[minmax(20rem,24rem)_minmax(0,1fr)]">
          <section className="min-w-0 rounded-2xl border border-border-default/20 bg-bg-surface p-4" aria-labelledby="agent-session-list-title">
            <h2 id="agent-session-list-title" className="mb-3 text-base font-semibold text-text-primary">{t('agentSessions.sessions')}</h2>
            <AgentSessionList
              items={sessions}
              selectedArchiveId={activeArchiveId}
              locale={locale}
              pending={listQuery.isPending}
              error={listQuery.error ? getErrorMessage(listQuery.error) : undefined}
              hasNextPage={Boolean(listQuery.hasNextPage)}
              fetchingNextPage={listQuery.isFetchingNextPage}
              t={t}
              onSelect={handleSelectSession}
              onLoadMore={handleLoadMore}
              onRefresh={handleRetryList}
            />
          </section>
          <section className="min-w-0 rounded-2xl border border-border-default/20 bg-bg-surface p-4" aria-labelledby="agent-session-detail-title">
            <h2 id="agent-session-detail-title" className="sr-only">{t('agentSessions.detail')}</h2>
            <AgentSessionTranscript
              session={activeSession}
              details={detailQuery.data?.pages ?? []}
              locale={locale}
              pending={Boolean(activeArchiveId) && detailQuery.isPending}
              error={detailQuery.error ? getErrorMessage(detailQuery.error) : undefined}
              hasOlder={Boolean(detailQuery.hasNextPage)}
              fetchingOlder={detailQuery.isFetchingNextPage}
              t={t}
              onLoadOlder={handleLoadOlder}
              onRetry={handleRetryDetail}
            />
          </section>
        </div>
          </>
        </AgentSessionEnvironmentGate>
      </div>
    </PageShell>
  )
}
