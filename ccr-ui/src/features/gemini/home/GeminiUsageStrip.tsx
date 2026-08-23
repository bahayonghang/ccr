import { useCallback, useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Link } from 'react-router'
import { getUsageDashboardV2 } from '@/api'
import { SIcon } from '@/ui'
import { t } from '../locale'

const last30Days = () => {
  const end = new Date()
  const start = new Date()
  start.setDate(end.getDate() - 29)
  const stamp = (date: Date) =>
    `${date.getFullYear()}-${`${date.getMonth() + 1}`.padStart(2, '0')}-${`${date.getDate()}`.padStart(2, '0')}`
  return { start: stamp(start), end: stamp(end) }
}

interface GeminiUsageStripProps {
  platform: 'antigravity' | 'opencode'
}

export function GeminiUsageStrip({ platform }: GeminiUsageStripProps) {
  const window = useMemo(() => last30Days(), [])
  const query = useQuery({
    queryKey: ['platform-usage-strip', platform, window.start, window.end],
    queryFn: () => getUsageDashboardV2(platform, window.start, window.end, 0, false),
    staleTime: 0,
  })
  const refresh = useCallback(() => {
    void query.refetch()
  }, [query])
  const summary = query.data?.summary
  const title = t(`platformUsage.platforms.${platform}.title`)
  const empty = !summary || (summary.total_requests === 0 && summary.total_tokens === 0)

  return (
    <section className="rounded-[1.75rem] border border-border-default/15 bg-bg-surface p-5" data-testid="platform-usage-insight">
      <div className="mb-4 flex items-start justify-between gap-3">
        <div>
          <p className="text-xs font-semibold uppercase tracking-wide text-text-muted">{t('platformUsage.eyebrow')}</p>
          <h2 className="text-lg font-semibold text-text-primary">{title}</h2>
        </div>
        <div className="flex gap-2">
          <button type="button" className="inline-flex items-center gap-1 rounded-lg border border-border-default px-3 py-1.5 text-sm" disabled={query.isFetching} onClick={refresh}>
            <SIcon name="RefreshCw" size="w-4 h-4" className={query.isFetching ? 'animate-spin' : undefined} />
            {t('platformUsage.retry')}
          </button>
          <Link to={`/usage?platform=${platform}`} className="inline-flex items-center gap-1 rounded-lg bg-accent-primary px-3 py-1.5 text-sm text-[color:var(--color-accent-primary-contrast)]">
            {t('platformUsage.openDashboard')}
            <SIcon name="ArrowUpRight" size="w-4 h-4" />
          </Link>
        </div>
      </div>
      {empty ? (
        <p className="text-sm text-text-muted">{t(`platformUsage.platforms.${platform}.emptyDescription`)}</p>
      ) : (
        <div className="grid gap-3 md:grid-cols-3">
          <UsageChip label={t('platformUsage.cards.cost')} value={`$${summary.total_cost_usd.toFixed(2)}`} />
          <UsageChip label={t('platformUsage.cards.tokens')} value={String(summary.total_tokens)} />
          <UsageChip label={t('platformUsage.cards.requests')} value={String(summary.total_requests)} />
        </div>
      )}
    </section>
  )
}

function UsageChip({ label, value }: { label: string; value: string }) {
  return (
    <article className="rounded-2xl border border-border-subtle bg-bg-elevated p-3">
      <span className="text-xs text-text-muted">{label}</span>
      <strong className="mt-1 block text-lg text-text-primary">{value}</strong>
    </article>
  )
}
