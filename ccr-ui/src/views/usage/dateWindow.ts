export const formatLocalDate = (date: Date) => {
  const year = date.getFullYear()
  const month = `${date.getMonth() + 1}`.padStart(2, '0')
  const day = `${date.getDate()}`.padStart(2, '0')
  return `${year}-${month}-${day}`
}

export type UsageRangePreset = 'today' | 'this_week' | 'this_month' | 'last_30d' | 'all_time'

export type UsageDateWindow = {
  start?: string
  end?: string
}

export const DEFAULT_USAGE_RANGE_PRESET: UsageRangePreset = 'last_30d'

const startOfLocalDay = (date: Date) =>
  new Date(date.getFullYear(), date.getMonth(), date.getDate())

const getInclusiveLocalDaySpan = (start: Date, end: Date) =>
  Math.max(1, Math.floor((startOfLocalDay(end).getTime() - startOfLocalDay(start).getTime()) / 86_400_000) + 1)

const startOfLocalWeek = (date: Date) => {
  const start = startOfLocalDay(date)
  const day = start.getDay()
  const diff = day === 0 ? -6 : 1 - day
  start.setDate(start.getDate() + diff)
  return start
}

const startOfLocalMonth = (date: Date) =>
  new Date(date.getFullYear(), date.getMonth(), 1)

export const getLocalDateWindow = (days: number, endDate = new Date()): UsageDateWindow => {
  const normalizedDays = Math.max(1, Math.floor(days))
  const end = startOfLocalDay(endDate)
  const start = new Date(end)
  start.setDate(end.getDate() - (normalizedDays - 1))
  return { start: formatLocalDate(start), end: formatLocalDate(end) }
}

export const getLocalDateRangeWindow = (
  preset: UsageRangePreset,
  endDate = new Date(),
): UsageDateWindow => {
  const end = startOfLocalDay(endDate)

  if (preset === 'all_time') {
    return {}
  }

  if (preset === 'today') {
    return { start: formatLocalDate(end), end: formatLocalDate(end) }
  }

  if (preset === 'this_week') {
    return { start: formatLocalDate(startOfLocalWeek(end)), end: formatLocalDate(end) }
  }

  if (preset === 'this_month') {
    return { start: formatLocalDate(startOfLocalMonth(end)), end: formatLocalDate(end) }
  }

  return getLocalDateWindow(30, end)
}

export const getUsageRangePresetImportDays = (
  preset: UsageRangePreset,
  endDate = new Date(),
) => {
  if (preset === 'today') return 1
  if (preset === 'this_week') return getInclusiveLocalDaySpan(startOfLocalWeek(endDate), endDate)
  if (preset === 'this_month') return getInclusiveLocalDaySpan(startOfLocalMonth(endDate), endDate)

  // `all_time` intentionally keeps the safe recent import window. Selecting an
  // unbounded dashboard range must not trigger a full history rebuild.
  return 30
}

export const getUsageRangePresetSpanDays = (
  preset: UsageRangePreset,
  trendDates: string[] = [],
  endDate = new Date(),
) => {
  if (preset === 'all_time') {
    const sortedDates = [...trendDates].filter(Boolean).sort()
    if (sortedDates.length >= 2) {
      const start = new Date(`${sortedDates[0]}T00:00:00`)
      const end = new Date(`${sortedDates[sortedDates.length - 1]}T00:00:00`)
      return getInclusiveLocalDaySpan(start, end)
    }

    return 365
  }

  const { start, end } = getLocalDateRangeWindow(preset, endDate)
  if (!start || !end) return 365

  return getInclusiveLocalDaySpan(new Date(`${start}T00:00:00`), new Date(`${end}T00:00:00`))
}

export const coerceLegacyDaysToUsageRangePreset = (days: number): UsageRangePreset => {
  if (days <= 1) return 'today'
  if (days <= 7) return 'this_week'
  if (days >= 365) return 'all_time'
  return DEFAULT_USAGE_RANGE_PRESET
}
