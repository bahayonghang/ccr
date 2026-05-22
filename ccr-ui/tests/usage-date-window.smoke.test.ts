import { describe, expect, it } from 'vitest'
import {
  coerceLegacyDaysToUsageRangePreset,
  formatLocalDate,
  getLocalDateRangeWindow,
  getLocalDateWindow,
  getUsageRangePresetImportDays,
  getUsageRangePresetSpanDays,
} from '@/views/usage/dateWindow'

describe('usage date window helpers', () => {
  it('formats local calendar dates without UTC shifting', () => {
    expect(formatLocalDate(new Date(2026, 0, 5, 23, 59, 59))).toBe('2026-01-05')
  })

  it('builds inclusive local day windows and clamps invalid ranges to one day', () => {
    const endDate = new Date(2026, 4, 19, 15, 30, 0)

    expect(getLocalDateWindow(7, endDate)).toEqual({
      start: '2026-05-13',
      end: '2026-05-19',
    })
    expect(getLocalDateWindow(1.8, endDate)).toEqual({
      start: '2026-05-19',
      end: '2026-05-19',
    })
    expect(getLocalDateWindow(0, endDate)).toEqual({
      start: '2026-05-19',
      end: '2026-05-19',
    })
  })

  it('preserves month and year boundaries', () => {
    expect(getLocalDateWindow(3, new Date(2026, 0, 1, 8, 0, 0))).toEqual({
      start: '2025-12-30',
      end: '2026-01-01',
    })
  })

  it('builds explicit range presets and keeps all-time unbounded', () => {
    const endDate = new Date(2026, 4, 21, 15, 30, 0)

    expect(getLocalDateRangeWindow('today', endDate)).toEqual({
      start: '2026-05-21',
      end: '2026-05-21',
    })
    expect(getLocalDateRangeWindow('this_week', endDate)).toEqual({
      start: '2026-05-18',
      end: '2026-05-21',
    })
    expect(getLocalDateRangeWindow('this_month', endDate)).toEqual({
      start: '2026-05-01',
      end: '2026-05-21',
    })
    expect(getLocalDateRangeWindow('last_30d', endDate)).toEqual({
      start: '2026-04-22',
      end: '2026-05-21',
    })
    expect(getLocalDateRangeWindow('all_time', endDate)).toEqual({})
  })

  it('uses safe recent import days for all-time and supports legacy day migration', () => {
    const endDate = new Date(2026, 4, 21, 15, 30, 0)

    expect(getUsageRangePresetImportDays('today', endDate)).toBe(1)
    expect(getUsageRangePresetImportDays('this_week', endDate)).toBe(4)
    expect(getUsageRangePresetImportDays('this_month', endDate)).toBe(21)
    expect(getUsageRangePresetImportDays('last_30d', endDate)).toBe(30)
    expect(getUsageRangePresetImportDays('all_time', endDate)).toBe(30)

    expect(coerceLegacyDaysToUsageRangePreset(1)).toBe('today')
    expect(coerceLegacyDaysToUsageRangePreset(7)).toBe('this_week')
    expect(coerceLegacyDaysToUsageRangePreset(90)).toBe('last_30d')
    expect(coerceLegacyDaysToUsageRangePreset(365)).toBe('all_time')
  })

  it('derives all-time chart span from trend dates', () => {
    expect(getUsageRangePresetSpanDays('all_time', ['2026-01-01', '2026-04-10'])).toBe(100)
  })
})
