import type { ApexOptions } from 'apexcharts'
import { ApexChart } from '../charts/ApexChart'
import { useUsageT } from '../translate'
import '../styles/usage-model-distribution-card.css'

interface DistributionSlice {
  id: string
  label: string
}

interface UsageModelDistributionCardProps {
  title: string
  subtitle?: string
  modelDistribution: DistributionSlice[]
  pieColors: string[]
  pieOptions: ApexOptions | Record<string, unknown>
  pieSeries: number[]
  shouldRenderChart: boolean
  variant?: 'panel' | 'embedded'
}

export function UsageModelDistributionCard({
  title,
  subtitle,
  modelDistribution,
  pieColors,
  pieOptions,
  pieSeries,
  shouldRenderChart,
  variant = 'embedded',
}: UsageModelDistributionCardProps) {
  const t = useUsageT()
  const hasData = shouldRenderChart && pieSeries.length > 0
  const hasDeferredData = !shouldRenderChart && pieSeries.length > 0

  return (
    <div
      className={[
        'distribution-card rounded-xl',
        variant === 'panel' ? 'distribution-card--panel glass-panel p-5' : 'distribution-card--embedded',
      ].join(' ')}
    >
      <div className="distribution-card__header">
        <div>
          <h3 className="distribution-card__title">{title}</h3>
          {subtitle ? <p className="distribution-card__subtitle">{subtitle}</p> : null}
        </div>
        {modelDistribution.length ? (
          <span className="distribution-card__badge">{modelDistribution.length}</span>
        ) : null}
      </div>
      <div className="distribution-card__body">
        <div className="distribution-card__chart-shell">
          {hasData ? (
            <ApexChart
              className="distribution-card__chart"
              type="donut"
              height={220}
              options={pieOptions}
              series={pieSeries}
            />
          ) : (
            <div className={['distribution-card__empty', hasDeferredData ? 'distribution-card__empty--deferred' : '']
              .filter(Boolean)
              .join(' ')}
            >
              {hasDeferredData
                ? t('usage.dashboard.chart.preparingDistribution')
                : t('usage.dashboard.table.noData')}
            </div>
          )}
        </div>
        {modelDistribution.length ? (
          <div className="distribution-card__legend">
            {modelDistribution.map((slice, index) => (
              <article key={slice.id} className="distribution-card__legend-item">
                <div className="distribution-card__legend-row">
                  <div className="distribution-card__legend-main">
                    <span
                      className="distribution-card__swatch"
                      style={{ backgroundColor: pieColors[index] || pieColors[0] }}
                    />
                    <div className="distribution-card__legend-copy">
                      <span className="distribution-card__label" title={slice.label}>
                        {slice.label}
                      </span>
                    </div>
                  </div>
                </div>
              </article>
            ))}
          </div>
        ) : null}
      </div>
    </div>
  )
}
