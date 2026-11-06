import { afterEach, describe, expect, it } from 'vitest'
import { capturedReportClock, reportNow } from '@/utils/reportClock'
import {
  getLocalDateRangeWindow, getReportDateRangeWindow, getReportDateWindow,
  getUsageRangePresetImportDays, getUsageRangePresetSpanDays,
} from '@/views/usage/dateWindow'
import { homeDateWindow } from '@/features/usage/dashboard/DashboardCostMetric'
import { useUsageViewStore } from '@/features/usage/stores'

const capture = (value: unknown) => Object.defineProperty(window, '__CCR_REPORT_CLOCK__', {
  value, configurable: true, writable: false,
})
afterEach(() => { Reflect.deleteProperty(window, '__CCR_REPORT_CLOCK__'); useUsageViewStore.getState().resetFilters() })

describe('explicit benchmark report clock', () => {
  it('keeps default live local windows without captured context', () => {
    expect(capturedReportClock()).toBeUndefined()
    expect(getReportDateRangeWindow('today')).toEqual(getLocalDateRangeWindow('today'))
    expect(Math.abs(Date.now() - reportNow().getTime())).toBeLessThan(2000)
  })
  it('anchors all default presets and home cost to the same UTC calendar', () => {
    capture(Object.freeze({ asOfUtc: '2000-01-01T00:30:00Z', timezone: 'UTC' }))
    expect(reportNow().toISOString()).toBe('2000-01-01T00:30:00.000Z')
    expect(getReportDateRangeWindow('today')).toEqual({ start: '2000-01-01', end: '2000-01-01' })
    expect(getReportDateRangeWindow('this_week')).toEqual({ start: '1999-12-27', end: '2000-01-01' })
    expect(getReportDateRangeWindow('this_month')).toEqual({ start: '2000-01-01', end: '2000-01-01' })
    expect(getReportDateRangeWindow('all_time')).toEqual({})
    expect(getReportDateWindow(7)).toEqual({ start: '1999-12-26', end: '2000-01-01' })
    expect(homeDateWindow(7)).toEqual({ startDate: '1999-12-26', endDate: '2000-01-01' })
    expect(getUsageRangePresetImportDays('this_week')).toBe(6)
    expect(getUsageRangePresetSpanDays('this_week')).toBe(6)
    expect(Date.now()).toBeGreaterThan(Date.parse('2000-01-01T00:30:00Z'))
  })
  it('retains explicit user dates and manual store range', () => {
    capture({ asOfUtc: '2000-01-01T00:30:00Z', timezone: 'UTC' })
    const explicit = new Date(2024, 5, 15, 12)
    expect(homeDateWindow(1, explicit)).toEqual({ startDate: '2024-06-15', endDate: '2024-06-15' })
    expect(getLocalDateRangeWindow('today', explicit)).toEqual({ start: '2024-06-15', end: '2024-06-15' })
    useUsageViewStore.getState().setTimeRange({ start: '2024-06-01', end: '2024-06-15' })
    expect(useUsageViewStore.getState().timeRange).toEqual({ start: '2024-06-01', end: '2024-06-15' })
    useUsageViewStore.getState().resetFilters()
    expect(useUsageViewStore.getState().timeRange).toEqual({ start: '1999-12-03', end: '2000-01-01' })
  })
  it('refuses invalid captured context without falling back to host date', () => {
    for (const invalid of [{ asOfUtc: '2000-02-31T00:00:00Z', timezone: 'UTC' }, { asOfUtc: 'invalid', timezone: 'UTC' }, { asOfUtc: '2000-01-01T00:00:00Z', timezone: 'Local' }, { asOfUtc: '2000-01-01T00:00:00-05:00', timezone: 'UTC' }]) {
      capture(invalid)
      expect(() => reportNow()).toThrow('Invalid benchmark report clock')
    }
  })
})
