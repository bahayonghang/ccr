import { createApp, defineComponent, h, nextTick } from 'vue'
import { afterEach, describe, expect, it } from 'vitest'
import UsageOverviewTab from '@/components/usage/UsageOverviewTab.vue'

const ChartStub = defineComponent({
  name: 'ChartStub',
  render() {
    return h('div', { class: 'chart-stub' }, 'chart')
  },
})

const mountOverviewTab = async () => {
  const el = document.createElement('div')
  document.body.appendChild(el)

  const longModelLabel = 'claude-opus-4-6-enterprise-ultra-extended-context-and-observability-build'
  const longProjectPath =
    'D:/workspace/very-long-folder-name/another-layer/of-reporting/usage-dashboard-overflow-repro-project'

  const app = createApp(
    defineComponent({
      render() {
        return h(UsageOverviewTab, {
          chartComponent: ChartStub,
          shouldLoadCharts: true,
          hasRenderableTrendData: true,
          trendSeries: [
            {
              name: 'Input',
              data: [
                { x: '2026-03-01T00:00:00Z', y: 1200 },
                { x: '2026-03-02T00:00:00Z', y: 1800 },
              ],
            },
            {
              name: 'Output',
              data: [
                { x: '2026-03-01T00:00:00Z', y: 480 },
                { x: '2026-03-02T00:00:00Z', y: 720 },
              ],
            },
            {
              name: 'Cache',
              data: [
                { x: '2026-03-01T00:00:00Z', y: 90 },
                { x: '2026-03-02T00:00:00Z', y: 110 },
              ],
            },
          ],
          trendOptions: {},
          trendSubtitle: '30 day desktop window',
          trendGranularityLabel: 'Daily',
          pieSeries: [82, 18],
          pieOptions: {},
          pieColors: ['#CA8FD1', '#7F78D8'],
          distributionSubtitle: '2 models visible',
          modelDistribution: [
            {
              id: 'primary',
              label: longModelLabel,
              totalCost: 82,
              totalTokens: 64000,
              requestCount: 96,
              share: 0.82,
              childCount: 1,
              isOther: false,
            },
            {
              id: 'others',
              label: 'Others',
              totalCost: 18,
              totalTokens: 16000,
              requestCount: 24,
              share: 0.18,
              childCount: 3,
              isOther: true,
            },
          ],
          modelStats: [],
          projectStats: [],
          overviewHighlights: [
            { id: 'density', label: 'Density', value: 'Daily', detail: '30 points' },
          ],
          topModelRankings: [
            {
              id: 'primary',
              label: longModelLabel,
              title: longModelLabel,
              detail: '96 requests · 64.0K',
              value: '$82.00',
              share: 0.82,
            },
          ],
          topProjectRankings: [
            {
              id: 'project',
              label: '.../overflow-repro-project',
              title: longProjectPath,
              detail: '64.0K · 96 requests',
              value: '$82.00',
              share: 0.82,
            },
          ],
          formatCost: (value: number) => `$${value.toFixed(2)}`,
          formatTokens: (value: number) => `${value}`,
          shortenPath: (value: string) => value,
        })
      },
    })
  )

  app.config.globalProperties.$t = (key: string) => key
  app.mount(el)
  await nextTick()
  await nextTick()

  return {
    el,
    unmount: () => {
      app.unmount()
      el.remove()
    },
    longModelLabel,
    longProjectPath,
  }
}

afterEach(() => {
  document.body.innerHTML = ''
})

describe('usage overview tab smoke', () => {
  it('renders long model and project labels with title fallbacks', async () => {
    const { el, unmount, longModelLabel, longProjectPath } = await mountOverviewTab()

    try {
      const canvas = el.querySelector('.overview-tab__canvas') as HTMLElement | null
      const trendShell = el.querySelector('.overview-tab__trend-shell') as HTMLElement | null
      const trendLegendItems = el.querySelectorAll('.overview-tab__trend-legend-item')
      const distributionLegend = el.querySelector(
        '.distribution-card__legend'
      ) as HTMLElement | null
      const distributionLabel = el.querySelector('.distribution-card__label')
      const rankModelLabel = el.querySelector('.overview-tab__rankings .overview-tab__rank-label')
      const rankProjectLabel = el.querySelectorAll(
        '.overview-tab__rankings .overview-tab__rank-label'
      )[1]

      expect(canvas).not.toBeNull()
      expect(trendShell).not.toBeNull()
      expect(trendLegendItems).toHaveLength(3)
      expect(trendLegendItems[0]?.textContent).toContain('Input')
      expect(trendLegendItems[1]?.textContent).toContain('Output')
      expect(trendLegendItems[2]?.textContent).toContain('Cache')
      expect(distributionLegend).not.toBeNull()
      expect(distributionLabel?.getAttribute('title')).toBe(longModelLabel)
      expect(rankModelLabel?.getAttribute('title')).toBe(longModelLabel)
      expect(rankProjectLabel?.getAttribute('title')).toBe(longProjectPath)
      expect(el.querySelectorAll('.overview-tab__rank-item')).toHaveLength(2)
      expect(trendShell?.querySelector('.chart-stub')).not.toBeNull()
      expect(el.querySelector('.overview-tab__empty--trend')).toBeNull()
      expect(distributionLegend?.querySelectorAll('.distribution-card__legend-item')).toHaveLength(
        2
      )
    } finally {
      unmount()
    }
  })
})
