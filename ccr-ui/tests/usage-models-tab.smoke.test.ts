import { createApp, defineComponent, h, nextTick } from 'vue'
import { describe, expect, it } from 'vitest'
import UsageModelsTab from '@/components/usage/UsageModelsTab.vue'
import type { ModelDistributionSlice } from '@/views/usage/usageDashboardPresentation'
import type { ModelStat } from '@/types/usage'

const translations: Record<string, string> = {
  'usage.dashboard.chart.costByModel': 'Cost by Model',
  'usage.dashboard.models.title': 'Models',
  'usage.dashboard.models.subtitle': 'Model cost and token profile',
  'usage.dashboard.table.model': 'Model',
  'usage.dashboard.table.requests': 'Requests',
  'usage.dashboard.table.input': 'Input',
  'usage.dashboard.table.output': 'Output',
  'usage.dashboard.table.cacheRead': 'Cache Read',
  'usage.dashboard.table.cacheWrite': 'Cache Write',
  'usage.dashboard.table.rate': 'Rate',
  'usage.dashboard.table.costWithCache': 'With Cache',
  'usage.dashboard.table.costWithoutCache': 'No Cache',
  'usage.dashboard.table.cacheSavings': 'Cache Saved',
  'usage.dashboard.table.pricingStatus': 'Pricing',
  'usage.dashboard.table.share': 'Share',
  'usage.dashboard.table.noData': 'No Data',
  'usage.dashboard.table.statusPriced': 'Priced',
  'usage.dashboard.table.statusLegacyAlias': 'Legacy Alias',
  'usage.dashboard.table.statusUnpriced': 'Unpriced',
}

const ChartStub = defineComponent({
  name: 'ChartStub',
  setup() {
    return () => h('div')
  },
})

const mountModelsTab = async (modelStats: ModelStat[]) => {
  const el = document.createElement('div')
  document.body.appendChild(el)

  const modelDistribution: ModelDistributionSlice[] = modelStats.map((model, index) => ({
    id: model.model,
    label: model.model,
    totalCost: model.cost_with_cache ?? model.total_cost,
    totalTokens: model.total_tokens,
    requestCount: model.request_count,
    share: index === 0 ? 1 : 0,
  }))

  const app = createApp(UsageModelsTab, {
    chartComponent: ChartStub,
    shouldLoadCharts: false,
    pieSeries: [],
    pieOptions: {},
    pieColors: ['#4f46e5'],
    distributionSubtitle: 'Distribution',
    modelDistribution,
    modelStats,
    formatCost: (value: number) => `$${value.toFixed(2)}`,
    formatTokens: (value: number) => value.toLocaleString(),
  })

  app.config.globalProperties.$t = (key: string) => translations[key] ?? key
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

describe('UsageModelsTab smoke', () => {
  it('renders pricing columns, cache savings, and unpriced status', async () => {
    const mounted = await mountModelsTab([
      {
        model: 'gpt-5.4',
        request_count: 4,
        total_tokens: 3000,
        total_cost: 6.4,
        input_tokens: 1800,
        output_tokens: 900,
        cache_read_tokens: 300,
        cache_creation_tokens: 120,
        cost_with_cache: 12.35,
        cost_without_cache: 20,
        cache_savings: 7.65,
        pricing_status: 'priced',
        pricing_source: 'official:openai',
        pricing_rate: '2.5/0.25/15',
      },
      {
        model: 'unknown-model',
        request_count: 1,
        total_tokens: 100,
        total_cost: 0,
        input_tokens: 100,
        output_tokens: 0,
        cache_read_tokens: 0,
        cache_creation_tokens: 0,
        cost_with_cache: 0,
        cost_without_cache: 0,
        cache_savings: 0,
        pricing_status: 'unpriced',
        pricing_source: 'unpriced',
        pricing_rate: '-',
      },
    ])

    try {
      const text = mounted.el.textContent ?? ''

      expect(text).toContain('With Cache')
      expect(text).toContain('No Cache')
      expect(text).toContain('Cache Saved')
      expect(text).toContain('Pricing')
      expect(text).toContain('gpt-5.4')
      expect(text).toContain('$12.35')
      expect(text).toContain('$20.00')
      expect(text).toContain('$7.65')
      expect(text).toContain('2.5/0.25/15')
      expect(text).toContain('unknown-model')
      expect(text).toContain('Unpriced')
      expect(mounted.el.querySelector('.models-tab__status--unpriced')).not.toBeNull()
    } finally {
      mounted.unmount()
    }
  })
})
