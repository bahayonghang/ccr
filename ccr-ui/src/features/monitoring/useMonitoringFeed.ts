import { useQuery, useQueryClient } from '@tanstack/react-query'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useCallback, useEffect, useRef, useState } from 'react'
import { fetchMonitoringFeedSnapshot, monitoringKeys } from '@/features/monitoring/queries'
import { logger } from '@/utils/logger'
import { isTauriRuntime } from '@/utils/tauriRuntime'
import { createEventBatcher, type EventBatcher } from './eventBatcher'
import { mergeBatch, normalizeLoggerEntry, normalizeMonitoringEntry } from './monitoring-normalize'
import {
  DEFAULT_INITIAL_COUNT,
  DEFAULT_MAX_ENTRIES,
  MONITORING_EVENT_NAME,
  type MonitoringEntry,
  type MonitoringTokenStats,
} from './monitoring-types'

interface MonitoringFeedOptions {
  initialCount?: number
  maxEntries?: number
}

export function useMonitoringFeed(options: MonitoringFeedOptions = {}) {
  const { initialCount = DEFAULT_INITIAL_COUNT, maxEntries = DEFAULT_MAX_ENTRIES } = options
  const queryClient = useQueryClient()
  const [isConnected, setIsConnected] = useState(true)
  const [tokenStats, setTokenStats] = useState<MonitoringTokenStats | null>(null)
  const listeningRef = useRef(false)
  const disposedRef = useRef(false)
  const unlistenersRef = useRef<UnlistenFn[]>([])
  const unsubscribeLoggerRef = useRef<(() => void) | null>(null)
  const maxEntriesRef = useRef(maxEntries)
  useEffect(() => {
    maxEntriesRef.current = maxEntries
  }, [maxEntries])

  const entriesBatcherRef = useRef<EventBatcher<MonitoringEntry> | null>(null)
  const statsBatcherRef = useRef<EventBatcher<MonitoringTokenStats> | null>(null)
  const getEntriesBatcher = useCallback(() => {
    if (!entriesBatcherRef.current) {
      entriesBatcherRef.current = createEventBatcher<MonitoringEntry>((batch) => {
        queryClient.setQueryData<MonitoringEntry[]>(monitoringKeys.feed(), (prev) =>
          mergeBatch(prev, batch, maxEntriesRef.current),
        )
      })
    }
    return entriesBatcherRef.current
  }, [queryClient])
  const getStatsBatcher = useCallback(() => {
    if (!statsBatcherRef.current) {
      statsBatcherRef.current = createEventBatcher<MonitoringTokenStats>((batch) => {
        setTokenStats(batch[batch.length - 1])
      })
    }
    return statsBatcherRef.current
  }, [])

  const feedQuery = useQuery({
    queryKey: monitoringKeys.feed(),
    staleTime: Infinity,
    queryFn: async () => {
      const previous = queryClient.getQueryData<MonitoringEntry[]>(monitoringKeys.feed())
      const rawEntries = await fetchMonitoringFeedSnapshot(initialCount)
      const incoming = rawEntries
        .map(normalizeMonitoringEntry)
        .filter((entry): entry is MonitoringEntry => entry !== null)
      return mergeBatch(previous, incoming, maxEntriesRef.current)
    },
  })

  const { refetch: refetchFeed } = feedQuery

  useEffect(() => {
    if (feedQuery.isError) setIsConnected(false)
    else if (feedQuery.isSuccess) setIsConnected(true)
  }, [feedQuery.isError, feedQuery.isSuccess])

  const track = useCallback((pending: Promise<UnlistenFn>) => {
    void pending.then((unlisten) => {
      if (disposedRef.current) unlisten()
      else unlistenersRef.current.push(unlisten)
    })
  }, [])

  const setupNativeListeners = useCallback(async () => {
    if (!isTauriRuntime()) return
    try {
      const unMonitoring = listen<unknown>(MONITORING_EVENT_NAME, (event) => {
        const entry = normalizeMonitoringEntry(event.payload)
        if (entry) getEntriesBatcher().push(entry)
      })
      const unStats = listen<MonitoringTokenStats>('token-stats', (event) => {
        getStatsBatcher().push(event.payload)
      })
      track(unMonitoring)
      track(unStats)
      setIsConnected(true)
    } catch {
      setIsConnected(false)
    }
  }, [getEntriesBatcher, getStatsBatcher, track])

  const stop = useCallback(() => {
    if (!listeningRef.current) return
    listeningRef.current = false
    disposedRef.current = true
    for (const unlisten of unlistenersRef.current) void unlisten()
    unlistenersRef.current = []
    unsubscribeLoggerRef.current?.()
    unsubscribeLoggerRef.current = null
    entriesBatcherRef.current?.dispose()
    entriesBatcherRef.current = null
    statsBatcherRef.current?.dispose()
    statsBatcherRef.current = null
  }, [])

  const start = useCallback(() => {
    if (listeningRef.current) return
    listeningRef.current = true
    disposedRef.current = false
    queryClient.setQueryData<MonitoringEntry[]>(monitoringKeys.feed(), (prev) =>
      mergeBatch(prev, logger.getHistory().map(normalizeLoggerEntry), maxEntriesRef.current),
    )
    unsubscribeLoggerRef.current = logger.subscribe((entry) => {
      getEntriesBatcher().push(normalizeLoggerEntry(entry))
    })
    void refetchFeed()
    void setupNativeListeners()
  }, [getEntriesBatcher, queryClient, refetchFeed, setupNativeListeners])

  useEffect(() => {
    start()
    return stop
  }, [start, stop])

  const clearLogs = useCallback(() => {
    queryClient.setQueryData<MonitoringEntry[]>(monitoringKeys.feed(), [])
  }, [queryClient])

  const refresh = useCallback(() => refetchFeed(), [refetchFeed])

  return {
    isConnected,
    logs: feedQuery.data ?? [],
    tokenStats,
    clearLogs,
    refresh,
    pause: stop,
    resume: start,
  }
}
