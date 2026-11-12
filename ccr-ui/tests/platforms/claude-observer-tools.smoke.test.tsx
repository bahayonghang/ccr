import { render, screen, within } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { BehaviorAnalysisTab } from '@/features/claude/observer/BehaviorAnalysisTab'
import type { SessionRow } from '@/types/claudeObserver'

vi.mock('@/features/claude/locale', () => ({ t: (key: string) => key }))
vi.mock('@/features/claude/observer/ObserverChart', () => ({ ObserverChart: () => <div /> }))

describe('observed tool sessions', () => {
  it('keeps missing costs unknown and distinguishes a recorded zero', () => {
    const session = (id: string, cost: number | null, calls: number): SessionRow => ({
      session_id: id, project_path: null, model: null, cost_usd: cost,
      tokens: null, tool_call_count: calls, started_at: null,
    })
    render(<BehaviorAnalysisTab
      heatmap={[{ dow: 1, hour: 8, count: 1, timezone: 'utc' }]}
      topTools={[]}
      sessions={[session('unknown', null, 7), session('free', 0, 1), session('no-calls', 0, 0)]}
      animationsEnabled={false}
      shouldRenderChart={false}
    />)
    const unknown = screen.getByText('unknown').closest('tr')!
    expect(within(unknown).getAllByText('claudeCode.observer.behavior.notRecorded')).toHaveLength(2)
    const free = screen.getByText('free').closest('tr')!
    expect(within(free).getAllByText('$0.0000')).toHaveLength(2)
    const noCalls = screen.getByText('no-calls').closest('tr')!
    expect(within(noCalls).getByText('claudeCode.observer.behavior.notRecorded')).toBeTruthy()
    expect(screen.getByText(/UTC/)).toBeTruthy()
  })
})
