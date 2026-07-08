import { createApp, defineComponent, h, nextTick } from 'vue'
import { describe, expect, it } from 'vitest'
import UsageModelsTab from '@/components/usage/UsageModelsTab.vue'
import type { ModelDistributionSlice } from '@/views/usage/usageDashboardPresentation'
import type { ModelStat } from '@/types/usage'
import { withUsageDashboardContext } from './helpers/usageDashboardContextStub'

const translations: Record<string, string> = {
  'usage.dashboard.chart.costByModel': 'Cost by Model',
  'usage.dashboard.chart.tokensByModel': 'Token Usage by Model',
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
  'usage.dashboard.table.statusStatic': 'Static',
  'usage.dashboard.table.statusSnapshot': 'Snapshot',
  'usage.dashboard.table.statusMixed': 'Mixed',
  'usage.dashboard.table.statusLegacyAlias': 'Legacy Alias',
  'usage.dashboard.table.statusUnpriced': 'Unpriced',
  'usage.dashboard.chart.preparingDistribution': 'Preparing distribution chart…',
}

const ChartStub = defineComponent({
  name: 'ChartStub',
  setup() {
    return () => h('div', { class: 'chart-stub' }, 'chart')
  },
})

const mountModelsTab = async (modelStats: ModelStat[], shouldRenderChart = false) => {
  const el = document.createElement('div')
  document.body.appendChild(el)

  const modelDistribution: ModelDistributionSlice[] = modelStats.map((model, index) => ({
    id: model.model,
    label: model.model,
    totalCost: model.cost_with_cache ?? 0,
    totalTokens: model.total_tokens,
    requestCount: model.request_count,
    share: index === 0 ? 1 : 0,
    childCount: 1,
    isOther: false,
  }))

  const app = createApp(
    withUsageDashboardContext(UsageModelsTab, {
      chartComponent: ChartStub,
      shouldRenderDistributionChart: shouldRenderChart,
      modelTokenPieSeries: modelDistribution.map((slice) => slice.totalTokens),
      modelTokenPieOptions: {},
      pieColors: ['#4f46e5'],
      distributionSubtitle: 'Distribution',
      modelTokenDistribution: modelDistribution,
      modelStats,
      formatCost: (value: number) => `$${value.toFixed(2)}`,
      formatTokens: (value: number) => value.toLocaleString(),
    })
  )

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
  it('sorts model rows and share by token usage rather than cost', async () => {
    const mounted = await mountModelsTab([
      {
        model: 'expensive-small',
        request_count: 3,
        total_tokens: 1000,
        total_cost: 100,
        input_tokens: 700,
        output_tokens: 300,
        cache_read_tokens: 0,
        cache_creation_tokens: 0,
        cost_with_cache: 100,
        cost_without_cache: 100,
        cache_savings: 0,
        pricing_status: 'priced',
        pricing_source: 'official',
        pricing_rate: '100',
      },
      {
        model: 'cheap-large',
        request_count: 2,
        total_tokens: 10000,
        total_cost: 1,
        input_tokens: 8000,
        output_tokens: 2000,
        cache_read_tokens: 0,
        cache_creation_tokens: 0,
        cost_with_cache: 1,
        cost_without_cache: 1,
        cache_savings: 0,
        pricing_status: 'priced',
        pricing_source: 'official',
        pricing_rate: '1',
      },
    ])

    try {
      const rows = mounted.el.querySelectorAll('tbody tr')

      expect(mounted.el.textContent).toContain('Token Usage by Model')
      expect(rows[0]?.textContent).toContain('cheap-large')
      expect(rows[0]?.textContent).toContain('91%')
      expect(rows[1]?.textContent).toContain('expensive-small')
      expect(rows[1]?.textContent).toContain('9%')
    } finally {
      mounted.unmount()
    }
  })

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

  it('derives cache savings from the cost delta for display consistency', async () => {
    const mounted = await mountModelsTab([
      {
        model: 'zero-saving-model',
        request_count: 1,
        total_tokens: 100,
        total_cost: 9,
        input_tokens: 60,
        output_tokens: 40,
        cache_read_tokens: 0,
        cache_creation_tokens: 0,
        cost_with_cache: 4,
        cost_without_cache: 10,
        cache_savings: 0,
        pricing_status: 'priced',
        pricing_source: 'official',
        pricing_rate: '1/2',
      },
    ])

    try {
      const text = mounted.el.textContent ?? ''

      expect(text).toContain('zero-saving-model')
      expect(text).toContain('$4.00')
      expect(text).toContain('$10.00')
      expect(text).toContain('$6.00')
    } finally {
      mounted.unmount()
    }
  })

  it('renders non-blocking pricing status variants with subdued badges', async () => {
    const mounted = await mountModelsTab([
      {
        model: 'mixed-model',
        request_count: 1,
        total_tokens: 100,
        total_cost: 1,
        input_tokens: 60,
        output_tokens: 40,
        cache_read_tokens: 0,
        cache_creation_tokens: 0,
        cost_with_cache: 1,
        cost_without_cache: 1,
        cache_savings: 0,
        pricing_status: 'mixed',
        pricing_source: 'mixed',
        pricing_rate: 'mixed',
      },
      {
        model: 'snapshot-model',
        request_count: 1,
        total_tokens: 90,
        total_cost: 0.8,
        input_tokens: 50,
        output_tokens: 40,
        cache_read_tokens: 0,
        cache_creation_tokens: 0,
        cost_with_cache: 0.8,
        cost_without_cache: 0.8,
        cache_savings: 0,
        pricing_status: 'snapshot',
        pricing_source: 'litellm',
        pricing_rate: 'snapshot',
      },
      {
        model: 'claude-fable-5',
        request_count: 1,
        total_tokens: 80,
        total_cost: 0.7,
        input_tokens: 40,
        output_tokens: 40,
        cache_read_tokens: 0,
        cache_creation_tokens: 0,
        cost_with_cache: 0.7,
        cost_without_cache: 0.7,
        cache_savings: 0,
        pricing_status: 'static',
        pricing_source: 'static-v1',
        pricing_rate: '10/1/50',
      },
    ])

    try {
      const text = mounted.el.textContent ?? ''

      expect(text).toContain('Mixed')
      expect(text).toContain('Snapshot')
      expect(text).toContain('Static')
      expect(text).toContain('claude-fable-5')
      expect(text).toContain('10/1/50')
      expect(mounted.el.querySelector('.models-tab__status--mixed')).not.toBeNull()
      expect(mounted.el.querySelector('.models-tab__status--snapshot')).not.toBeNull()
      expect(mounted.el.querySelector('.models-tab__status--static')).not.toBeNull()
    } finally {
      mounted.unmount()
    }
  })

  it('defers the model distribution chart behind a preparing placeholder', async () => {
    const mounted = await mountModelsTab([
      {
        model: 'deferred-model',
        request_count: 1,
        total_tokens: 100,
        total_cost: 9,
        input_tokens: 60,
        output_tokens: 40,
        cache_read_tokens: 0,
        cache_creation_tokens: 0,
        cost_with_cache: 4,
        cost_without_cache: 10,
        cache_savings: 0,
        pricing_status: 'priced',
        pricing_source: 'official',
        pricing_rate: '1/2',
      },
    ])

    try {
      expect(mounted.el.querySelector('.chart-stub')).toBeNull()
      expect(mounted.el.textContent).toContain('Preparing distribution chart…')
      expect(mounted.el.textContent).not.toContain('No Data')
    } finally {
      mounted.unmount()
    }
  })
})
