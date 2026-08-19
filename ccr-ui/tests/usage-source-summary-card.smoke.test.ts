import { createApp, nextTick } from 'vue'
import { describe, expect, it, vi } from 'vitest'
import UsageSourceSummaryCard from '@/components/usage/UsageSourceSummaryCard.vue'
import type { SourceBreakdown } from '@/types/usage'
import { USAGE_SOURCE_DEFINITIONS } from '@/views/usage/usageSources'
import { createI18nStub } from './helpers/i18n-stub'

const sourceStats: SourceBreakdown[] = [
  {
    source: 'codex',
    event_count: 8,
    total_tokens: 2000,
    total_cost: 1.5,
    active_days: 2,
    share_tokens: 0.8,
    share_cost: 0.75,
  },
  {
    source: 'claude',
    event_count: 2,
    total_tokens: 500,
    total_cost: 0.5,
    active_days: 1,
    share_tokens: 0.2,
    share_cost: 0.25,
  },
]

const mountSourceCard = async (
  onSelect = vi.fn(),
  stats: SourceBreakdown[] = sourceStats
) => {
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(UsageSourceSummaryCard, {
    sourceStats: stats,
    selectedPlatform: 'codex',
    formatCost: (value: number) => `$${value.toFixed(2)}`,
    formatTokens: (value: number) => value.toLocaleString(),
    onSelectSource: onSelect,
  })

  app.use(createI18nStub())
  app.mount(el)
  await nextTick()

  return {
    el,
    onSelect,
    unmount: () => {
      app.unmount()
      el.remove()
    },
  }
}

describe('UsageSourceSummaryCard smoke', () => {
  it('exposes the canonical llmusage source filters', () => {
    expect(USAGE_SOURCE_DEFINITIONS.map((source) => source.id)).toEqual([
      'claude',
      'codex',
      'opencode',
      'antigravity',
      'kimi_code',
      'pi',
      'grok',
      'zcode',
      'deepseek_harness',
    ])
    expect(USAGE_SOURCE_DEFINITIONS.find((source) => source.id === 'antigravity')?.fallbackLabel)
      .toBe('Antigravity CLI')
  })

  it('renders source ranking and emits selected source filters', async () => {
    const mounted = await mountSourceCard()

    try {
      const text = mounted.el.textContent ?? ''
      const rows = mounted.el.querySelectorAll('.source-card__item')

      expect(text).toContain('Usage by source')
      expect(rows[0]?.textContent).toContain('Codex')
      expect(rows[0]?.classList.contains('source-card__item--active')).toBe(true)
      expect(text).toContain('$1.50')
      expect(text).toContain('80.0%')

      const secondButton = rows[1] as HTMLButtonElement
      secondButton.click()
      expect(mounted.onSelect).toHaveBeenCalledWith('claude')
    } finally {
      mounted.unmount()
    }
  })

  it('emits the canonical Antigravity filter from source rows', async () => {
    const mounted = await mountSourceCard(vi.fn(), [{
      source: 'antigravity',
      event_count: 3,
      total_tokens: 700,
      total_cost: 0.4,
      active_days: 1,
      share_tokens: 1,
      share_cost: 1,
    }])

    try {
      const row = mounted.el.querySelector('.source-card__item') as HTMLButtonElement
      expect(row.textContent).toContain('Antigravity CLI')
      row.click()
      expect(mounted.onSelect).toHaveBeenCalledWith('antigravity')
    } finally {
      mounted.unmount()
    }
  })
})
