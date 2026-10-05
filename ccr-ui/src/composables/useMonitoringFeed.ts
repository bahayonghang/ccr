import { useQuery, useQueryClient } from '@tanstack/react-query'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useCallback, useEffect, useRef, useState } from 'react'
import type { UnknownRecord } from '@/types/common'
import {
  fetchMonitoringFeedSnapshot,
  monitoringKeys,
} from '@/features/monitoring/queries'
import { createEventBatcher, type EventBatcher } from '@/shell/eventBridge'
import { logger, type LoggerEntry, type LogLevel } from '@/utils/logger'
import { isTauriRuntime } from '@/utils/tauriRuntime'

// Monitoring feed 的 React 迁移（08-22-state-logic-port 批次 5b-ii）。
// 原 per-instance shallowRef 缓冲改为 Query 缓存（monitoringKeys.feed），事件写入按
// state-logic-port 批次 3 约定走「ref 累积 + 定时批量 setQueryData」：
// - app:monitoring 条目与前端 logger 条目 → createEventBatcher（250ms，eventBridge 常量）
//   批量提交；token-stats 为替换语义（保留最新值），同用 batcher 以合并渲染；
// - 初始快照经 fetchMonitoringFeedSnapshot（getMonitoringFeed → 回退 getRecentEvents），
//   失败时 query 进入 error → isConnected 置 false（原 loadInitialFeed 双失败语义）。
//
// 语义说明（相对 Vue 版的差异登记）：
// - Query 缓存为共享存储：多个消费者（MonitoringView/DashboardView）挂载各自独立实例
//   （各自的监听器、batcher、logger 订阅与 start/pause/resume 生命周期），但条目缓冲
//   共享同一 queryKey —— maxEntries 取各实例传入值，后 flush 者生效；
// - 去重键改为「与当前缓存内容比对」（原 seenEntries Set 在 trim 后同样重建，
//   行为等价：仍在缓冲内的条目不重复进入，被裁剪的条目可重新进入）；
// - isConnected 由快照 query 成败与原生监听安装结果共同派生（原单字段的两路赋值点
//   一一对应）；监听回调内逐条的 isConnected.value = true 冗余赋值省去。
//
// 签名变化（消费方均为待迁移 .vue 视图）：Ref<T> → 普通值。

export type MonitoringLevel = LogLevel

export interface MonitoringEntry {
  id: string
  timestamp: string
  level: MonitoringLevel
  channel: string
  eventType: string
  source: string
  message: string
  correlationId?: string | null
  fields?: unknown
}

export interface MonitoringTokenStats {
  input_tokens: number
  output_tokens: number
  cache_tokens: number
  request_count: number
  estimated_cost_cents: number
  last_updated: string
}

interface MonitoringFeedOptions {
  initialCount?: number
  maxEntries?: number
}

const MONITORING_EVENT_NAME = 'app:monitoring'
const DEFAULT_INITIAL_COUNT = 100
const DEFAULT_MAX_ENTRIES = 500

let fallbackSequence = 0

const isRecord = (value: unknown): value is UnknownRecord => {
  return typeof value === 'object' && value !== null
}

const readString = (record: UnknownRecord, ...keys: string[]): string | undefined => {
  for (const key of keys) {
    const value = record[key]
    if (typeof value === 'string' && value.trim().length > 0) {
      return value
    }
    if (typeof value === 'number' || typeof value === 'boolean') {
      return String(value)
    }
  }
  return undefined
}

const createFallbackId = (prefix: string): string => {
  fallbackSequence += 1
  return `${prefix}-${Date.now()}-${fallbackSequence}`
}

const normalizeLevel = (value: unknown): MonitoringLevel => {
  if (typeof value !== 'string') {
    return 'info'
  }

  switch (value.toLowerCase()) {
    case 'debug':
      return 'debug'
    case 'warn':
    case 'warning':
      return 'warn'
    case 'error':
      return 'error'
    default:
      return 'info'
  }
}

const toEventType = (value: string): string => {
  return value
    .replace(/([a-z0-9])([A-Z])/g, '$1.$2')
    .replace(/[_\s-]+/g, '.')
    .toLowerCase()
}

const normalizeSource = (record: UnknownRecord, fallback = 'tauri'): string => {
  return readString(record, 'source', 'channel') ?? fallback
}

const buildLegacyMessage = (type: string, data: UnknownRecord): string => {
  switch (type) {
    case 'CheckinCompleted':
    case 'CheckinFailed':
      return (
        readString(data, 'message') ??
        `Checkin ${type === 'CheckinCompleted' ? 'completed' : 'failed'}`
      )
    case 'SyncStatusChanged':
      return readString(data, 'message') ?? 'Sync status changed'
    case 'TaskProgress':
      return readString(data, 'message') ?? 'Task progress updated'
    case 'Notification': {
      const title = readString(data, 'title')
      const message = readString(data, 'message') ?? 'Notification received'
      return title ? `${title}: ${message}` : message
    }
    case 'EnvironmentChanged':
      return readString(data, 'status')
        ? `Environment ${readString(data, 'env_id', 'envId') ?? 'unknown'} ${readString(data, 'status')}`
        : 'Environment changed'
    case 'UsageImportCompleted': {
      const importedCount = readString(data, 'imported_count', 'importedCount') ?? '0'
      const platform = readString(data, 'platform') ?? 'unknown'
      return `Imported ${importedCount} usage records for ${platform}`
    }
    default:
      return readString(data, 'message') ?? type
  }
}

const normalizeLegacyEvent = (record: UnknownRecord): MonitoringEntry | null => {
  const event = isRecord(record.event) ? record.event : null
  const type = event ? readString(event, 'type') : undefined
  if (!type) {
    return null
  }

  const data = event && isRecord(event.data) ? event.data : {}
  const level =
    type === 'CheckinFailed'
      ? 'error'
      : type === 'Notification'
        ? normalizeLevel(readString(data, 'level'))
        : 'info'

  return {
    id: readString(record, 'id') ?? createFallbackId('legacy-monitoring'),
    timestamp: readString(record, 'timestamp') ?? new Date().toISOString(),
    level,
    channel: type.startsWith('Checkin')
      ? 'checkin'
      : type.startsWith('Usage')
        ? 'usage'
        : type.startsWith('Environment')
          ? 'environment'
          : type.startsWith('Sync')
            ? 'sync'
            : type.startsWith('Task')
              ? 'task'
              : 'app',
    eventType: toEventType(type),
    source: normalizeSource(data),
    message: buildLegacyMessage(type, data),
    fields: Object.keys(data).length > 0 ? data : undefined,
  }
}

const normalizeMonitoringEntry = (raw: unknown): MonitoringEntry | null => {
  if (!isRecord(raw)) {
    return null
  }

  const legacyEntry = normalizeLegacyEvent(raw)
  if (legacyEntry) {
    return legacyEntry
  }

  const message = readString(raw, 'message')
  if (!message) {
    return null
  }

  return {
    id: readString(raw, 'id') ?? createFallbackId('monitoring'),
    timestamp: readString(raw, 'timestamp') ?? new Date().toISOString(),
    level: normalizeLevel(raw.level),
    channel: readString(raw, 'channel') ?? 'system',
    eventType: readString(raw, 'eventType', 'event_type') ?? 'log',
    source: normalizeSource(raw),
    message,
    correlationId: readString(raw, 'correlationId', 'correlation_id') ?? null,
    fields: raw.fields ?? raw.metadata,
  }
}

const normalizeLoggerEntry = (entry: LoggerEntry): MonitoringEntry => {
  return {
    id: entry.id,
    timestamp: entry.timestamp,
    level: entry.level,
    channel: 'frontend',
    eventType: `frontend.${entry.level}`,
    source: entry.source,
    message: entry.message,
    correlationId: entry.correlationId,
    fields: entry.data,
  }
}

const buildEntryKey = (entry: MonitoringEntry): string => {
  return [
    entry.timestamp,
    entry.level,
    entry.channel,
    entry.eventType,
    entry.source,
    entry.message,
  ].join('|')
}

const compareEntriesByTimestamp = (left: MonitoringEntry, right: MonitoringEntry): number => {
  return new Date(left.timestamp).getTime() - new Date(right.timestamp).getTime()
}

const trimEntries = (entries: MonitoringEntry[], maxEntries: number): MonitoringEntry[] => {
  return entries.length > maxEntries ? entries.slice(-maxEntries) : entries
}

const insertEntryByTimestamp = (
  entries: MonitoringEntry[],
  entry: MonitoringEntry
): MonitoringEntry[] => {
  if (entries.length === 0) {
    return [entry]
  }

  const lastEntry = entries[entries.length - 1]
  if (compareEntriesByTimestamp(lastEntry, entry) <= 0) {
    return [...entries, entry]
  }


  // 二分插入到拷贝上（原 nextEntries.splice 就地写法已随不可变迁移改写，
  // 见 mutation-rewrite.md 对应行）。
  const nextEntries = [...entries]
  let low = 0
  let high = nextEntries.length

  while (low < high) {
    const mid = Math.floor((low + high) / 2)
    if (compareEntriesByTimestamp(nextEntries[mid], entry) <= 0) {
      low = mid + 1
    } else {
      high = mid
    }
  }

  nextEntries.splice(low, 0, entry)
  return nextEntries
}

/** 批量合入：按键去重 + 按时间戳插入 + 裁剪（原 mergeEntries 的纯函数化）。 */
function mergeBatch(prev: MonitoringEntry[] | undefined, batch: MonitoringEntry[], maxEntries: number): MonitoringEntry[] {
  const base = prev ?? []
  const existingKeys = new Set(base.map(buildEntryKey))
  let merged = base

  for (const entry of batch) {
    const entryKey = buildEntryKey(entry)
    if (existingKeys.has(entryKey)) {
      continue
    }
    existingKeys.add(entryKey)
    merged = insertEntryByTimestamp(merged, entry)
  }

  return trimEntries(merged, maxEntries)
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

  /** 高频事件批量提交器（懒建；stop 时 dispose 并置空）。 */
  const entriesBatcherRef = useRef<EventBatcher<MonitoringEntry> | null>(null)
  const statsBatcherRef = useRef<EventBatcher<MonitoringTokenStats> | null>(null)
  const getEntriesBatcher = useCallback(() => {
    if (!entriesBatcherRef.current) {
      entriesBatcherRef.current = createEventBatcher<MonitoringEntry>((batch) => {
        queryClient.setQueryData<MonitoringEntry[]>(monitoringKeys.feed(), (prev) =>
          mergeBatch(prev, batch, maxEntriesRef.current)
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

  // isConnected 与快照成败联动（原 loadInitialFeed 成功/双失败的赋值点）。
  useEffect(() => {
    if (feedQuery.isError) {
      setIsConnected(false)
    } else if (feedQuery.isSuccess) {
      setIsConnected(true)
    }
  }, [feedQuery.isError, feedQuery.isSuccess])

  const track = useCallback((pending: Promise<UnlistenFn>) => {
    void pending.then((unlisten) => {
      if (disposedRef.current) {
        unlisten()
      } else {
        unlistenersRef.current.push(unlisten)
      }
    })
  }, [])

  const setupNativeListeners = useCallback(async () => {
    if (!isTauriRuntime()) {
      return
    }

    try {
      const unMonitoring = listen<unknown>(MONITORING_EVENT_NAME, (event) => {
        const entry = normalizeMonitoringEntry(event.payload)
        if (entry) {
          getEntriesBatcher().push(entry)
        }
      })

      const unStats = listen<MonitoringTokenStats>('token-stats', (event) => {
        getStatsBatcher().push(event.payload)
      })

      // 取消协议：cleanup 已跑过时迟到的 unlisten 立即调用（eventBridge 同款协议）。
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
    for (const unlisten of unlistenersRef.current) {
      void unlisten()
    }
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

    // logger 历史直接合入缓存；新条目进批量缓冲（原 mergeEntries(getHistory()) + subscribe）。
    queryClient.setQueryData<MonitoringEntry[]>(monitoringKeys.feed(), (prev) =>
      mergeBatch(
        prev,
        logger.getHistory().map(normalizeLoggerEntry),
        maxEntriesRef.current
      )
    )
    unsubscribeLoggerRef.current = logger.subscribe((entry) => {
      getEntriesBatcher().push(normalizeLoggerEntry(entry))
    })

    void refetchFeed()
    void setupNativeListeners()
  }, [getEntriesBatcher, queryClient, refetchFeed, setupNativeListeners])

  // start/stop 幂等：缓存视图可通过 pause/resume 在后台页间断开/重连事件源，
  // 避免切走后仍在后台持续合并事件；重复调用 start 不会重复挂监听。
  useEffect(() => {
    start()
    return stop
  }, [start, stop])

  const clearLogs = useCallback(() => {
    queryClient.setQueryData<MonitoringEntry[]>(monitoringKeys.feed(), [])
  }, [queryClient])

  const refresh = useCallback(() => {
    return refetchFeed()
  }, [refetchFeed])

  return {
    isConnected,
    logs: feedQuery.data ?? [],
    tokenStats,
    clearLogs,
    refresh,
    /** 暂停事件消费（缓存视图后台态调用） */
    pause: stop,
    /** 恢复事件消费并重新拉取初始快照（缓存视图回前台调用） */
    resume: start,
  }
}
