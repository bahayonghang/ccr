import { useCallback, useEffect, useRef, useState } from 'react'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getMonitoringFeed, getRecentEvents } from '@/api'
import { logger } from '@/utils/logger'
import { isTauriRuntime } from '@/utils/tauriRuntime'

export interface DashboardSignalEntry {
  id: string
  timestamp: string
  level: string
  channel: string
  eventType: string
  source: string
  message: string
}

interface RecordLike {
  [key: string]: unknown
}

const isRecord = (value: unknown): value is RecordLike =>
  typeof value === 'object' && value !== null

const readString = (record: RecordLike, ...keys: string[]): string | undefined => {
  for (const key of keys) {
    const value = record[key]
    if (typeof value === 'string' && value.trim()) return value
    if (typeof value === 'number' || typeof value === 'boolean') return String(value)
  }
  return undefined
}

const normalizeLevel = (value: unknown): string => {
  if (typeof value !== 'string') return 'info'
  const lowered = value.toLowerCase()
  if (lowered === 'warn' || lowered === 'warning') return 'warn'
  if (lowered === 'error') return 'error'
  if (lowered === 'debug') return 'debug'
  return 'info'
}

const normalizeEntry = (raw: unknown, fallbackPrefix: string): DashboardSignalEntry | null => {
  if (!isRecord(raw)) return null
  const message = readString(raw, 'message')
  if (!message) return null
  return {
    id: readString(raw, 'id') ?? `${fallbackPrefix}-${Date.now()}`,
    timestamp: readString(raw, 'timestamp') ?? new Date().toISOString(),
    level: normalizeLevel(raw.level),
    channel: readString(raw, 'channel') ?? 'system',
    eventType: readString(raw, 'eventType', 'event_type') ?? 'log',
    source: readString(raw, 'source', 'channel') ?? 'tauri',
    message,
  }
}

const mergeSignalEntries = (
  previous: DashboardSignalEntry[],
  incoming: DashboardSignalEntry[],
  limit: number,
): DashboardSignalEntry[] => {
  const keys = new Set(previous.map((entry) => entry.id))
  const next = [...previous]
  incoming.forEach((entry) => {
    if (keys.has(entry.id)) return
    keys.add(entry.id)
    next.push(entry)
  })
  next.sort((left, right) => new Date(left.timestamp).getTime() - new Date(right.timestamp).getTime())
  return next.length > limit ? next.slice(-limit) : next
}

export function useDashboardSignals(limit = 24) {
  const [logs, setLogs] = useState<DashboardSignalEntry[]>([])
  const unlistenersRef = useRef<UnlistenFn[]>([])
  const unsubscribeLoggerRef = useRef<(() => void) | null>(null)

  const mergeEntries = useCallback((incoming: DashboardSignalEntry[]) => {
    setLogs((previous) => mergeSignalEntries(previous, incoming, limit))
  }, [limit])

  const stop = useCallback(() => {
    unlistenersRef.current.forEach((unlisten) => {
      void unlisten()
    })
    unlistenersRef.current = []
    unsubscribeLoggerRef.current?.()
    unsubscribeLoggerRef.current = null
  }, [])

  const start = useCallback(() => {
    const history = logger.getHistory().map((entry) => ({
      id: entry.id,
      timestamp: entry.timestamp,
      level: entry.level,
      channel: 'frontend',
      eventType: `frontend.${entry.level}`,
      source: entry.source,
      message: entry.message,
    }))
    mergeEntries(history)
    unsubscribeLoggerRef.current = logger.subscribe((entry) => {
      mergeEntries([
        {
          id: entry.id,
          timestamp: entry.timestamp,
          level: entry.level,
          channel: 'frontend',
          eventType: `frontend.${entry.level}`,
          source: entry.source,
          message: entry.message,
        },
      ])
    })

    if (!isTauriRuntime()) return

    void loadNativeFeed(mergeEntries, unlistenersRef)
  }, [mergeEntries])

  useEffect(() => {
    start()
    return stop
  }, [start, stop])

  return { logs, pause: stop, resume: start }
}

async function loadNativeFeed(
  mergeEntries: (incoming: DashboardSignalEntry[]) => void,
  unlistenersRef: { current: UnlistenFn[] },
) {
  try {
    const snapshot = await getMonitoringFeed({ count: 6 })
    mergeEntries(
      snapshot
        .map((item) => normalizeEntry(item, 'monitoring'))
        .filter((item): item is DashboardSignalEntry => item !== null),
    )
  } catch {
    try {
      const fallback = await getRecentEvents(6)
      mergeEntries(
        fallback
          .map((item) => normalizeEntry(item, 'legacy'))
          .filter((item): item is DashboardSignalEntry => item !== null),
      )
    } catch {
      mergeEntries([])
    }
  }

  try {
    const unlisten = await listen<unknown>('app:monitoring', (event) => {
      const entry = normalizeEntry(event.payload, 'monitoring')
      if (entry) mergeEntries([entry])
    })
    unlistenersRef.current.push(unlisten)
  } catch {
    // 非原生或监听失败时只保留已有快照。
  }
}
