/* eslint-disable no-console -- This is a logger utility, console output is expected */
import { isTauriRuntime } from '@/utils/tauriRuntime'
import { getErrorMessage } from '@/utils/errorHandler'
import { appendFrontendLogs } from '@/api/generated/events'
import { redactLogText, redactLogValue } from '@/utils/logRedact'
import type { FrontendLogInputDto as FrontendLogInput } from '@/types/generated/events/FrontendLogInputDto'
import type { JsonValueDto } from '@/types/generated/events/JsonValueDto'

export type LogLevel = 'debug' | 'info' | 'warn' | 'error'

export interface LoggerEntry {
  id: string
  level: LogLevel
  message: string
  timestamp: string
  source: string
  correlationId: string
  data?: unknown
}

export const MAX_MESSAGE_CHARS = 2000
export const MAX_SOURCE_CHARS = 64
export const MAX_CORR_CHARS = 64
export const MAX_FIELDS_JSON_BYTES = 8192
export const MAX_BRIDGE_QUEUE = 100
export const MAX_BRIDGE_ATTEMPTS = 3

type LoggerListener = (entry: LoggerEntry) => void

export interface LoggerOptions {
  isTauriRuntime?: () => boolean
  appendFrontendLogs?: (entries: FrontendLogInput[]) => Promise<void>
  now?: () => Date
  sessionId?: string
}

let logSequence = 0

const createLogId = (now: Date): string => {
  logSequence += 1
  return `frontend-${now.getTime()}-${logSequence}`
}

const createSessionId = (): string => {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID()
  }
  return `frontend-session-${Date.now()}`
}

const truncateChars = (value: string, maxChars: number): string => {
  return Array.from(value).slice(0, maxChars).join('')
}

const estimateJsonBytes = (value: unknown): number => {
  try {
    return new TextEncoder().encode(JSON.stringify(value)).length
  } catch {
    return MAX_FIELDS_JSON_BYTES + 1
  }
}

export const sanitizeLoggerData = (data: unknown): unknown => {
  if (typeof data === 'undefined') {
    return undefined
  }

  if (data instanceof Error) {
    return redactLogValue({
      name: data.name,
      message: data.message,
    })
  }

  try {
    const cloned = JSON.parse(JSON.stringify(data)) as unknown
    const redacted = redactLogValue(cloned)
    if (estimateJsonBytes(redacted) > MAX_FIELDS_JSON_BYTES) {
      return { truncated: true }
    }
    return redacted
  } catch {
    return redactLogValue({ value: String(data) })
  }
}

const toJsonFields = (data: unknown): JsonValueDto | undefined => {
  if (typeof data === 'undefined') {
    return undefined
  }
  return data as JsonValueDto
}

export class Logger {
  private readonly isDevelopment = import.meta.env.DEV
  private readonly sessionId: string
  private readonly resolveRuntime: () => boolean
  private readonly appendLogs: (entries: FrontendLogInput[]) => Promise<void>
  private readonly now: () => Date
  private logHistory: LoggerEntry[] = []
  private listeners = new Set<LoggerListener>()
  private maxHistorySize = 100
  private nativeBridgeQueue: FrontendLogInput[] = []
  private nativeBridgeTimer: ReturnType<typeof setTimeout> | null = null
  private nativeBridgeInFlight = false
  private nativeBridgeStatus: 'unknown' | 'ready' | 'disabled' = 'unknown'
  private nativeBridgeAttempts = 0

  constructor(options: LoggerOptions = {}) {
    this.sessionId = truncateChars(options.sessionId ?? createSessionId(), MAX_CORR_CHARS)
    this.resolveRuntime = options.isTauriRuntime ?? isTauriRuntime
    this.appendLogs = options.appendFrontendLogs ?? appendFrontendLogs
    this.now = options.now ?? (() => new Date())
  }

  getSessionId(): string {
    return this.sessionId
  }

  getBridgeQueueLength(): number {
    return this.nativeBridgeQueue.length
  }

  private formatMessage(level: LogLevel, message: string, data?: unknown): LoggerEntry {
    const timestamp = this.now().toISOString()
    const redactedMessage = truncateChars(redactLogText(message), MAX_MESSAGE_CHARS)
    return {
      id: createLogId(this.now()),
      level,
      message: redactedMessage,
      timestamp,
      source: 'frontend',
      correlationId: this.sessionId,
      data: sanitizeLoggerData(data),
    }
  }

  private addToHistory(entry: LoggerEntry): void {
    this.logHistory.push(entry)
    if (this.logHistory.length > this.maxHistorySize) {
      this.logHistory.shift()
    }
  }

  private notifyListeners(entry: LoggerEntry): void {
    for (const listener of this.listeners) {
      try {
        listener(entry)
      } catch (error) {
        if (this.isDevelopment) {
          console.warn('[logger] listener failed', error)
        }
      }
    }
  }

  private enqueueNativeBridge(entry: LoggerEntry): void {
    if (!this.resolveRuntime() || this.nativeBridgeStatus === 'disabled') {
      return
    }

    while (this.nativeBridgeQueue.length >= MAX_BRIDGE_QUEUE) {
      this.nativeBridgeQueue.shift()
    }

    this.nativeBridgeQueue.push({
      level: entry.level,
      message: entry.message,
      source: truncateChars(entry.source, MAX_SOURCE_CHARS) || 'frontend',
      timestamp: entry.timestamp,
      correlationId: entry.correlationId,
      fields: toJsonFields(entry.data),
    })

    if (this.nativeBridgeTimer) {
      return
    }

    this.nativeBridgeTimer = setTimeout(() => {
      this.nativeBridgeTimer = null
      void this.flushNativeBridge()
    }, 250)
  }

  private shouldDisableNativeBridge(error: unknown): boolean {
    const message = getErrorMessage(error)
    return /append_frontend_logs/i.test(message) || /unknown command|not found|unsupported/i.test(message)
  }

  async flushNativeBridge(): Promise<void> {
    if (
      this.nativeBridgeInFlight
      || this.nativeBridgeStatus === 'disabled'
      || this.nativeBridgeQueue.length === 0
      || !this.resolveRuntime()
    ) {
      return
    }

    const entries = this.nativeBridgeQueue.splice(0, this.nativeBridgeQueue.length)
    this.nativeBridgeInFlight = true

    try {
      await this.appendLogs(entries)
      this.nativeBridgeStatus = 'ready'
      this.nativeBridgeAttempts = 0
    } catch (error) {
      this.nativeBridgeAttempts += 1
      const unknownCommand = this.shouldDisableNativeBridge(error)
      if (this.nativeBridgeAttempts >= MAX_BRIDGE_ATTEMPTS) {
        this.nativeBridgeAttempts = 0
        if (unknownCommand) {
          this.nativeBridgeStatus = 'disabled'
        }
      } else {
        this.nativeBridgeQueue.unshift(...entries)
        if (!this.nativeBridgeTimer) {
          this.nativeBridgeTimer = setTimeout(() => {
            this.nativeBridgeTimer = null
            void this.flushNativeBridge()
          }, 1000)
        }
      }

      if (this.isDevelopment) {
        console.warn('[logger] native bridge unavailable', error)
      }
    } finally {
      this.nativeBridgeInFlight = false
    }
  }

  private log(level: LogLevel, message: string, data?: unknown): void {
    const entry = this.formatMessage(level, message, data)
    this.addToHistory(entry)
    this.notifyListeners(entry)

    if (level === 'warn' || level === 'error') {
      this.enqueueNativeBridge(entry)
    }

    if (!this.isDevelopment && level === 'debug') {
      return
    }

    const logMethod =
      level === 'warn' ? console.warn : level === 'error' ? console.error : console.log
    const prefix = `[${entry.timestamp}] [${level.toUpperCase()}]`

    if (typeof entry.data !== 'undefined') {
      logMethod(prefix, entry.message, entry.data)
    } else {
      logMethod(prefix, entry.message)
    }
  }

  subscribe(listener: LoggerListener): () => void {
    this.listeners.add(listener)
    return () => {
      this.listeners.delete(listener)
    }
  }

  debug(message: string, data?: unknown): void {
    this.log('debug', message, data)
  }

  info(message: string, data?: unknown): void {
    this.log('info', message, data)
  }

  warn(message: string, data?: unknown): void {
    this.log('warn', message, data)
  }

  error(message: string, data?: unknown): void {
    this.log('error', message, data)
  }

  getHistory(): LoggerEntry[] {
    return [...this.logHistory]
  }

  clearHistory(): void {
    this.logHistory = []
  }
}

export const logger = new Logger()
