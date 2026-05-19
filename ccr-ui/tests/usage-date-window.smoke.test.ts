import { describe, expect, it } from 'vitest'
import { formatLocalDate, getLocalDateWindow } from '@/views/usage/dateWindow'

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
})
