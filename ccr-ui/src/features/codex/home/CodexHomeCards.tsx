import { memo } from 'react'
import { Link } from 'react-router'
import { SIcon, cn } from '@/ui'
import type { CodexDashboardActionItem, CodexDashboardHealthItem, CodexDashboardInventoryItem } from '../dashboard-model'
import { toneIconClass } from '../ui-classes'

export const ReadinessCard = memo(function ReadinessCard({ item }: { item: CodexDashboardHealthItem }) {
  return (
    <Link
      to={item.to}
      className="relative flex min-h-[13rem] overflow-hidden rounded-3xl border border-[color:var(--stage-border-soft)] bg-[var(--stage-surface-soft)] p-4 transition-all duration-200 hover:-translate-y-px hover:border-accent-primary/22"
    >
      <div className="flex min-w-0 flex-1 flex-col">
        <div className="mb-4 flex items-center justify-between gap-3">
          <div className={cn('flex h-9 w-9 shrink-0 items-center justify-center rounded-xl border', toneIconClass[item.tone])}>
            <SIcon name={item.icon} size="w-5 h-5" />
          </div>
          <span className="rounded-md border border-[color:var(--stage-chip-neutral-border)] bg-[var(--stage-chip-neutral-bg)] px-2.5 py-1 text-[0.68rem] font-medium text-[color:var(--stage-chip-neutral-text)]">
            {item.statusLabel}
          </span>
        </div>
        <p className="text-xs font-medium text-[color:var(--stage-text-quiet)]">{item.title}</p>
        <p className="mt-2 break-words text-lg font-semibold leading-snug text-[color:var(--stage-text-primary)]">{item.value}</p>
        <p className="mt-auto pt-3 text-sm leading-6 text-[color:var(--stage-text-secondary)]">{item.detail}</p>
      </div>
      <SIcon name="ArrowUpRight" size="w-4 h-4" className="absolute right-4 top-4 text-[color:var(--stage-text-quiet)] opacity-0 transition-opacity duration-200 group-hover:opacity-100" />
    </Link>
  )
})

export const NextActionRow = memo(function NextActionRow({
  action,
  index,
}: {
  action: CodexDashboardActionItem
  index: number
}) {
  return (
    <Link
      to={action.to}
      className={cn(
        'relative grid grid-cols-[auto_auto_1fr_auto] items-start gap-3 rounded-3xl border border-[color:var(--stage-border-soft)] bg-[var(--stage-surface-soft)] p-4 transition-all duration-200 hover:-translate-y-px hover:border-accent-primary/22',
        action.tone === 'danger' && 'border-accent-danger/20',
        action.tone === 'warning' && 'border-accent-warning/18',
      )}
    >
      <span className="pt-2 text-xs font-semibold tracking-[0.14em] text-[color:var(--stage-text-quiet)]">{index + 1}</span>
      <div className={cn('flex h-11 w-11 shrink-0 items-center justify-center rounded-xl border', toneIconClass[action.tone])}>
        <SIcon name={action.icon} size="w-5 h-5" />
      </div>
      <div className="min-w-0">
        <h3 className="text-base font-semibold text-[color:var(--stage-text-primary)]">{action.title}</h3>
        <p className="mt-1 text-sm leading-6 text-[color:var(--stage-text-secondary)]">{action.description}</p>
      </div>
      <SIcon name="ArrowRight" size="w-4 h-4" className="mt-3 text-[color:var(--stage-text-quiet)]" />
    </Link>
  )
})

export const ManageRow = memo(function ManageRow({ item }: { item: CodexDashboardInventoryItem }) {
  return (
    <Link
      to={item.to}
      className="flex items-center gap-3 rounded-2xl border border-[color:var(--stage-border-soft)] bg-[var(--stage-surface-soft)] px-3 py-3 transition-all duration-200 hover:translate-x-px hover:border-accent-primary/20"
    >
      <div className={cn('flex h-9 w-9 shrink-0 items-center justify-center rounded-xl border', toneIconClass[item.tone])}>
        <SIcon name={item.icon} size="w-5 h-5" />
      </div>
      <div className="min-w-0 flex-1">
        <span className="block truncate text-sm font-semibold text-[color:var(--stage-text-primary)]">{item.title}</span>
        <small className="mt-0.5 block truncate text-xs text-[color:var(--stage-text-muted)]">{item.detail}</small>
      </div>
      <strong className="max-w-[7rem] truncate text-sm font-semibold text-[color:var(--stage-text-secondary)]">{item.value}</strong>
    </Link>
  )
})
