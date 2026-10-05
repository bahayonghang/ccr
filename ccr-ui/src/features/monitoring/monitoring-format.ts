import type { TranslateFunction } from '@/utils/tf'
import type { MonitoringLevel } from './monitoring-types'

export function formatWholeNumber(locale: string, value: number): string {
  return new Intl.NumberFormat(locale).format(value)
}

export function formatCompactNumber(locale: string, value: number): string {
  if (Math.abs(value) >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`
  if (Math.abs(value) >= 1_000) return `${(value / 1_000).toFixed(1)}K`
  return formatWholeNumber(locale, value)
}

export function formatCostUsd(value: number): string {
  return `$${value.toFixed(value >= 100 ? 2 : 4)}`
}

export function formatDateTime(locale: string, timestamp: string): string {
  const date = new Date(timestamp)
  if (Number.isNaN(date.getTime())) return timestamp
  return new Intl.DateTimeFormat(locale, {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  }).format(date)
}

export function formatTime(locale: string, timestamp: string): string {
  const date = new Date(timestamp)
  if (Number.isNaN(date.getTime())) return '--:--:--'
  return new Intl.DateTimeFormat(locale, {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
  }).format(date)
}

export function getLevelClass(level: MonitoringLevel): string {
  if (level === 'error') return 'bg-accent-danger/15 text-accent-danger'
  if (level === 'warn') return 'bg-accent-warning/15 text-accent-warning'
  if (level === 'info') return 'bg-accent-primary/15 text-accent-primary'
  return 'bg-text-muted/15 text-text-muted'
}

export function healthStatusOf(counts: Record<MonitoringLevel, number>, total: number): 'critical' | 'attention' | 'healthy' | 'quiet' {
  if (counts.error > 0) return 'critical'
  if (counts.warn > 0) return 'attention'
  if (total > 0) return 'healthy'
  return 'quiet'
}

export function healthStatusLabelOf(status: ReturnType<typeof healthStatusOf>, t: TranslateFunction): string {
  if (status === 'critical') return t('monitoring.healthCritical')
  if (status === 'attention') return t('monitoring.healthAttention')
  if (status === 'healthy') return t('monitoring.healthHealthy')
  return t('monitoring.healthQuiet')
}

export function healthStatusClassOf(status: ReturnType<typeof healthStatusOf>): string {
  if (status === 'critical') return 'border-accent-danger/30 bg-accent-danger/10 text-accent-danger'
  if (status === 'attention') return 'border-accent-warning/30 bg-accent-warning/10 text-accent-warning'
  if (status === 'healthy') return 'border-accent-success/30 bg-accent-success/10 text-accent-success'
  return 'border-border-default/45 bg-bg-elevated text-text-secondary'
}
