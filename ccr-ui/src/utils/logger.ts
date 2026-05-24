/* eslint-disable no-console -- This is a logger utility, console output is expected */
import { isTauriRuntime } from '@/utils/tauriRuntime'

export type LogLevel = 'debug' | 'info' | 'warn' | 'error'

export interface LoggerEntry {
  id: string
  level: LogLevel
  message: string
  timestamp: string
  source: string
  data?: unknown
}

interface FrontendLogInput {
  level: LogLevel
  message: string
  source: string
  timestamp?: string
  fields?: unknown
}

type LoggerListener = (entry: LoggerEntry) => void

let logSequence = 0

const createLogId = (): string => {
  logSequence += 1
  return `frontend-${Date.now()}-${logSequence}`
}

class Logger {
  private isDevelopment = import.meta.env.DEV
  private logHistory: LoggerEntry[] = []
  private listeners = new Set<LoggerListener>()
  private maxHistorySize = 100
  private nativeBridgeQueue: FrontendLogInput[] = []
  private nativeBridgeTimer: ReturnType<typeof setTimeout> | null = null
  private nativeBridgeInFlight = false
  private nativeBridgeStatus: 'unknown' | 'ready' | 'disabled' = 'unknown'

  private formatMessage(level: LogLevel, message: string, data?: unknown): LoggerEntry {
    return {
      id: createLogId(),
      level,
      message,
      timestamp: new Date().toISOString(),
      source: 'frontend',
      data,
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

  private normalizeFields(data: unknown): unknown {
    if (typeof data === 'undefined') {
      return undefined
    }

    if (data instanceof Error) {
      return {
        name: data.name,
        message: data.message,
        stack: data.stack,
      }
    }

    try {
      return JSON.parse(JSON.stringify(data))
    } catch {
      return { value: String(data) }
    }
  }

  private enqueueNativeBridge(entry: LoggerEntry): void {
    if (!isTauriRuntime() || this.nativeBridgeStatus === 'disabled') {
      return
    }

    this.nativeBridgeQueue.push({
      level: entry.level,
      message: entry.message,
      source: entry.source,
      timestamp: entry.timestamp,
      fields: this.normalizeFields(entry.data),
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
    const message = error instanceof Error ? error.message : String(error)
    return /append_frontend_logs/i.test(message) || /unknown command|not found|unsupported/i.test(message)
  }

  private async flushNativeBridge(): Promise<void> {
    if (
      this.nativeBridgeInFlight
      || this.nativeBridgeStatus === 'disabled'
      || this.nativeBridgeQueue.length === 0
      || !isTauriRuntime()
    ) {
      return
    }

    const entries = this.nativeBridgeQueue.splice(0, this.nativeBridgeQueue.length)
    this.nativeBridgeInFlight = true

    try {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('append_frontend_logs', { entries })
      this.nativeBridgeStatus = 'ready'
    } catch (error) {
      if (this.shouldDisableNativeBridge(error)) {
        this.nativeBridgeStatus = 'disabled'
        this.nativeBridgeQueue = []
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

    if (typeof data !== 'undefined') {
      logMethod(prefix, message, data)
    } else {
      logMethod(prefix, message)
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
