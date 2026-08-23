import type { LogLevel } from '@/utils/logger'

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

export const DEFAULT_INITIAL_COUNT = 100
export const DEFAULT_MAX_ENTRIES = 500
export const MONITORING_EVENT_NAME = 'app:monitoring'
