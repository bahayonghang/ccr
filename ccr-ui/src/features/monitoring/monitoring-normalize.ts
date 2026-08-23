import type { UnknownRecord } from '@/types/common'
import type { LoggerEntry } from '@/utils/logger'
import type { MonitoringEntry, MonitoringLevel } from './monitoring-types'

let fallbackSequence = 0

const isRecord = (value: unknown): value is UnknownRecord => {
  return typeof value === 'object' && value !== null
}

const readString = (record: UnknownRecord, ...keys: string[]): string | undefined => {
  for (const key of keys) {
    const value = record[key]
    if (typeof value === 'string' && value.trim().length > 0) return value
    if (typeof value === 'number' || typeof value === 'boolean') return String(value)
  }
  return undefined
}

const createFallbackId = (prefix: string): string => {
  fallbackSequence += 1
  return `${prefix}-${Date.now()}-${fallbackSequence}`
}

const normalizeLevel = (value: unknown): MonitoringLevel => {
  if (typeof value !== 'string') return 'info'
  const lower = value.toLowerCase()
  if (lower === 'debug') return 'debug'
  if (lower === 'warn' || lower === 'warning') return 'warn'
  if (lower === 'error') return 'error'
  return 'info'
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

const channelOfType = (type: string): string => {
  if (type.startsWith('Checkin')) return 'checkin'
  if (type.startsWith('Usage')) return 'usage'
  if (type.startsWith('Environment')) return 'environment'
  if (type.startsWith('Sync')) return 'sync'
  if (type.startsWith('Task')) return 'task'
  return 'app'
}

const checkinMessage = (type: string, data: UnknownRecord): string => {
  return readString(data, 'message') ?? `Checkin ${type === 'CheckinCompleted' ? 'completed' : 'failed'}`
}

const notificationMessage = (data: UnknownRecord): string => {
  const title = readString(data, 'title')
  const message = readString(data, 'message') ?? 'Notification received'
  return title ? `${title}: ${message}` : message
}

const environmentMessage = (data: UnknownRecord): string => {
  if (!readString(data, 'status')) return 'Environment changed'
  return `Environment ${readString(data, 'env_id', 'envId') ?? 'unknown'} ${readString(data, 'status')}`
}

const usageImportMessage = (data: UnknownRecord): string => {
  const importedCount = readString(data, 'imported_count', 'importedCount') ?? '0'
  const platform = readString(data, 'platform') ?? 'unknown'
  return `Imported ${importedCount} usage records for ${platform}`
}

const buildLegacyMessage = (type: string, data: UnknownRecord): string => {
  if (type === 'CheckinCompleted' || type === 'CheckinFailed') return checkinMessage(type, data)
  if (type === 'SyncStatusChanged') return readString(data, 'message') ?? 'Sync status changed'
  if (type === 'TaskProgress') return readString(data, 'message') ?? 'Task progress updated'
  if (type === 'Notification') return notificationMessage(data)
  if (type === 'EnvironmentChanged') return environmentMessage(data)
  if (type === 'UsageImportCompleted') return usageImportMessage(data)
  return readString(data, 'message') ?? type
}

const normalizeLegacyEvent = (record: UnknownRecord): MonitoringEntry | null => {
  const event = isRecord(record.event) ? record.event : null
  const type = event ? readString(event, 'type') : undefined
  if (!type) return null
  const data = event && isRecord(event.data) ? event.data : {}
  const level =
    type === 'CheckinFailed' ? 'error' : type === 'Notification' ? normalizeLevel(readString(data, 'level')) : 'info'
  return {
    id: readString(record, 'id') ?? createFallbackId('legacy-monitoring'),
    timestamp: readString(record, 'timestamp') ?? new Date().toISOString(),
    level,
    channel: channelOfType(type),
    eventType: toEventType(type),
    source: normalizeSource(data),
    message: buildLegacyMessage(type, data),
    fields: Object.keys(data).length > 0 ? data : undefined,
  }
}

export const normalizeMonitoringEntry = (raw: unknown): MonitoringEntry | null => {
  if (!isRecord(raw)) return null
  const legacyEntry = normalizeLegacyEvent(raw)
  if (legacyEntry) return legacyEntry
  const message = readString(raw, 'message')
  if (!message) return null
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

export const normalizeLoggerEntry = (entry: LoggerEntry): MonitoringEntry => {
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

export const buildEntryKey = (entry: MonitoringEntry): string => {
  return [entry.timestamp, entry.level, entry.channel, entry.eventType, entry.source, entry.message].join('|')
}

const compareEntriesByTimestamp = (left: MonitoringEntry, right: MonitoringEntry): number => {
  return new Date(left.timestamp).getTime() - new Date(right.timestamp).getTime()
}

const trimEntries = (entries: MonitoringEntry[], maxEntries: number): MonitoringEntry[] => {
  return entries.length > maxEntries ? entries.slice(-maxEntries) : entries
}

const insertEntryByTimestamp = (entries: MonitoringEntry[], entry: MonitoringEntry): MonitoringEntry[] => {
  if (entries.length === 0) return [entry]
  const lastEntry = entries[entries.length - 1]
  if (compareEntriesByTimestamp(lastEntry, entry) <= 0) return [...entries, entry]
  const nextEntries = [...entries]
  let low = 0
  let high = nextEntries.length
  while (low < high) {
    const mid = Math.floor((low + high) / 2)
    if (compareEntriesByTimestamp(nextEntries[mid], entry) <= 0) low = mid + 1
    else high = mid
  }
  nextEntries.splice(low, 0, entry)
  return nextEntries
}

export function mergeBatch(
  prev: MonitoringEntry[] | undefined,
  batch: MonitoringEntry[],
  maxEntries: number,
): MonitoringEntry[] {
  const base = prev ?? []
  const existingKeys = new Set(base.map(buildEntryKey))
  let merged = base
  for (const entry of batch) {
    const entryKey = buildEntryKey(entry)
    if (existingKeys.has(entryKey)) continue
    existingKeys.add(entryKey)
    merged = insertEntryByTimestamp(merged, entry)
  }
  return trimEntries(merged, maxEntries)
}
