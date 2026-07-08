import { createApp, defineComponent, h, nextTick } from 'vue'
import { describe, expect, it } from 'vitest'
import UsageCostTab from '@/components/usage/UsageCostTab.vue'
import type { DailyTrend, ModelStat, SourceBreakdown, UsageSummary } from '@/types/usage'
import { createI18nStub } from './helpers/i18n-stub'
import { makeModelStat } from './helpers/usageFixtures'
import { withUsageDashboardContext } from './helpers/usageDashboardContextStub'

const ChartStub = defineComponent({
  name: 'ChartStub',
  props: {
    options: { type: Object, required: true },
    series: { type: Array, required: true },
    type: { type: String, required: true },
  },
  setup(props) {
    return () =>
      h(
        'pre',
        { class: 'chart-stub' },
        JSON.stringify({
          type: props.type,
          series: props.series,
        })
      )
  },
})

const summary: UsageSummary = {
  total_requests: 10,
  total_tokens: 3000,
  total_input_tokens: 1800,
  total_output_tokens: 800,
  total_cache_read_tokens: 400,
  total_cost_usd: 3,
  cache_efficiency: 0.18,
}

const trends: DailyTrend[] = [
  {
    date: '2026-05-20',
    request_count: 4,
    total_tokens: 1000,
    input_tokens: 600,
    output_tokens: 300,
    reasoning_output_tokens: 50,
    cache_read_tokens: 100,
    cache_creation_tokens: 0,
    cost_usd: 1,
  },
  {
    date: '2026-05-21',
    request_count: 6,
    total_tokens: 2000,
    input_tokens: 1200,
    output_tokens: 500,
    reasoning_output_tokens: 100,
    cache_read_tokens: 300,
    cache_creation_tokens: 0,
    cost_usd: 2,
  },
]

const sourceStats: SourceBreakdown[] = [
  {
    source: 'codex',
    event_count: 7,
    total_tokens: 2200,
    total_cost: 2.25,
    active_days: 2,
    share_tokens: 0.73,
    share_cost: 0.75,
  },
  {
    source: 'claude',
    event_count: 3,
    total_tokens: 800,
    total_cost: 0.75,
    active_days: 1,
    share_tokens: 0.27,
    share_cost: 0.25,
  },
]

const modelStats: ModelStat[] = [
  makeModelStat({
    model: 'cheap-large',
    request_count: 5,
    total_tokens: 2000,
    total_cost: 0.5,
    cost_with_cache: 0.5,
  }),
  makeModelStat({
    model: 'expensive-small',
    request_count: 2,
    total_tokens: 400,
    total_cost: 2,
    cost_with_cache: 2,
  }),
]

const mountCostTab = async () => {
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(
    withUsageDashboardContext(UsageCostTab, {
      chartComponent: ChartStub,
      summary,
      trends,
      sourceStats,
      modelStats,
      formatCost: (value: number) => `$${value.toFixed(2)}`,
      formatTokens: (value: number) => value.toLocaleString(),
    })
  )

  app.use(createI18nStub())
  app.mount(el)
  await nextTick()

  return {
    el,
    unmount: () => {
      app.unmount()
      el.remove()
    },
  }
}

describe('UsageCostTab smoke', () => {
  it('renders daily cost, source ranking, and model ranking from dashboard payload data', async () => {
    const mounted = await mountCostTab()

    try {
      const text = mounted.el.textContent ?? ''

      expect(text).toContain('Cost concentration')
      expect(text).toContain('$3.00')
      expect(text).toContain('"Total Cost","data":[1,2]')
      expect(text).toContain('Source cost ranking')
      expect(text).toContain('Codex')
      expect(text).toContain('75.0%')
      expect(text).toContain('Top model costs')
      expect(text.indexOf('expensive-small')).toBeLessThan(text.indexOf('cheap-large'))
    } finally {
      mounted.unmount()
    }
  })
})
