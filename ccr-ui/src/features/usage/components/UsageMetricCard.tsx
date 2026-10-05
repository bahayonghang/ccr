import { SIcon } from '@/ui'
import type { UsageSummaryCard } from '@/views/usage/usageSummaryCards'
import { Sparkline } from '../Sparkline'
import { useUsageT } from '../translate'
import '../styles/usage-metric-card.css'

interface UsageMetricCardProps {
  card: UsageSummaryCard
}

export function UsageMetricCard({ card }: UsageMetricCardProps) {
  const t = useUsageT()
  const sparklineValues = card.sparkline.map((point) => point.value)

  return (
    <article className={`usage-metric-card usage-metric-card--${card.tone}`}>
      <div className="usage-metric-card__topline">
        <span className="usage-metric-card__icon">
          <SIcon name={card.icon} size="w-4 h-4" />
        </span>
        <span className="usage-metric-card__label">{card.label}</span>
      </div>
      <div className="usage-metric-card__body">
        <strong className="usage-metric-card__value">{card.value}</strong>
        <span className={`usage-metric-card__delta usage-metric-card__delta--${card.deltaSentiment}`}>
          {card.deltaLabel}
        </span>
      </div>
      <p className="usage-metric-card__detail">{card.detail}</p>
      <div className="usage-metric-card__sparkline">
        <Sparkline
          className="usage-metric-card__spark"
          values={sparklineValues}
          label={card.sparklineLabel}
        />
      </div>
      <dl className="usage-metric-card__stats">
        <div>
          <dt>{t('usage.dashboard.cards.average')}</dt>
          <dd>{card.averageLabel}</dd>
        </div>
        <div>
          <dt>{t('usage.dashboard.cards.peak')}</dt>
          <dd>{card.peakLabel}</dd>
        </div>
      </dl>
    </article>
  )
}
