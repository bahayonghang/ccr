import { useCallback, useMemo, useRef, useState } from 'react'
import { getUsageLogsV2 } from '@/api'
import { getErrorMessage } from '@/utils/errorHandler'
import {
  buildUsageLogsQuery,
  normalizePaginatedLogs,
} from '@/utils/usageDashboardPayload'
import type { PaginatedLogs, UsagePlatform } from '@/types/usage'

const LOGS_PAGE_SIZE = 50

type LogsDirection = 'reset' | 'next' | 'prev' | 'same'

const nextPageForDirection = (input: {
  direction: LogsDirection
  logsPage: number
  logs: PaginatedLogs | null
  cursorStack: Array<string | null>
}) => {
  if (input.direction === 'reset') return { page: 1, stack: [null] as Array<string | null>, abort: false }
  if (input.direction === 'prev') return { page: Math.max(1, input.logsPage - 1), stack: input.cursorStack, abort: false }
  if (input.direction !== 'next') return { page: input.logsPage, stack: input.cursorStack, abort: false }
  if (!input.logs?.next_cursor) return { page: input.logsPage, stack: input.cursorStack, abort: true }
  const stack = [...input.cursorStack]
  stack[input.logsPage] = input.logs.next_cursor ?? null
  return { page: input.logsPage + 1, stack, abort: false }
}

export function useUsageLogsPager(params: {
  platform?: UsagePlatform
  start?: string
  end?: string
  onError: (message: string) => void
}) {
  const [logs, setLogs] = useState<PaginatedLogs | null>(null)
  const [logsLoading, setLogsLoading] = useState(false)
  const [logsPage, setLogsPage] = useState(1)
  const [logsModelFilter, setLogsModelFilter] = useState<string | undefined>(undefined)
  const cursorStackRef = useRef<Array<string | null>>([null])

  const logsTotalPages = useMemo(() => {
    const total = logs?.total
    if (!total || total <= 0) return 1
    return Math.max(1, Math.ceil(total / LOGS_PAGE_SIZE))
  }, [logs])

  const hasLogsTotal = logs?.total != null
  const canPrevLogs = logsPage > 1
  const canNextLogs = Boolean(logs?.next_cursor) || Boolean(logs && logs.total && logsPage < logsTotalPages)
  const showLogsPager = Boolean(logs && (canPrevLogs || canNextLogs || (hasLogsTotal && logsTotalPages > 1)))

  const fetchLogs = useCallback(async (direction: LogsDirection = 'same') => {
    setLogsLoading(true)
    try {
      const resolved = nextPageForDirection({
        direction,
        logsPage,
        logs,
        cursorStack: cursorStackRef.current,
      })
      if (resolved.abort) return
      cursorStackRef.current = resolved.stack
      const previousTotal = logs?.total ?? null
      setLogsPage(resolved.page)
      const result = await getUsageLogsV2(
        buildUsageLogsQuery({
          platform: params.platform,
          model: logsModelFilter,
          startDate: params.start,
          endDate: params.end,
          page: resolved.page,
          pageSize: LOGS_PAGE_SIZE,
          cursor: resolved.stack[resolved.page - 1] ?? null,
          includeTotal: resolved.page === 1 && previousTotal == null,
        }),
      )
      setLogs(normalizePaginatedLogs(result, resolved.page, LOGS_PAGE_SIZE, previousTotal))
    } catch (caught) {
      params.onError(getErrorMessage(caught))
    } finally {
      setLogsLoading(false)
    }
  }, [logs, logsModelFilter, logsPage, params])

  return {
    logs,
    logsLoading,
    logsPage,
    logsPageSize: LOGS_PAGE_SIZE,
    logsTotalPages,
    hasLogsTotal,
    canPrevLogs,
    canNextLogs,
    showLogsPager,
    fetchLogs,
    setLogsModelFilter,
  }
}
