import type { DailyTrend, ModelStat } from '@/types/usage'

export type TrendGranularity = 'day' | 'week' | 'month'

export interface UsageTrendBucket {
  id: string
  startDate: string
  endDate: string
  requestCount: number
  inputTokens: number
  outputTokens: number
  totalTokens: number
  cacheReadTokens: number
  cacheCreationTokens: number
  costUsd: number
}

export interface ModelDistributionSlice {
  id: string
  label: string
  totalCost: number
  totalTokens: number
  requestCount: number
  share: number
  childCount: number
  isOther: boolean
}

const DAY_MS = 86_400_000
const modelCost = (model: ModelStat) => model.cost_with_cache ?? 0

const parseUtcDate = (value: string) => {
  const [year, month, day] = value.split('-').map(Number)
  return new Date(Date.UTC(year, (month || 1) - 1, day || 1))
}

const formatUtcDate = (date: Date) => date.toISOString().slice(0, 10)

const startOfWeekUtc = (date: Date) => {
  const normalized = new Date(Date.UTC(date.getUTCFullYear(), date.getUTCMonth(), date.getUTCDate()))
  const day = normalized.getUTCDay()
  const diff = day === 0 ? -6 : 1 - day
  normalized.setUTCDate(normalized.getUTCDate() + diff)
  return normalized
}

const startOfMonthUtc = (date: Date) =>
  new Date(Date.UTC(date.getUTCFullYear(), date.getUTCMonth(), 1))

const toBucketKey = (date: string, granularity: TrendGranularity) => {
  const parsed = parseUtcDate(date)

  if (granularity === 'month') {
    return formatUtcDate(startOfMonthUtc(parsed))
  }

  if (granularity === 'week') {
    return formatUtcDate(startOfWeekUtc(parsed))
  }

  return formatUtcDate(parsed)
}

export const selectTrendGranularity = (days: number): TrendGranularity => {
  if (days >= 365) return 'month'
  if (days >= 90) return 'week'
  return 'day'
}

export const aggregateDailyTrends = (
  trends: DailyTrend[],
  granularity: TrendGranularity,
): UsageTrendBucket[] => {
  const buckets = new Map<string, UsageTrendBucket>()

  for (const item of trends) {
    const key = toBucketKey(item.date, granularity)
    const existing = buckets.get(key)

    if (existing) {
      existing.endDate = item.date > existing.endDate ? item.date : existing.endDate
      existing.requestCount += item.request_count
      existing.totalTokens += item.total_tokens
      existing.inputTokens += item.input_tokens
      existing.outputTokens += item.output_tokens
      existing.cacheReadTokens += item.cache_read_tokens
      existing.cacheCreationTokens += item.cache_creation_tokens
      existing.costUsd += item.cost_usd
      continue
    }

    buckets.set(key, {
      id: key,
      startDate: key,
      endDate: item.date,
      requestCount: item.request_count,
      totalTokens: item.total_tokens,
      inputTokens: item.input_tokens,
      outputTokens: item.output_tokens,
      cacheReadTokens: item.cache_read_tokens,
      cacheCreationTokens: item.cache_creation_tokens,
      costUsd: item.cost_usd,
    })
  }

  return Array.from(buckets.values()).sort((left, right) => left.startDate.localeCompare(right.startDate))
}

export const expandTrendBucketEnd = (bucket: UsageTrendBucket, granularity: TrendGranularity) => {
  if (bucket.endDate !== bucket.startDate) return bucket.endDate

  const start = parseUtcDate(bucket.startDate)

  if (granularity === 'day') {
    return bucket.startDate
  }

  if (granularity === 'week') {
    return formatUtcDate(new Date(start.getTime() + DAY_MS * 6))
  }

  return formatUtcDate(new Date(Date.UTC(start.getUTCFullYear(), start.getUTCMonth() + 1, 0)))
}

export const groupModelDistribution = (
  modelStats: ModelStat[],
  maxVisible = 6,
): ModelDistributionSlice[] => {
  const sorted = [...modelStats].sort((left, right) => {
    if (modelCost(right) !== modelCost(left)) {
      return modelCost(right) - modelCost(left)
    }
    if (right.total_tokens !== left.total_tokens) {
      return right.total_tokens - left.total_tokens
    }
    return right.request_count - left.request_count
  })

  const totalCost = sorted.reduce((sum, item) => sum + modelCost(item), 0)
  const visible = sorted.slice(0, maxVisible)
  const hidden = sorted.slice(maxVisible)

  const slices = visible.map((item) => ({
    id: item.model,
    label: item.model,
    totalCost: modelCost(item),
    totalTokens: item.total_tokens,
    requestCount: item.request_count,
    share: totalCost > 0 ? modelCost(item) / totalCost : 0,
    childCount: 1,
    isOther: false,
  }))

  if (hidden.length > 0) {
    const otherCost = hidden.reduce((sum, item) => sum + modelCost(item), 0)
    const otherTokens = hidden.reduce((sum, item) => sum + item.total_tokens, 0)
    const otherRequests = hidden.reduce((sum, item) => sum + item.request_count, 0)

    slices.push({
      id: 'others',
      label: 'Others',
      totalCost: otherCost,
      totalTokens: otherTokens,
      requestCount: otherRequests,
      share: totalCost > 0 ? otherCost / totalCost : 0,
      childCount: hidden.length,
      isOther: true,
    })
  }

  return slices
}
