import { useCallback } from 'react'
import { Link } from 'react-router'
import { formatInsightUsd, formatRoi, formatTokens } from '@/features/claude/observer/formatters'
import { t } from '@/features/claude/locale'
import { SIcon } from '@/ui'
import type { InsightDto, SubscriptionDto } from '@/types/claudeObserver'

type TabId = 'cost' | 'token' | 'behavior'

export function ObserverHeader({ pricingNote }: { pricingNote: string }) {
  return (
    <header className="flex flex-wrap items-end justify-between gap-3">
      <div className="grid min-w-0 gap-0.5">
        <p className="text-[0.7rem] font-bold tracking-[0.12em] text-text-muted uppercase">
          {t('claudeCode.observer.eyebrow')}
        </p>
        <h2 className="text-[1.4rem] font-semibold text-text-primary">{t('claudeCode.observer.title')}</h2>
        <p className="text-xs leading-5 text-text-secondary">{pricingNote}</p>
      </div>
      <Link
        to="/usage"
        className="inline-flex items-center rounded-xl border border-border-default px-3 py-2 text-xs font-semibold text-text-secondary no-underline hover:border-accent-primary/30 hover:bg-accent-primary/5 hover:text-text-primary"
      >
        {t('claudeCode.observer.fullDashboardLink')}
      </Link>
    </header>
  )
}

export function ObserverSubscriptionBar({
  showBanner,
  subscription,
  roi,
  onOpen,
}: {
  showBanner: boolean
  subscription: SubscriptionDto | null
  roi: number | null
  onOpen: () => void
}) {
  if (showBanner) {
    return (
      <div className="flex items-center justify-between gap-3 rounded-2xl border border-border-default px-3 py-2">
        <div className="flex items-center gap-2 text-sm font-semibold text-text-primary">
          <SIcon name="Sparkles" size="w-4 h-4" className="text-accent-primary" />
          <span>
            {t('claudeCode.observer.subscription.banner', {
              monthly: formatInsightUsd(subscription?.monthly_usd ?? 0),
              roi: formatRoi(roi),
            })}
          </span>
        </div>
        <button
          type="button"
          className="inline-flex items-center gap-1 rounded-xl border border-border-default px-2 py-1 text-xs text-text-secondary"
          title={t('claudeCode.observer.subscription.openDialog')}
          onClick={onOpen}
        >
          <SIcon name="Settings" size="w-4 h-4" />
        </button>
      </div>
    )
  }
  return (
    <div className="flex justify-end">
      <button
        type="button"
        className="inline-flex items-center gap-1 rounded-xl border border-border-default px-3 py-1 text-xs text-text-secondary"
        title={t('claudeCode.observer.subscription.openDialog')}
        onClick={onOpen}
      >
        <SIcon name="Settings" size="w-4 h-4" />
        <span>{t('claudeCode.observer.subscription.openDialog')}</span>
      </button>
    </div>
  )
}

function HeroCard({
  label,
  value,
  detail,
  accent,
}: {
  label: string
  value: string
  detail: string
  accent?: boolean
}) {
  const className = accent
    ? 'grid gap-1 rounded-[1.1rem] border border-accent-primary/20 bg-accent-primary/10 px-4 py-3'
    : 'grid gap-1 rounded-[1.1rem] border border-[color:var(--surface-card-border)] bg-[var(--surface-card-bg)] px-4 py-3 shadow-[var(--surface-card-shadow)]'
  return (
    <article className={className}>
      <p className="text-[0.72rem] font-bold tracking-wide text-text-muted uppercase">{label}</p>
      <p className="text-[1.85rem] font-semibold leading-tight tabular-nums text-text-primary">{value}</p>
      <p className="text-xs text-text-secondary">{detail}</p>
    </article>
  )
}

function monthDetailText(insight: InsightDto | undefined, subscription: SubscriptionDto | null, hasRoi: boolean): string {
  if (hasRoi) {
    return t('claudeCode.observer.metric.monthValueDetail', {
      monthly: formatInsightUsd(subscription?.monthly_usd ?? 0),
      roi: formatRoi(insight?.roi ?? null),
    })
  }
  return `${formatTokens(insight?.month_tokens ?? 0)} ${t('claudeCode.observer.metric.tokensUnit')}`
}

export function ObserverHeroGrid({
  insight,
  subscription,
  hasRoi,
}: {
  insight: InsightDto | undefined
  subscription: SubscriptionDto | null
  hasRoi: boolean
}) {
  return (
    <section className="grid gap-3 md:grid-cols-3">
      <HeroCard
        label={t('claudeCode.observer.metric.todayValue')}
        value={formatInsightUsd(insight?.today_value_usd ?? 0)}
        detail={`${formatTokens(insight?.today_tokens ?? 0)} ${t('claudeCode.observer.metric.tokensUnit')}`}
      />
      <HeroCard
        label={t('claudeCode.observer.metric.monthValue')}
        value={formatInsightUsd(insight?.month_value_usd ?? 0)}
        detail={monthDetailText(insight, subscription, hasRoi)}
        accent
      />
      <HeroCard
        label={t('claudeCode.observer.metric.totalValue')}
        value={formatInsightUsd(insight?.total_value_usd ?? 0)}
        detail={t('claudeCode.observer.metric.totalValueDetail', {
          sessions: insight?.total_sessions ?? 0,
          projects: insight?.total_projects ?? 0,
        })}
      />
    </section>
  )
}

export function ObserverTabButton({
  id,
  label,
  active,
  onSelect,
}: {
  id: TabId
  label: string
  active: boolean
  onSelect: (id: TabId) => void
}) {
  const handleClick = useCallback(() => {
    onSelect(id)
  }, [id, onSelect])
  const className = active
    ? 'rounded-2xl border border-accent-primary/20 bg-bg-surface px-3 py-2 text-sm font-semibold text-text-primary'
    : 'rounded-2xl border border-transparent bg-transparent px-3 py-2 text-sm font-semibold text-text-secondary hover:bg-[var(--surface-status-bg)] hover:text-text-primary'
  return (
    <button type="button" className={className} onClick={handleClick}>
      {label}
    </button>
  )
}

export type { TabId }
