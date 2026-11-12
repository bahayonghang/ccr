import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import type { InsightDto } from '@/types/claudeObserver'
import { ObserverHeroGrid } from '@/features/claude/observer/ObserverChrome'

vi.mock('@/features/claude/locale', () => ({ t: (key: string) => key }))

describe('observer project attribution', () => {
  it.each([
    [0, true, 'totalValueDetail'],
    [0, false, 'totalValueUnattributedDetail'],
    [2, false, 'totalValuePartialDetail'],
    [2, true, 'totalValueDetail'],
  ] as const)('distinguishes count %i and complete %s', (projects, complete, detail) => {
    const insight: InsightDto = {
      today_value_usd: 0, month_value_usd: 0, total_value_usd: 0,
      today_tokens: 0, month_tokens: 0, total_sessions: 0, total_projects: projects,
      project_attribution_complete: complete,
      subscription: { mode: 'auto', plan: 'team', monthly_usd: 200 },
      roi: null, pricing_version: 'fixture',
    }
    render(<ObserverHeroGrid insight={insight} subscription={insight.subscription} hasRoi={false} />)
    expect(screen.getByText(`claudeCode.observer.metric.${detail}`)).toBeTruthy()
  })
})
