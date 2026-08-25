import { fireEvent, render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router'
import { describe, expect, it } from 'vitest'
import { DashboardSignalStream } from '@/features/usage/dashboard/DashboardSignalStream'
import type { DashboardSignalEntry } from '@/features/usage/dashboard/useDashboardSignals'

const entry = (
  overrides: Pick<DashboardSignalEntry, 'id' | 'message'> & Partial<DashboardSignalEntry>,
): DashboardSignalEntry => ({
  timestamp: '2026-08-25T12:00:00.000Z',
  level: 'info',
  channel: 'usage',
  eventType: 'log',
  source: 'tauri',
  ...overrides,
})

const renderStream = (entries: DashboardSignalEntry[], limit?: number) =>
  render(
    <MemoryRouter>
      <DashboardSignalStream entries={entries} limit={limit} />
    </MemoryRouter>,
  )

const trailingCount = (label: string | null | undefined) => {
  const match = label?.trim().match(/(\d+)$/)
  if (!match) {
    throw new Error(`expected trailing count in "${label ?? ''}"`)
  }
  return Number(match[1])
}

const filterRadio = (pattern: RegExp) => screen.getByRole('radio', { name: pattern })

const visibleRows = (container: HTMLElement) =>
  [...container.querySelectorAll<HTMLElement>('.dashboard-signal')]

describe('dashboard signal stream', () => {
  it('keeps three filter radios whose counts are aggregated, pre-filter, and pre-slice', () => {
    const entries = [
      entry({ id: 'info-1', message: 'imported records', level: 'info', timestamp: '2026-08-25T12:08:00Z' }),
      entry({ id: 'info-2', message: 'shell mounted', level: 'info', timestamp: '2026-08-25T12:07:00Z' }),
      entry({ id: 'warn-1', message: 'empty array', level: 'warn', timestamp: '2026-08-25T12:06:00Z' }),
      entry({ id: 'error-1', message: 'import failed', level: 'error', timestamp: '2026-08-25T12:05:00Z' }),
      entry({ id: 'dup-a', message: 'retry timeout', level: 'warn', channel: 'sync', timestamp: '2026-08-25T12:04:03Z' }),
      entry({ id: 'dup-b', message: 'retry timeout', level: 'warn', channel: 'sync', timestamp: '2026-08-25T12:04:02Z' }),
      entry({ id: 'dup-c', message: 'retry timeout', level: 'warn', channel: 'sync', timestamp: '2026-08-25T12:04:01Z' }),
    ]

    const { container } = renderStream(entries, 3)

    const allRadio = filterRadio(/全部|All/)
    const warnRadio = filterRadio(/警告以上|Warn\+/)
    const errorRadio = filterRadio(/错误|Error/)

    expect(screen.getByRole('radiogroup')).toBeTruthy()
    expect(trailingCount(allRadio.textContent)).toBe(5)
    expect(trailingCount(warnRadio.textContent)).toBe(3)
    expect(trailingCount(errorRadio.textContent)).toBe(1)
    expect(visibleRows(container)).toHaveLength(3)
    expect(trailingCount(allRadio.textContent)).toBeGreaterThan(visibleRows(container).length)
  })

  it('shows error-level rows in the warn filter and hides info rows', () => {
    const entries = [
      entry({ id: 'info-1', message: 'quiet heartbeat', level: 'info', timestamp: '2026-08-25T12:03:00Z' }),
      entry({ id: 'warn-1', message: 'observer empty', level: 'warn', timestamp: '2026-08-25T12:02:00Z' }),
      entry({ id: 'error-1', message: 'usage import failed', level: 'error', timestamp: '2026-08-25T12:01:00Z' }),
    ]

    const { container } = renderStream(entries)

    fireEvent.click(filterRadio(/警告以上|Warn\+/))

    const rows = visibleRows(container)
    expect(rows.map((row) => row.getAttribute('data-level'))).toEqual(['warn', 'error'])
    expect(container.textContent).toContain('usage import failed')
    expect(container.textContent).not.toContain('quiet heartbeat')
  })

  it('aggregates adjacent duplicates into one row with ×N', () => {
    const entries = [
      entry({ id: 'a', message: 'claudeObserver empty', channel: 'runtime', level: 'warn', timestamp: '2026-08-25T12:00:03Z' }),
      entry({ id: 'b', message: 'claudeObserver empty', channel: 'runtime', level: 'warn', timestamp: '2026-08-25T12:00:02Z' }),
      entry({ id: 'c', message: 'claudeObserver empty', channel: 'runtime', level: 'warn', timestamp: '2026-08-25T12:00:01Z' }),
    ]

    const { container } = renderStream(entries)

    expect(visibleRows(container)).toHaveLength(1)
    expect(container.querySelector('.dashboard-signal__count')?.textContent).toBe('×3')
    expect(trailingCount(filterRadio(/全部|All/).textContent)).toBe(1)
  })

  it('does not aggregate duplicates that are not adjacent after time sort', () => {
    const entries = [
      entry({ id: 'a', message: 'same text', channel: 'usage', timestamp: '2026-08-25T12:00:03Z' }),
      entry({ id: 'b', message: 'other text', channel: 'usage', timestamp: '2026-08-25T12:00:02Z' }),
      entry({ id: 'c', message: 'same text', channel: 'usage', timestamp: '2026-08-25T12:00:01Z' }),
    ]

    const { container } = renderStream(entries)

    expect(visibleRows(container)).toHaveLength(3)
    expect(container.querySelector('.dashboard-signal__count')).toBeNull()
  })

  it('keeps the channel column and a monitoring empty-state CTA', () => {
    const { container, rerender } = renderStream([
      entry({ id: 'live', message: 'imported 6424 records', channel: 'usage' }),
    ])

    const row = visibleRows(container)[0]
    expect(row?.querySelector('.dashboard-signal__time')).toBeTruthy()
    expect(row?.querySelector('.dashboard-signal__dot')).toBeTruthy()
    expect(row?.querySelector('.dashboard-signal__channel')?.textContent).toBe('usage')
    expect(row?.querySelector('.dashboard-signal__message')?.textContent).toBe(
      'imported 6424 records',
    )
    expect(row?.querySelector('.dashboard-signal__level')?.textContent).toMatch(/信息|Info/)
    expect(screen.getByRole('link', { name: /打开监控|Open monitoring/ }).getAttribute('href')).toBe(
      '/monitoring',
    )

    rerender(
      <MemoryRouter>
        <DashboardSignalStream entries={[]} />
      </MemoryRouter>,
    )

    expect(container.querySelector('.dashboard-signals__empty')).toBeTruthy()
    expect(screen.getByRole('link', { name: /打开监控|Open monitoring/ }).getAttribute('href')).toBe(
      '/monitoring',
    )
    expect(container.querySelector('.dashboard-signals__footer')).toBeNull()
  })

  it('truncates long messages with a title tooltip and keeps channel available via title', () => {
    const longMessage = `usage import failed ${'x'.repeat(180)}`
    const { container } = renderStream([
      entry({
        id: 'long',
        message: longMessage,
        channel: 'environment-health',
        level: 'error',
      }),
    ])

    const message = container.querySelector('.dashboard-signal__message')
    const channel = container.querySelector('.dashboard-signal__channel')
    expect(message?.getAttribute('title')).toBe(longMessage)
    expect(channel?.getAttribute('title')).toBe('environment-health')
    expect(container.querySelector('.dashboard-signal__level')?.textContent).toMatch(/错误|Error/)
  })
})
