import { memo } from 'react'
import { Link } from 'react-router'
import { SIcon, cn } from '@/ui'
import { ocToneClass } from '../ui-classes'

export interface OpenCodeCapabilityItem {
  id: string
  title: string
  description: string
  href: string
  icon: string
  tone: 'lime' | 'violet' | 'cyan' | 'amber' | 'emerald'
  badge: string
  cta: string
  status: string
}

export const CapabilityCard = memo(function CapabilityCard({ item }: { item: OpenCodeCapabilityItem }) {
  return (
    <Link to={item.href} className="group block h-full">
      <article className="flex h-full flex-col justify-between gap-4 rounded-2xl border border-border-default/55 bg-bg-base p-4">
        <div className="flex items-center justify-between gap-3">
          <div className={cn('flex h-11 w-11 items-center justify-center rounded-2xl border', ocToneClass[item.tone])}>
            <SIcon name={item.icon} size="w-5 h-5" />
          </div>
          <span
            className={
              item.status === 'warning'
                ? 'rounded-full border border-accent-warning/30 bg-accent-warning/10 px-3 py-1 text-xs font-semibold text-accent-warning'
                : 'rounded-full border border-border-default/55 px-3 py-1 text-xs font-semibold text-text-secondary'
            }
          >
            {item.badge}
          </span>
        </div>
        <div>
          <h2 className="text-lg font-semibold text-text-primary">{item.title}</h2>
          <p className="mt-2 text-sm leading-6 text-text-secondary">{item.description}</p>
        </div>
        <div className="flex items-center justify-between text-sm text-text-muted">
          <span>{item.cta}</span>
          <SIcon name="ArrowRight" size="w-4 h-4" />
        </div>
      </article>
    </Link>
  )
})

export const ActionLink = memo(function ActionLink({
  href,
  label,
  detail,
}: {
  href: string
  label: string
  detail: string
}) {
  return (
    <Link to={href} className="flex flex-col rounded-2xl border border-border-default/55 bg-bg-base px-3 py-2">
      <span className="text-[0.6875rem] font-semibold uppercase tracking-wide text-text-muted">{label}</span>
      <strong className="text-sm text-text-primary">{detail}</strong>
    </Link>
  )
})
