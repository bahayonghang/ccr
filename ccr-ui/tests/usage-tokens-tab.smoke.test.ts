import { createApp, defineComponent, h, nextTick } from 'vue'
import { describe, expect, it } from 'vitest'
import type { DailyTrend } from '@/types/usage'
import UsageTokensTab from '@/components/usage/UsageTokensTab.vue'
import { createI18nStub } from './helpers/i18n-stub'
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

const mountTokensTab = async (trends: DailyTrend[]) => {
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(
    withUsageDashboardContext(UsageTokensTab, {
      chartComponent: ChartStub,
      trends,
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

const dailyTrend: DailyTrend = {
  date: '2026-05-21',
  request_count: 2,
  total_tokens: 150,
  input_tokens: 70,
  output_tokens: 50,
  reasoning_output_tokens: 20,
  cache_read_tokens: 25,
  cache_creation_tokens: 5,
  cost_usd: 0.1,
}

describe('UsageTokensTab smoke', () => {
  it('renders assistant output and reasoning as separate table/chart values', async () => {
    const mounted = await mountTokensTab([dailyTrend])

    try {
      const text = mounted.el.textContent ?? ''

      expect(text).toContain('Daily token composition')
      expect(text).toContain('Assistant Output')
      expect(text).toContain('Reasoning')
      expect(text).toContain('2026-05-21')
      expect(text).toContain('70')
      expect(text).toContain('30')
      expect(text).toContain('20')
      expect(text).toContain('"Assistant Output","data":[30]')
      expect(text).toContain('"Reasoning","data":[20]')
      expect(text).not.toContain('"Assistant Output","data":[50]')
    } finally {
      mounted.unmount()
    }
  })
})
