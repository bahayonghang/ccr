import { useMemo } from 'react'
import { ChartPreparingState } from '@/features/claude/observer/ChartPreparingState'
import { dailyCostOptions, dailyCostSeries } from '@/features/claude/observer/chartOptions'
import { formatUsd, shortenPath } from '@/features/claude/observer/formatters'
import { ObserverChart } from '@/features/claude/observer/ObserverChart'
import { RankList } from '@/features/claude/observer/RankList'
import { t } from '@/features/claude/locale'
import type { BreakdownRow, DailyPoint } from '@/types/claudeObserver'

interface CostAttributionTabProps {
  daily: DailyPoint[]
  byProject: BreakdownRow[]
  byModel: BreakdownRow[]
  animationsEnabled: boolean
  shouldRenderChart: boolean
}

const cardClass =
  'rounded-[1.1rem] border border-[color:var(--surface-card-border)] bg-[var(--surface-card-bg)] p-4 shadow-[var(--surface-card-shadow)]'

export function CostAttributionTab({
  daily,
  byProject,
  byModel,
  animationsEnabled,
  shouldRenderChart,
}: CostAttributionTabProps) {
  const motion = useMemo(() => ({ enabled: animationsEnabled }), [animationsEnabled])
  const series = useMemo(() => dailyCostSeries(daily), [daily])
  const options = useMemo(() => dailyCostOptions(motion), [motion])
  const projectRows = useMemo(
    () =>
      (Array.isArray(byProject) ? byProject : []).slice(0, 10).map((row) => ({
        key: row.key,
        label: shortenPath(row.key),
        title: row.key,
        value: formatUsd(row.cost_usd),
        amount: row.cost_usd,
      })),
    [byProject],
  )
  const modelRows = useMemo(
    () =>
      (Array.isArray(byModel) ? byModel : []).slice(0, 10).map((row) => ({
        key: row.key,
        label: row.key,
        value: formatUsd(row.cost_usd),
        amount: row.cost_usd,
      })),
    [byModel],
  )
  const empty = t('claudeCode.observer.empty.noTrend')
  const hasDaily = daily.length > 0

  return (
    <div className="grid gap-3.5" data-testid="claude-observer-cost">
      <section className={cardClass}>
        <header className="mb-3 grid gap-0.5">
          <p className="text-sm font-semibold text-text-primary">{t('claudeCode.observer.chart.dailyTrend30')}</p>
          <p className="text-xs leading-5 text-text-secondary">{t('claudeCode.observer.chart.dailyTrend30Sub')}</p>
        </header>
        <div className="relative h-65 min-w-0">
          {hasDaily && shouldRenderChart ? (
            <ObserverChart type="area" height={260} options={options} series={series} />
          ) : hasDaily ? (
            <ChartPreparingState label={t('claudeCode.observer.chart.preparingTrend')} />
          ) : (
            <div className="flex min-h-50 items-center justify-center rounded-2xl border border-dashed border-border-default/25 text-sm text-text-muted">
              {empty}
            </div>
          )}
        </div>
      </section>
      <section className="grid gap-3.5 md:grid-cols-2">
        <article className={cardClass}>
          <header className="mb-3 grid gap-0.5">
            <p className="text-sm font-semibold text-text-primary">{t('claudeCode.observer.chart.byProject')}</p>
            <p className="text-xs leading-5 text-text-secondary">{t('claudeCode.observer.chart.byProjectSub')}</p>
          </header>
          <RankList rows={projectRows} empty={empty} tone="primary" />
        </article>
        <article className={cardClass}>
          <header className="mb-3 grid gap-0.5">
            <p className="text-sm font-semibold text-text-primary">{t('claudeCode.observer.chart.byModel')}</p>
            <p className="text-xs leading-5 text-text-secondary">{t('claudeCode.observer.chart.byModelSub')}</p>
          </header>
          <RankList rows={modelRows} empty={empty} tone="secondary" />
        </article>
      </section>
    </div>
  )
}
