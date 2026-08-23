/** 紧凑 token 计数：B / M / k 三档。 */
export function formatTokens(value: number): string {
  if (!Number.isFinite(value) || value <= 0) return '0'
  if (value >= 1e9) return `${(value / 1e9).toFixed(2)}B`
  if (value >= 1e6) return `${(value / 1e6).toFixed(2)}M`
  if (value >= 1e3) return `${(value / 1e3).toFixed(1)}k`
  return value.toLocaleString()
}

/** 美元金额：≥100 取整、≥1 两位小数、否则四位小数。 */
export function formatUsd(value: number): string {
  if (!Number.isFinite(value)) return '$0.00'
  if (value >= 100) return `$${value.toFixed(0)}`
  if (value >= 1) return `$${value.toFixed(2)}`
  return `$${value.toFixed(4)}`
}

/** 洞察面板用更高阈值的美元格式。 */
export function formatInsightUsd(value: number): string {
  if (!Number.isFinite(value)) return '$0.00'
  if (value >= 1000) return `$${value.toFixed(0)}`
  if (value >= 1) return `$${value.toFixed(2)}`
  return `$${value.toFixed(4)}`
}

export function formatRoi(roi: number | null): string {
  if (roi === null || !Number.isFinite(roi)) return '-'
  return `${roi.toFixed(1)}×`
}

export function formatPercent(rate: number): string {
  const pct = Math.max(0, Math.min(1, rate)) * 100
  return `${pct.toFixed(1)}%`
}

export function barPercent(value: number, max: number): number {
  if (max <= 0) return 0
  return Math.max(6, Math.round((value / max) * 100))
}

export function shortenPath(raw: string, limit = 42): string {
  if (!raw) return ''
  if (raw.length <= limit) return raw
  const segments = raw.replace(/\\/g, '/').split('/').filter(Boolean)
  if (segments.length <= 2) return raw
  return `…/${segments.slice(-2).join('/')}`
}

export function shortenId(raw: string): string {
  if (!raw) return ''
  if (raw.length <= 12) return raw
  return `${raw.slice(0, 6)}…${raw.slice(-4)}`
}
