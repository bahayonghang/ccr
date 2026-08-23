import { useMemo } from 'react'
import { ChartPreparingState } from '@/features/claude/observer/ChartPreparingState'
import { tokenStackOptions, tokenStackSeries } from '@/features/claude/observer/chartOptions'
import { formatPercent, formatTokens } from '@/features/claude/observer/formatters'
import { ObserverChart } from '@/features/claude/observer/ObserverChart'
import { t } from '@/features/claude/locale'
import type { CacheStatsDto, DailyPoint } from '@/types/claudeObserver'

interface TokenDetailTabProps {
  stats: CacheStatsDto | null
  daily: DailyPoint[]
  animationsEnabled: boolean
  shouldRenderChart: boolean
}

const statCard =
  'grid gap-1 rounded-2xl border border-[color:var(--surface-card-border)] bg-[var(--surface-card-bg)] p-4'

export function TokenDetailTab({
  stats,
  daily,
  animationsEnabled,
  shouldRenderChart,
}: TokenDetailTabProps) {
  const motion = useMemo(() => ({ enabled: animationsEnabled }), [animationsEnabled])
  const series = useMemo(() => tokenStackSeries(daily), [daily])
  const options = useMemo(() => tokenStackOptions(motion), [motion])
  const hasDaily = daily.length > 0

  return (
    <div className="grid gap-3.5" data-testid="claude-observer-token">
      <section className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
        <article className={statCard}>
          <p className="text-[0.7rem] font-bold tracking-wide text-text-muted uppercase">
            {t('claudeCode.observer.metric.cacheHitRate')}
          </p>
          <p className="text-2xl font-semibold tabular-nums text-text-primary">
            {formatPercent(stats?.hit_rate ?? 0)}
          </p>
          <p className="text-xs text-text-secondary">{t('claudeCode.observer.metric.cacheHitRateDetail')}</p>
        </article>
        <article className={statCard}>
          <p className="text-[0.7rem] font-bold tracking-wide text-text-muted uppercase">
            {t('claudeCode.observer.metric.inputUncached')}
          </p>
          <p className="text-2xl font-semibold tabular-nums text-text-primary">
            {formatTokens(stats?.total_input_tokens ?? 0)}
          </p>
          <p className="text-xs text-text-secondary">{t('claudeCode.observer.metric.inputUncachedDetail')}</p>
        </article>
        <article className={statCard}>
          <p className="text-[0.7rem] font-bold tracking-wide text-text-muted uppercase">
            {t('claudeCode.observer.metric.output')}
          </p>
          <p className="text-2xl font-semibold tabular-nums text-text-primary">
            {formatTokens(stats?.total_output_tokens ?? 0)}
          </p>
          <p className="text-xs text-text-secondary">{t('claudeCode.observer.metric.outputDetail')}</p>
        </article>
        <article className={statCard}>
          <p className="text-[0.7rem] font-bold tracking-wide text-text-muted uppercase">
            {t('claudeCode.observer.metric.cacheRead')}
          </p>
          <p className="text-2xl font-semibold tabular-nums text-text-primary">
            {formatTokens(stats?.total_cache_read_tokens ?? 0)}
          </p>
          <p className="text-xs text-text-secondary">{t('claudeCode.observer.metric.cacheReadDetail')}</p>
        </article>
      </section>
      <section className="rounded-[1.1rem] border border-[color:var(--surface-card-border)] bg-[var(--surface-card-bg)] p-4 shadow-[var(--surface-card-shadow)]">
        <header className="mb-3 grid gap-0.5">
          <p className="text-sm font-semibold text-text-primary">{t('claudeCode.observer.chart.dailyTokens30')}</p>
          <p className="text-xs leading-5 text-text-secondary">{t('claudeCode.observer.chart.dailyTokens30Sub')}</p>
        </header>
        <div className="relative h-70 min-w-0">
          {hasDaily && shouldRenderChart ? (
            <ObserverChart type="bar" height={280} options={options} series={series} />
          ) : hasDaily ? (
            <ChartPreparingState label={t('claudeCode.observer.chart.preparingTrend')} />
          ) : (
            <div className="flex min-h-50 items-center justify-center rounded-2xl border border-dashed border-border-default/25 text-sm text-text-muted">
              {t('claudeCode.observer.empty.noTrend')}
            </div>
          )}
        </div>
      </section>
      <section className="grid gap-1 rounded-2xl border border-accent-warning/20 bg-accent-warning/10 p-4">
        <p className="text-sm font-semibold text-text-primary">
          {t('claudeCode.observer.tokenDetail.cacheWriteExplainTitle')}
        </p>
        <p className="text-xs leading-6 text-text-secondary">{t('claudeCode.observer.tokenDetail.cacheWriteExplain')}</p>
      </section>
    </div>
  )
}
