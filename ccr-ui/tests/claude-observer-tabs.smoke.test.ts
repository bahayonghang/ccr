import { createApp, defineComponent, h, nextTick, type Component } from 'vue'
import { afterEach, describe, expect, it } from 'vitest'
import CostAttributionTab from '@/components/claude-observer/CostAttributionTab.vue'
import TokenDetailTab from '@/components/claude-observer/TokenDetailTab.vue'
import BehaviorAnalysisTab from '@/components/claude-observer/BehaviorAnalysisTab.vue'
import { createI18nStub } from './helpers/i18n-stub'

const mount = async (
  component: Component,
  props: Record<string, unknown>,
  translations: Record<string, string>
) => {
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(
    defineComponent({
      render() {
        return h(component, props)
      },
    })
  )

  app.use(createI18nStub())
  app.config.globalProperties.$t = (key: string) => translations[key] ?? key
  app.mount(el)
  await nextTick()
  await nextTick()

  return {
    el,
    unmount: () => {
      app.unmount()
      el.remove()
    },
  }
}

afterEach(() => {
  document.body.innerHTML = ''
})

describe('claude observer tabs smoke', () => {
  it('shows a preparing placeholder for deferred cost charts', async () => {
    const translations = {
      'claudeCode.observer.chart.dailyTrend30': 'Daily cost',
      'claudeCode.observer.chart.dailyTrend30Sub': 'USD',
      'claudeCode.observer.chart.byProject': 'By project',
      'claudeCode.observer.chart.byProjectSub': 'Top projects',
      'claudeCode.observer.chart.byModel': 'By model',
      'claudeCode.observer.chart.byModelSub': 'Top models',
      'claudeCode.observer.chart.preparingTrend': 'Preparing trend chart…',
      'claudeCode.observer.empty.noTrend': 'No data yet',
    }

    const mounted = await mount(
      CostAttributionTab,
      {
        daily: [{ date: '2026-06-01', cost_usd: 12.34 }],
        byProject: [{ key: 'workspace-a', cost_usd: 4 }],
        byModel: [{ key: 'claude-opus', cost_usd: 8.34 }],
        animationsEnabled: false,
        shouldRenderChart: false,
      },
      translations,
    )

    try {
      expect(mounted.el.querySelector('.chart-stub')).toBeNull()
      expect(mounted.el.textContent).toContain('Preparing trend chart…')
      expect(mounted.el.textContent).not.toContain('No data yet')
    } finally {
      mounted.unmount()
    }
  })

  it('shows a preparing placeholder for deferred token charts', async () => {
    const translations = {
      'claudeCode.observer.chart.dailyTokens30': 'Token mix',
      'claudeCode.observer.chart.dailyTokens30Sub': 'Stacked',
      'claudeCode.observer.tokenDetail.cacheWriteExplainTitle': 'Why cache_write costs',
      'claudeCode.observer.tokenDetail.cacheWriteExplain': 'Explain',
      'claudeCode.observer.chart.preparingTrend': 'Preparing trend chart…',
      'claudeCode.observer.empty.noTrend': 'No data yet',
      'claudeCode.observer.metric.cacheHitRate': 'Hit rate',
      'claudeCode.observer.metric.cacheHitRateDetail': 'detail',
      'claudeCode.observer.metric.inputUncached': 'Input',
      'claudeCode.observer.metric.inputUncachedDetail': 'detail',
      'claudeCode.observer.metric.output': 'Output',
      'claudeCode.observer.metric.outputDetail': 'detail',
      'claudeCode.observer.metric.cacheRead': 'Cache read',
      'claudeCode.observer.metric.cacheReadDetail': 'detail',
    }

    const mounted = await mount(
      TokenDetailTab,
      {
        stats: {
          hit_rate: 0.5,
          total_input_tokens: 100,
          total_output_tokens: 50,
          total_cache_read_tokens: 25,
        },
        daily: [{ date: '2026-06-01', input_tokens: 100, output_tokens: 50, cache_read_tokens: 25, cache_write_tokens: 10 }],
        animationsEnabled: false,
        shouldRenderChart: false,
      },
      translations,
    )

    try {
      expect(mounted.el.querySelector('.chart-stub')).toBeNull()
      expect(mounted.el.textContent).toContain('Preparing trend chart…')
      expect(mounted.el.textContent).not.toContain('No data yet')
    } finally {
      mounted.unmount()
    }
  })

  it('shows a preparing placeholder for deferred behavior charts', async () => {
    const translations = {
      'claudeCode.observer.chart.toolHeatmap': 'Heatmap',
      'claudeCode.observer.chart.toolHeatmapSub': 'Last 30 days',
      'claudeCode.observer.chart.topTools': 'Top tools',
      'claudeCode.observer.chart.topToolsSub': 'Sorted',
      'claudeCode.observer.behavior.sourceNote': 'Source note',
      'claudeCode.observer.behavior.efficiencyTitle': 'Efficiency',
      'claudeCode.observer.behavior.efficiencySub': 'Top sessions',
      'claudeCode.observer.behavior.colSession': 'Session',
      'claudeCode.observer.behavior.colProject': 'Project',
      'claudeCode.observer.behavior.colCost': 'Cost',
      'claudeCode.observer.behavior.colTools': 'Tools',
      'claudeCode.observer.behavior.colCostPerTool': '$/call',
      'claudeCode.observer.chart.preparingHeatmap': 'Preparing heatmap…',
      'claudeCode.observer.empty.noTrend': 'No data yet',
    }

    const mounted = await mount(
      BehaviorAnalysisTab,
      {
        heatmap: [{ dow: 1, hour: 12, count: 3 }],
        topTools: [{ tool_name: 'bash', call_count: 10 }],
        sessions: [{ session_id: 'abc123', project_path: '/workspace', cost_usd: 2, tool_call_count: 4 }],
        animationsEnabled: false,
        shouldRenderChart: false,
      },
      translations,
    )

    try {
      expect(mounted.el.querySelector('.chart-stub')).toBeNull()
      expect(mounted.el.textContent).toContain('Preparing heatmap…')
      expect(mounted.el.textContent).not.toContain('No data yet')
    } finally {
      mounted.unmount()
    }
  })
})
