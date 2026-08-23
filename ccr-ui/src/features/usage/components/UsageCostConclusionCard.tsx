import type { ReactNode } from 'react'
import { SIcon } from '@/ui'
import type { UsageSummaryCard } from '@/views/usage/usageSummaryCards'
import { useUsageT } from '../translate'
import '../styles/usage-cost-conclusion-card.css'

interface UsageCostConclusionCardProps {
  card: UsageSummaryCard
  children?: ReactNode
}

export function UsageCostConclusionCard({ card, children }: UsageCostConclusionCardProps) {
  const t = useUsageT()

  return (
    <article className={`usage-cost-conclusion usage-cost-conclusion--${card.tone}`}>
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
      {children ? <div className="usage-cost-conclusion__embed">{children}</div> : null}
    </article>
  )
}
