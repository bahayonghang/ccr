import { useMemo } from 'react'
import { ChartPreparingState } from '@/features/claude/observer/ChartPreparingState'
import { heatmapOptions, heatmapSeries } from '@/features/claude/observer/chartOptions'
import { formatUsd, shortenId, shortenPath } from '@/features/claude/observer/formatters'
import { ObserverChart } from '@/features/claude/observer/ObserverChart'
import { RankList } from '@/features/claude/observer/RankList'
import { t } from '@/features/claude/locale'
import type { HeatmapCell, SessionRow, TopToolRow } from '@/types/claudeObserver'

interface BehaviorAnalysisTabProps {
  heatmap: HeatmapCell[]
  topTools: TopToolRow[]
  sessions: SessionRow[]
  animationsEnabled: boolean
  shouldRenderChart: boolean
}

const cardClass =
  'min-w-0 rounded-[1.1rem] border border-[color:var(--surface-card-border)] bg-[var(--surface-card-bg)] p-4 shadow-[var(--surface-card-shadow)]'

function costPerTool(row: SessionRow): number {
  if (row.tool_call_count <= 0) return 0
  return row.cost_usd / row.tool_call_count
}

export function BehaviorAnalysisTab({
  heatmap,
  topTools,
  sessions,
  animationsEnabled,
  shouldRenderChart,
}: BehaviorAnalysisTabProps) {
  const motion = useMemo(() => ({ enabled: animationsEnabled }), [animationsEnabled])
  const series = useMemo(() => heatmapSeries(heatmap), [heatmap])
  const options = useMemo(() => heatmapOptions(motion), [motion])
  const toolRows = useMemo(
    () =>
      topTools.slice(0, 10).map((row) => ({
        key: row.tool_name,
        label: row.tool_name,
        title: row.tool_name,
        value: row.call_count.toLocaleString(),
        amount: row.call_count,
      })),
    [topTools],
  )
  const sessionRows = useMemo(() => sessions.slice(0, 10), [sessions])
  const empty = t('claudeCode.observer.empty.noTrend')
  const hasHeatmap = heatmap.length > 0

  return (
    <div className="grid gap-3.5" data-testid="claude-observer-behavior">
      <p className="m-0 rounded-xl border border-dashed border-border-default/20 bg-accent-info/5 px-3 py-2 text-xs leading-5 text-text-muted">
        {t('claudeCode.observer.behavior.sourceNote')}
      </p>
      <section className="grid gap-3.5 lg:grid-cols-[minmax(0,1.25fr)_minmax(0,1fr)]">
        <article className={cardClass}>
          <header className="mb-3 grid gap-0.5">
            <p className="text-sm font-semibold text-text-primary">{t('claudeCode.observer.chart.toolHeatmap')}</p>
            <p className="text-xs leading-5 text-text-secondary">{t('claudeCode.observer.chart.toolHeatmapSub')}</p>
          </header>
          <div className="relative h-65 min-w-0">
            {hasHeatmap && shouldRenderChart ? (
              <ObserverChart type="heatmap" height={260} options={options} series={series} />
            ) : hasHeatmap ? (
              <ChartPreparingState label={t('claudeCode.observer.chart.preparingHeatmap')} />
            ) : (
              <div className="flex min-h-50 items-center justify-center rounded-2xl border border-dashed border-border-default/25 text-sm text-text-muted">
                {empty}
              </div>
            )}
          </div>
        </article>
        <article className={cardClass}>
          <header className="mb-3 grid gap-0.5">
            <p className="text-sm font-semibold text-text-primary">{t('claudeCode.observer.chart.topTools')}</p>
            <p className="text-xs leading-5 text-text-secondary">{t('claudeCode.observer.chart.topToolsSub')}</p>
          </header>
          <RankList rows={toolRows} empty={empty} tone="info" />
        </article>
      </section>
      <section className={cardClass}>
        <header className="mb-3 grid gap-0.5">
          <p className="text-sm font-semibold text-text-primary">{t('claudeCode.observer.behavior.efficiencyTitle')}</p>
          <p className="text-xs leading-5 text-text-secondary">{t('claudeCode.observer.behavior.efficiencySub')}</p>
        </header>
        {sessionRows.length > 0 ? (
          <div className="w-full overflow-x-auto">
            <table className="w-full border-collapse text-sm">
              <thead>
                <tr>
                  <th className="border-b border-border-default/15 px-3 py-2 text-left text-[0.72rem] font-semibold tracking-wide text-text-muted uppercase">
                    {t('claudeCode.observer.behavior.colSession')}
                  </th>
                  <th className="border-b border-border-default/15 px-3 py-2 text-left text-[0.72rem] font-semibold tracking-wide text-text-muted uppercase">
                    {t('claudeCode.observer.behavior.colProject')}
                  </th>
                  <th className="border-b border-border-default/15 px-3 py-2 text-right text-[0.72rem] font-semibold tracking-wide text-text-muted uppercase">
                    {t('claudeCode.observer.behavior.colCost')}
                  </th>
                  <th className="border-b border-border-default/15 px-3 py-2 text-right text-[0.72rem] font-semibold tracking-wide text-text-muted uppercase">
                    {t('claudeCode.observer.behavior.colTools')}
                  </th>
                  <th className="border-b border-border-default/15 px-3 py-2 text-right text-[0.72rem] font-semibold tracking-wide text-text-muted uppercase">
                    {t('claudeCode.observer.behavior.colCostPerTool')}
                  </th>
                </tr>
              </thead>
              <tbody>
                {sessionRows.map((row) => (
                  <tr key={row.session_id}>
                    <td className="border-b border-border-default/15 px-3 py-2 font-mono text-xs text-text-secondary" title={row.session_id}>
                      {shortenId(row.session_id)}
                    </td>
                    <td className="border-b border-border-default/15 px-3 py-2 text-text-primary" title={row.project_path ?? ''}>
                      {shortenPath(row.project_path ?? '—', 36)}
                    </td>
                    <td className="border-b border-border-default/15 px-3 py-2 text-right tabular-nums">{formatUsd(row.cost_usd)}</td>
                    <td className="border-b border-border-default/15 px-3 py-2 text-right tabular-nums">
                      {row.tool_call_count.toLocaleString()}
                    </td>
                    <td className="border-b border-border-default/15 px-3 py-2 text-right tabular-nums">{formatUsd(costPerTool(row))}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <div className="flex min-h-30 items-center justify-center rounded-2xl border border-dashed border-border-default/25 text-sm text-text-muted">
            {empty}
          </div>
        )}
      </section>
    </div>
  )
}
