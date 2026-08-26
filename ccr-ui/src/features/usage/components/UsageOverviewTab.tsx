import { ApexChart } from '../charts/ApexChart'
import { useUsageDashboardContext } from '../UsageDashboardContext'
import { useUsageT } from '../translate'
import { UsageModelDistributionCard } from './UsageModelDistributionCard'
import { UsageSourceSummaryCard } from './UsageSourceSummaryCard'
import '../styles/usage-overview-tab.css'

export function UsageOverviewTab() {
  const ctx = useUsageDashboardContext()
  const t = useUsageT()
  const formatShare = (value: number) => `${Math.round(value * 100)}%`

  return (
    <div className="overview-tab">
      {ctx.sourceStats.length > 0 ? (
        <UsageSourceSummaryCard
          formatCost={ctx.formatCost}
          formatTokens={ctx.formatTokens}
          selectedPlatform={ctx.selectedPlatform}
          sourceStats={ctx.sourceStats}
          onSelectSource={ctx.updateSelectedPlatform}
        />
      ) : null}
      <section className="overview-tab__canvas overview-tab__hero">
        <div className="overview-tab__trend glass-panel rounded-2xl p-4">
          <div className="overview-tab__panel-head">
            <div className="overview-tab__trend-copy">
              <p className="overview-tab__eyebrow">{t('usage.dashboard.chart.trendEyebrow')}</p>
              <h3 className="overview-tab__panel-title">{t('usage.dashboard.chart.trendTitle')}</h3>
              <p className="overview-tab__panel-subtitle">{ctx.trendSubtitle}</p>
            </div>
            <span className="overview-tab__trend-chip">{ctx.trendGranularityLabel}</span>
          </div>
          <div className="overview-tab__trend-shell">
            {ctx.shouldRenderTrendChart && ctx.hasRenderableTrendData ? (
              <ApexChart
                className="overview-tab__chart"
                type="area"
                height="100%"
                options={ctx.trendOptions}
                series={ctx.trendSeries}
              />
            ) : (
              <div className="overview-tab__empty overview-tab__empty--trend">
                {ctx.hasRenderableTrendData
                  ? t('usage.dashboard.chart.preparingTrend')
                  : t('usage.dashboard.chart.noTrend')}
              </div>
            )}
          </div>
        </div>
        <aside className="overview-tab__distribution glass-panel rounded-xl p-4">
          <UsageModelDistributionCard
            title={t('usage.dashboard.chart.costByModel')}
            subtitle={ctx.distributionSubtitle}
            modelDistribution={ctx.modelDistribution}
            pieColors={ctx.pieColors}
            pieOptions={ctx.pieOptions}
            pieSeries={ctx.pieSeries}
            shouldRenderChart={ctx.shouldRenderDistributionChart}
            variant="embedded"
          />
        </aside>
      </section>
      <section className="overview-tab__insights-strip glass-panel rounded-xl p-4">
        {ctx.overviewHighlights.length > 0 ? (
          <div className="overview-tab__insight-grid">
            {ctx.overviewHighlights.map((item) => (
              <article key={item.id} className="overview-tab__insight-tile">
                <div className="overview-tab__insight-copy">
                  <span className="overview-tab__insight-label">{item.label}</span>
                  <strong className="overview-tab__insight-value" title={item.value}>{item.value}</strong>
                  <span className="overview-tab__insight-detail">{item.detail}</span>
                </div>
              </article>
            ))}
          </div>
        ) : (
          <div className="overview-tab__rank-empty">{t('usage.dashboard.table.noData')}</div>
        )}
      </section>
      <section className="overview-tab__rankings">
        {[
          { title: t('usage.dashboard.rankings.modelsTitle'), items: ctx.topModelRankings },
          { title: t('usage.dashboard.rankings.projectsTitle'), items: ctx.topProjectRankings },
        ].map((panel) => (
          <div key={panel.title} className="overview-tab__rank-panel glass-panel rounded-xl p-4">
            <h3 className="overview-tab__panel-title">{panel.title}</h3>
            {panel.items.length > 0 ? (
              <ol className="overview-tab__rank-list">
                {panel.items.map((item, index) => (
                  <li key={item.id} className="overview-tab__rank-item">
                    <span className="overview-tab__rank-index">{index + 1}</span>
                    <div className="overview-tab__rank-main">
                      <div className="overview-tab__rank-row">
                        <span className="overview-tab__rank-label" title={item.title}>{item.label}</span>
                        <strong className="overview-tab__rank-value">{item.value}</strong>
                        <span className="overview-tab__rank-share">{formatShare(item.share)}</span>
                      </div>
                      <span className="overview-tab__rank-detail">{item.detail}</span>
                      <div className="overview-tab__rank-bar">
                        <span style={{ width: `${Math.round(item.share * 100)}%` }} />
                      </div>
                    </div>
                  </li>
                ))}
              </ol>
            ) : (
              <div className="overview-tab__rank-empty">{t('usage.dashboard.table.noData')}</div>
            )}
          </div>
        ))}
      </section>
    </div>
  )
}
