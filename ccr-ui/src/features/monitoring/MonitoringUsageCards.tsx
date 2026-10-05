import { memo } from 'react'
import { SIcon } from '@/ui'

export interface UsageMetricCard {
  id: string
  label: string
  value: string
  detail: string
  icon: string
}

interface MonitoringUsageCardsProps {
  cards: UsageMetricCard[]
}

const UsageCard = memo(function UsageCard({ card }: { card: UsageMetricCard }) {
  return (
    <div className="rounded-2xl border border-border-default/45 bg-bg-elevated p-3" data-testid={`monitoring-usage-card-${card.id}`}>
      <div className="flex items-center justify-between gap-3">
        <p className="text-xs font-medium text-text-muted">{card.label}</p>
        <SIcon name={card.icon} size="w-4 h-4" className="text-text-muted" />
      </div>
      <p className="mt-2 text-xl font-semibold tabular-nums tracking-tight text-text-primary">{card.value}</p>
      <p className="mt-1 min-h-5 text-xs leading-5 text-text-muted">{card.detail}</p>
    </div>
  )
})

export const MonitoringUsageCards = memo(function MonitoringUsageCards({ cards }: MonitoringUsageCardsProps) {
  return (
    <div className="mt-3 grid grid-cols-2 gap-2">
      {cards.map((card) => (
        <UsageCard key={card.id} card={card} />
      ))}
    </div>
  )
})
