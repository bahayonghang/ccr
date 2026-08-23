import { useMemo } from 'react'
import { buildChartAnimations, buildChartTheme } from '@/views/usage/usageChartOptions'
import { usageSourceFallbackLabel } from '@/views/usage/usageSources'
import { ApexChart } from '../charts/ApexChart'
import { useUsageDashboardContext } from '../UsageDashboardContext'
import { useUsageT } from '../translate'
import '../styles/usage-cost-tab.css'

export function UsageCostTab() {
  const ctx = useUsageDashboardContext()
  const t = useUsageT()
  const theme = buildChartTheme()
  const totalCost = ctx.summary?.total_cost_usd ?? 0
  const totalRequests = ctx.summary?.total_requests ?? 0
  const hasTrendRows = ctx.trends.length > 0

  const chartSeries = useMemo(() => [{
    name: t('usage.dashboard.table.cost'),
    data: ctx.trends.map((item) => item.cost_usd),
  }], [ctx.trends, t])

  const chartOptions = useMemo(() => ({
    chart: {
      background: 'transparent',
      fontFamily: 'inherit',
      toolbar: { show: false },
      animations: buildChartAnimations(),
      redrawOnParentResize: false,
      redrawOnWindowResize: false,
    },
    theme: { mode: theme.mode },
    colors: [theme.primary],
    dataLabels: { enabled: false },
    grid: { borderColor: theme.grid, strokeDashArray: 4 },
    xaxis: {
      categories: ctx.trends.map((item) => item.date),
      labels: { style: { colors: theme.textMuted } },
      axisBorder: { show: false },
      axisTicks: { show: false },
    },
    yaxis: {
      labels: {
        style: { colors: theme.textMuted },
        formatter: (value: number) => ctx.formatCost(value),
      },
    },
  }), [ctx, theme])

  const sourceRankings = [...ctx.sourceStats]
    .filter((item) => item.total_cost > 0 || item.total_tokens > 0)
    .sort((left, right) => right.total_cost - left.total_cost)

  return (
    <section className="cost-tab">
      <article className="cost-tab__anchor glass-panel">
        <div>
          <p className="cost-tab__eyebrow">{t('usage.dashboard.cost.eyebrow')}</p>
          <h3>{t('usage.dashboard.cost.title')}</h3>
          <p>{t('usage.dashboard.cost.subtitle')}</p>
        </div>
        <div className="cost-tab__anchor-value">
          <span>{t('usage.dashboard.cost.totalCost')}</span>
          <strong>{ctx.formatCost(totalCost)}</strong>
          <small>{totalRequests.toLocaleString()} {t('usage.dashboard.table.requests')}</small>
        </div>
      </article>
      <article className="cost-tab__chart-card glass-panel">
        {hasTrendRows ? (
          <ApexChart type="bar" height={300} options={chartOptions} series={chartSeries} />
        ) : (
          <div className="cost-tab__empty">{t('usage.dashboard.table.noData')}</div>
        )}
      </article>
      <section className="cost-tab__rankings">
        <article className="cost-tab__ranking-card glass-panel">
          <h3>{t('usage.dashboard.cost.sourceTitle')}</h3>
          <ol className="cost-tab__ranking-list">
            {sourceRankings.map((item) => (
              <li key={item.source} className="cost-tab__ranking-item">
                <div className="cost-tab__ranking-main">
                  <strong>{usageSourceFallbackLabel(item.source)}</strong>
                  <b>{ctx.formatCost(item.total_cost)}</b>
                </div>
              </li>
            ))}
          </ol>
        </article>
      </section>
    </section>
  )
}
