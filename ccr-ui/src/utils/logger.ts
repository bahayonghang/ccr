/* eslint-disable no-console -- This is a logger utility, console output is expected */

type LogLevel = 'debug' | 'info' | 'warn' | 'error'

interface LogEntry {
  level: LogLevel
  message: string
  timestamp: string
  data?: unknown
}

class Logger {
  private isDevelopment = import.meta.env.DEV
  private logHistory: LogEntry[] = []
  private maxHistorySize = 100

  private formatMessage(level: LogLevel, message: string, data?: unknown): LogEntry {
    return {
      level,
      message,
      timestamp: new Date().toISOString(),
      data,
    }
  }

  private addToHistory(entry: LogEntry): void {
    this.logHistory.push(entry)
    if (this.logHistory.length > this.maxHistorySize) {
      this.logHistory.shift()
    }
  }

  private log(level: LogLevel, message: string, data?: unknown): void {
    const entry = this.formatMessage(level, message, data)
    this.addToHistory(entry)

    if (!this.isDevelopment && level === 'debug') {
      return
    }

    const logMethod =
      level === 'warn' ? console.warn : level === 'error' ? console.error : console.log
    const prefix = `[${entry.timestamp}] [${level.toUpperCase()}]`

    if (data) {
      logMethod(prefix, message, data)
    } else {
      logMethod(prefix, message)
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

  getHistory(): LogEntry[] {
    return [...this.logHistory]
  }

  clearHistory(): void {
    this.logHistory = []
  }
}

export const logger = new Logger()
