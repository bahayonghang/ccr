import { formatRelativeTime, formatTimestamp } from '@/utils/codexHelpers'

export const SESSION_LIMIT = 160
export const DETAIL_LIMIT = 120
export const EXPORT_LIMIT = 200

export function formatTokenCount(value: number): string {
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`
  if (value >= 1_000) return `${(value / 1_000).toFixed(1)}K`
  return String(value)
}

export function formatSessionRelative(value: string | null | undefined, fallback: string): string {
  if (!value) return fallback
  return formatRelativeTime(value)
}

export function formatSessionAbsolute(value: string | null | undefined, fallback: string): string {
  if (!value) return fallback
  return formatTimestamp(value)
}
