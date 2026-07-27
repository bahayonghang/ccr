import { onMounted, onUnmounted, ref, shallowRef, type Ref } from 'vue'
import type { UnknownRecord } from '@/types/common'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getMonitoringFeed, getRecentEvents, type MonitoringFeedQuery } from '@/api'
import { logger, type LoggerEntry, type LogLevel } from '@/utils/logger'
import { isTauriRuntime } from '@/utils/tauriRuntime'

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

export function useMonitoringFeed(options: MonitoringFeedOptions = {}) {
  const { initialCount = DEFAULT_INITIAL_COUNT, maxEntries = DEFAULT_MAX_ENTRIES } = options

  const isConnected: Ref<boolean> = ref(true)
  const logs = shallowRef<MonitoringEntry[]>([])
  const tokenStats: Ref<MonitoringTokenStats | null> = ref(null)

  const seenEntries = new Set<string>()
  const unlisteners: UnlistenFn[] = []
  let unsubscribeLogger: (() => void) | null = null
  let listening = false

  const mergeEntries = (entries: MonitoringEntry[]) => {
    if (entries.length === 0) {
      return
    }

    let merged = logs.value
    for (const entry of entries) {
      const entryKey = buildEntryKey(entry)
      if (seenEntries.has(entryKey)) {
        continue
      }

      seenEntries.add(entryKey)
      merged = insertEntryByTimestamp(merged, entry)
    }

    const trimmed = trimEntries(merged, maxEntries)
    logs.value = trimmed

    if (trimmed !== merged) {
      seenEntries.clear()
      for (const entry of trimmed) {
        seenEntries.add(buildEntryKey(entry))
      }
    }
  }

  const clearLogs = () => {
    logs.value = []
    seenEntries.clear()
  }

  const loadInitialFeed = async () => {
    if (!isTauriRuntime()) {
      return
    }

    const query: MonitoringFeedQuery = { count: initialCount }

    try {
      const entries = await getMonitoringFeed(query)
      mergeEntries(
        entries
          .map(normalizeMonitoringEntry)
          .filter((entry): entry is MonitoringEntry => entry !== null)
      )
      isConnected.value = true
      return
    } catch {
      // Fall through to legacy event feed while backend migration is in progress.
    }

    try {
      const entries = await getRecentEvents(initialCount)
      mergeEntries(
        entries
          .map(normalizeMonitoringEntry)
          .filter((entry): entry is MonitoringEntry => entry !== null)
      )
      isConnected.value = true
    } catch {
      isConnected.value = false
    }
  }

  const setupNativeListeners = async () => {
    if (!isTauriRuntime()) {
      return
    }

    try {
      const unMonitoring = await listen<unknown>(MONITORING_EVENT_NAME, (event) => {
        const entry = normalizeMonitoringEntry(event.payload)
        if (entry) {
          mergeEntries([entry])
          isConnected.value = true
        }
      })
      unlisteners.push(unMonitoring)

      const unStats = await listen<MonitoringTokenStats>('token-stats', (event) => {
        tokenStats.value = event.payload
      })
      unlisteners.push(unStats)

      isConnected.value = true
    } catch {
      isConnected.value = false
    }
  }

  onMounted(() => {
    start()
  })

  onUnmounted(() => {
    stop()
  })

  // start/stop 幂等：缓存视图（keep-alive）可通过 pause/resume 在 deactivated/activated
  // 间断开/重连事件源，避免切走后仍在后台持续合并事件；重复调用 start 不会重复挂监听。
  function start() {
    if (listening) return
    listening = true
    mergeEntries(logger.getHistory().map(normalizeLoggerEntry))
    unsubscribeLogger = logger.subscribe((entry) => {
      mergeEntries([normalizeLoggerEntry(entry)])
    })

    void loadInitialFeed()
    void setupNativeListeners()
  }

  function stop() {
    if (!listening) return
    listening = false
    for (const unlisten of unlisteners) {
      void unlisten()
    }
    unlisteners.length = 0

    unsubscribeLogger?.()
    unsubscribeLogger = null
  }

  return {
    isConnected,
    logs,
    tokenStats,
    clearLogs,
    refresh: loadInitialFeed,
    /** 暂停事件消费（缓存视图 onDeactivated 调用） */
    pause: stop,
    /** 恢复事件消费并重新拉取初始快照（缓存视图 onActivated 调用） */
    resume: start,
  }
}
