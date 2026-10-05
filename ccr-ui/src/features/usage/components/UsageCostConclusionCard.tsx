import type { ReactNode } from 'react'
import { SIcon } from '@/ui'
import type { UsageSummaryCard } from '@/views/usage/usageSummaryCards'
import { Sparkline } from '../Sparkline'
import { useUsageT } from '../translate'
import '../styles/usage-cost-conclusion-card.css'

interface UsageCostConclusionCardProps {
  card: UsageSummaryCard
  children?: ReactNode
}

export function UsageCostConclusionCard({ card, children }: UsageCostConclusionCardProps) {
  const t = useUsageT()
  const sparklineValues = card.sparkline.map((point) => point.value)

  return (
    <article className={`usage-cost-conclusion usage-cost-conclusion--${card.tone}`}>
      <div className="usage-cost-conclusion__main">
        <div className="usage-cost-conclusion__identity">
          <div className="usage-cost-conclusion__head">
            <span className="usage-cost-conclusion__icon">
              <SIcon name={card.icon} size="w-4 h-4" />
            </span>
            <span className="usage-cost-conclusion__label">{card.label}</span>
          </div>
          <div className="usage-cost-conclusion__value-row">
            <strong className="usage-cost-conclusion__value">{card.value}</strong>
            <span className={`usage-cost-conclusion__delta usage-cost-conclusion__delta--${card.deltaSentiment}`}>
              {card.deltaLabel}
              <small>{t('usage.dashboard.cards.periodOverPeriod')}</small>
            </span>
          </div>
          <p className="usage-cost-conclusion__detail">{card.detail}</p>
        </div>
        <div className="usage-cost-conclusion__trend">
          <div className="usage-cost-conclusion__sparkline">
            <Sparkline
              className="usage-cost-conclusion__spark"
              values={sparklineValues}
              label={card.sparklineLabel}
            />
          </div>
          <dl className="usage-cost-conclusion__stats">
            <div>
              <dt>{t('usage.dashboard.cards.average')}</dt>
              <dd>{card.averageLabel}</dd>
            </div>
            <div>
              <dt>{t('usage.dashboard.cards.peak')}</dt>
              <dd>{card.peakLabel}</dd>
            </div>
          </dl>
        </div>
      </div>
      {children ? <div className="usage-cost-conclusion__embed">{children}</div> : null}
    </article>
  )
}
