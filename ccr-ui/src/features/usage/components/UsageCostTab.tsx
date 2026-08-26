import { useMemo, useRef } from 'react'
import { buildChartTheme, getTrendTickAmount } from '@/views/usage/usageChartOptions'
import {
  buildDailyBarChartOptions,
  stabilizeDailyBarSeries,
  toDailyBarPoints,
  type DailyBarSeries,
} from '@/views/usage/usageDailyBarChart'
import { usageSourceFallbackLabel } from '@/views/usage/usageSources'
import { ApexChart } from '../charts/ApexChart'
import { useUsageDashboardContext } from '../UsageDashboardContext'
import { useUsageT } from '../translate'
import '../styles/usage-cost-tab.css'

export function UsageCostTab() {
  const ctx = useUsageDashboardContext()
  const t = useUsageT()
  const theme = ctx.chartTheme ?? buildChartTheme()
  const locale = ctx.locale || 'zh-CN'
  const totalCost = ctx.summary?.total_cost_usd ?? 0
  const totalRequests = ctx.summary?.total_requests ?? 0
  const hasTrendRows = ctx.trends.length > 0
  const previousSeries = useRef<DailyBarSeries[] | undefined>(undefined)

  const chartSeries = useMemo(() => {
    const next: DailyBarSeries[] = [{
      name: t('usage.dashboard.table.cost'),
      data: toDailyBarPoints(ctx.trends, (item) => item.cost_usd),
    }]
    const stable = stabilizeDailyBarSeries(previousSeries.current, next)
    previousSeries.current = stable
    return stable
  }, [ctx.trends, t])

  const chartOptions = useMemo(
    () =>
      buildDailyBarChartOptions({
        theme,
        locale,
        granularity: 'day',
        tickAmount: getTrendTickAmount(ctx.trends.length),
        stacked: false,
        palette: 'cost',
        formatY: ctx.formatCost,
      }),
    [ctx.formatCost, ctx.trends.length, locale, theme],
  )

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
          {sourceRankings.length > 0 ? (
            <ol className="cost-tab__ranking-list">
              {sourceRankings.map((item, index) => (
                <li key={item.source} className="cost-tab__ranking-item">
                  <span className="cost-tab__rank">{index + 1}</span>
                  <div className="cost-tab__ranking-main">
                    <div className="cost-tab__ranking-row">
                      <strong>{usageSourceFallbackLabel(item.source)}</strong>
                      <b>{ctx.formatCost(item.total_cost)}</b>
                      <small>{`${Math.round(item.share_cost * 100)}%`}</small>
                    </div>
                    <div className="cost-tab__bar">
                      <span style={{ width: `${Math.round(item.share_cost * 100)}%` }} />
                    </div>
                  </div>
                </li>
              ))}
            </ol>
          ) : (
            <div className="cost-tab__empty cost-tab__empty--compact">
              {t('usage.dashboard.table.noData')}
            </div>
          )}
        </article>
      </section>
    </section>
  )
}
