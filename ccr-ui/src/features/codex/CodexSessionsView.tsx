import { useCallback, useMemo, useRef, useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useVirtualizer } from '@tanstack/react-virtual'
import { useForm } from 'react-hook-form'
import { Link } from 'react-router'
import {
  cloneCodexSession,
  deleteCodexSession,
  exportCodexSession,
  getCodexSessionDetail,
  listCodexSessions,
} from '@/api'
import { surfaceNotify } from '@/configs/surfaceNotify'
import { copyText } from '@/utils/clipboard'
import { getErrorMessage } from '@/utils/errorHandler'
import { PageHeader, PageShell, PillToggleGroup, SIcon } from '@/ui'
import { CodexSubnav } from './CodexSubnav'
import { SessionDetailPanel } from './sessions/SessionDetailPanel'
import { SessionListPanel } from './sessions/SessionListPanel'
import { DETAIL_LIMIT, EXPORT_LIMIT, SESSION_LIMIT, formatTokenCount } from './sessions/session-format'
import { panelCardClass, primaryBtnClass, secondaryBtnClass } from './ui-classes'
import { codexKeys } from './queries'
import { useCodexLocale } from './useCodexLocale'
import type { CodexSessionSummary } from '@/types'

type SessionSort = 'recent' | 'tokens'

export function CodexSessionsView() {
  const { tt } = useCodexLocale()
  const queryClient = useQueryClient()
  const searchForm = useForm({ defaultValues: { q: '' } })
  const searchQuery = searchForm.watch('q')
  const [sessionSort, setSessionSort] = useState<SessionSort>('recent')
  const [selectedFilePath, setSelectedFilePath] = useState('')
  const listRef = useRef<HTMLDivElement>(null)
  const unknownTime = tt('未知时间', 'Unknown time')

  const listQuery = useQuery({
    queryKey: codexKeys.sessions.list(SESSION_LIMIT),
    queryFn: () => listCodexSessions({ limit: SESSION_LIMIT }),
  })
  const sessions = useMemo(() => listQuery.data?.sessions ?? [], [listQuery.data])
  const filteredSessions = useMemo(() => {
    const query = searchQuery.trim().toLowerCase()
    const matched = query
      ? sessions.filter((session) =>
          [session.session_id, session.cwd ?? '', session.model ?? '', session.preview ?? '', session.relative_path]
            .some((value) => value.toLowerCase().includes(query)),
        )
      : [...sessions]
    if (sessionSort !== 'tokens') return matched
    return matched.sort((left, right) => {
      const rightTokens = right.total_input_tokens + right.total_output_tokens
      const leftTokens = left.total_input_tokens + left.total_output_tokens
      return rightTokens - leftTokens
    })
  }, [searchQuery, sessionSort, sessions])

  const selectedPath = selectedFilePath || filteredSessions[0]?.file_path || ''
  const detailQuery = useQuery({
    queryKey: codexKeys.sessions.detail(selectedPath),
    queryFn: () => getCodexSessionDetail(selectedPath, DETAIL_LIMIT),
    enabled: Boolean(selectedPath),
  })
  const selectedSession = detailQuery.data?.session ?? null
  const totalTokens = sessions.reduce(
    (total, session) => total + session.total_input_tokens + session.total_output_tokens,
    0,
  )

  const virtualizer = useVirtualizer({
    count: filteredSessions.length,
    getScrollElement: () => listRef.current,
    estimateSize: () => 132,
    overscan: 8,
  })

  const handleOpen = useCallback((filePath: string) => {
    setSelectedFilePath(filePath)
  }, [])
  const handleRefresh = useCallback(() => {
    void listQuery.refetch()
    void detailQuery.refetch()
  }, [detailQuery, listQuery])
  const handleClearSearch = useCallback(() => {
    searchForm.reset({ q: '' })
  }, [searchForm])

  const exportMutation = useMutation({
    mutationFn: async (session: CodexSessionSummary) => exportCodexSession(session.file_path, EXPORT_LIMIT),
    onSuccess: (payload) => {
      const blob = new Blob([payload.content], { type: 'text/markdown;charset=utf-8' })
      const url = URL.createObjectURL(blob)
      const link = document.createElement('a')
      link.href = url
      link.download = payload.file_name
      document.body.appendChild(link)
      link.click()
      document.body.removeChild(link)
      URL.revokeObjectURL(url)
      surfaceNotify.success(tt('Session Markdown 已导出', 'Session Markdown exported'))
    },
    onError: (error) => surfaceNotify.error(getErrorMessage(error)),
  })
  const cloneMutation = useMutation({
    mutationFn: async (session: CodexSessionSummary) => cloneCodexSession(session.file_path),
    onSuccess: async (payload) => {
      await queryClient.invalidateQueries({ queryKey: codexKeys.sessions.all })
      setSelectedFilePath(payload.session.file_path)
      surfaceNotify.success(tt('Session 已克隆到本地会话目录', 'Session cloned to the local session directory'))
    },
    onError: (error) => surfaceNotify.error(getErrorMessage(error)),
  })
  const deleteMutation = useMutation({
    mutationFn: async (session: CodexSessionSummary) => deleteCodexSession(session.file_path),
    onSuccess: async () => {
      const fallback = sessions.find((session) => session.file_path !== selectedPath)?.file_path ?? ''
      await queryClient.invalidateQueries({ queryKey: codexKeys.sessions.all })
      setSelectedFilePath(fallback)
      surfaceNotify.success(tt('Session 已删除', 'Session deleted'))
    },
    onError: (error) => surfaceNotify.error(getErrorMessage(error)),
  })

  const actionLoading = exportMutation.isPending || cloneMutation.isPending || deleteMutation.isPending
  const handleExport = useCallback(() => {
    if (selectedSession) void exportMutation.mutateAsync(selectedSession)
  }, [exportMutation, selectedSession])
  const handleClone = useCallback(() => {
    if (selectedSession) void cloneMutation.mutateAsync(selectedSession)
  }, [cloneMutation, selectedSession])
  const handleDelete = useCallback(async () => {
    if (!selectedSession) return
    const ok = await surfaceNotify.confirm({
      title: tt('删除 Session', 'Delete session'),
      message: tt(
        `确认删除 ${selectedSession.session_id} 吗？这个操作会直接删除本地 JSONL 文件。`,
        `Delete ${selectedSession.session_id}? This will delete the local JSONL file.`,
      ),
      confirmText: tt('删除', 'Delete'),
      cancelText: tt('取消', 'Cancel'),
      type: 'danger',
    })
    if (ok) void deleteMutation.mutateAsync(selectedSession)
  }, [deleteMutation, selectedSession, tt])
  const handleCopyPath = useCallback(async () => {
    if (!selectedSession) return
    const ok = await copyText(selectedSession.file_path)
    surfaceNotify[ok ? 'success' : 'error'](ok ? tt('已复制 session 文件路径', 'Session file path copied') : tt('复制失败', 'Copy failed'))
  }, [selectedSession, tt])
  const handleCopyCwd = useCallback(async () => {
    if (!selectedSession?.cwd) return
    const ok = await copyText(selectedSession.cwd)
    surfaceNotify[ok ? 'success' : 'error'](ok ? tt('已复制工作目录', 'Working directory copied') : tt('复制失败', 'Copy failed'))
  }, [selectedSession, tt])

  const sortOptions = useMemo(
    () => [
      { value: 'recent' as const, label: tt('最近', 'Recent') },
      { value: 'tokens' as const, label: 'Tokens' },
    ],
    [tt],
  )

  return (
    <PageShell
      header={
        <PageHeader
          title={tt('Codex 会话', 'Codex Sessions')}
          description={tt(
            '直接读取本地 `~/.codex/sessions`，集中查看会话上下文、导出记录和复制工作流样本。',
            'Read local `~/.codex/sessions` directly to inspect session context, export records, and copy workflow samples.',
          )}
          leading={
            <div className="flex h-10 w-10 items-center justify-center rounded-xl border border-platform-codex/20 bg-platform-codex/10">
              <SIcon name="MessagesSquare" size="w-6 h-6" className="text-platform-codex" />
            </div>
          }
          actions={
            <div className="flex flex-wrap gap-2">
              <Link to="/codex" className={secondaryBtnClass}>
                <SIcon name="ArrowLeft" size="w-4 h-4" />
                <span>{tt('返回 Codex', 'Back to Codex')}</span>
              </Link>
              <button type="button" className={primaryBtnClass} disabled={listQuery.isFetching} onClick={handleRefresh}>
                <SIcon name="RefreshCw" size="w-4 h-4" className={listQuery.isFetching ? 'animate-spin' : undefined} />
                {tt('刷新列表', 'Refresh list')}
              </button>
            </div>
          }
        />
      }
      subnav={<CodexSubnav />}
    >
      <div className="mb-4 grid gap-4 md:grid-cols-3">
        <div className={panelCardClass}>
          <p className="text-xs font-medium text-text-ghost">{tt('已加载会话', 'Loaded sessions')}</p>
          <p className="mt-2 text-2xl font-semibold tabular-nums text-text-primary">{sessions.length}</p>
          <p className="mt-2 text-sm text-text-muted">
            {tt(`当前窗口最多展示 ${SESSION_LIMIT} 条最近记录`, `This window shows up to ${SESSION_LIMIT} recent records`)}
          </p>
        </div>
        <div className={panelCardClass}>
          <p className="text-xs font-medium text-text-ghost">{tt('列表 Tokens', 'List tokens')}</p>
          <p className="mt-2 text-2xl font-semibold tabular-nums text-text-primary">{formatTokenCount(totalTokens)}</p>
          <p className="mt-2 text-sm text-text-muted">{tt('来自当前已加载的会话摘要', 'From the currently loaded session summaries')}</p>
        </div>
        <div className={panelCardClass}>
          <p className="text-xs font-medium text-text-ghost">{tt('当前会话消息', 'Current session messages')}</p>
          <p className="mt-2 text-2xl font-semibold tabular-nums text-text-primary">{selectedSession?.message_count ?? 0}</p>
          <p className="mt-2 text-sm text-text-muted">{tt('仅统计用户与助手消息', 'Only user and assistant messages are counted')}</p>
        </div>
      </div>

      {listQuery.error ? (
        <div className="mb-4 flex items-center gap-2 rounded-xl border border-accent-danger/20 bg-accent-danger/10 px-4 py-3 text-sm text-accent-danger">
          <SIcon name="AlertTriangle" size="w-4 h-4" />
          <span>{getErrorMessage(listQuery.error)}</span>
        </div>
      ) : null}

      <div className="grid gap-4 xl:grid-cols-[minmax(22.5rem,26.25rem)_minmax(0,1fr)]">
        <section className={`${panelCardClass} min-h-[45rem]`}>
          <div className="mb-4 flex flex-col gap-4">
            <div>
              <h2 className="text-base font-semibold text-text-primary">{tt('最近会话', 'Recent sessions')}</h2>
              <p className="text-sm text-text-muted">
                {tt('左侧列表用于快速切换，右侧查看详情和导出', 'Use the left list for quick switching and the right panel for details and export')}
              </p>
              <PillToggleGroup className="mt-3" options={sortOptions} value={sessionSort} onValueChange={setSessionSort} />
            </div>
            <label className="relative block">
              <SIcon name="Search" size="w-4 h-4" className="absolute left-3 top-1/2 -translate-y-1/2 text-text-muted" />
              <input
                type="text"
                className="w-full rounded-xl border border-border-default bg-bg-elevated py-2 pr-3 pl-9 text-sm"
                placeholder={tt('搜索 session id / cwd / model', 'Search session id / cwd / model')}
                {...searchForm.register('q')}
              />
            </label>
          </div>
          <SessionListPanel
            tt={tt}
            unknownTime={unknownTime}
            listRef={listRef}
            virtualizer={virtualizer}
            filteredSessions={filteredSessions}
            selectedPath={selectedPath}
            pending={listQuery.isPending}
            onOpen={handleOpen}
            onClearSearch={handleClearSearch}
          />
        </section>

        <SessionDetailPanel
          tt={tt}
          unknownTime={unknownTime}
          selectedPath={selectedPath}
          selectedSession={selectedSession}
          detail={detailQuery.data ?? null}
          detailPending={detailQuery.isPending}
          actionLoading={actionLoading}
          onCopyPath={handleCopyPath}
          onExport={handleExport}
          onClone={handleClone}
          onDelete={handleDelete}
          onCopyCwd={handleCopyCwd}
        />
      </div>
    </PageShell>
  )
}
