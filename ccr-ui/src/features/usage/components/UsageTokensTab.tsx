import { memo, useCallback, useMemo, useState } from 'react'
import {
  getUsageTokenRowChartTotal,
  toUsageTokenBreakdownRows,
  type UsageTokenBreakdownMode,
} from '@/views/usage/usageTokenBreakdown'
import { buildChartAnimations, buildChartTheme } from '@/views/usage/usageChartOptions'
import { ApexChart } from '../charts/ApexChart'
import { useUsageDashboardContext } from '../UsageDashboardContext'
import { useUsageT } from '../translate'
import '../styles/usage-tokens-tab.css'

const MODES: UsageTokenBreakdownMode[] = ['breakdown', 'total']

export function UsageTokensTab() {
  const ctx = useUsageDashboardContext()
  const t = useUsageT()
  const [activeMode, setActiveMode] = useState<UsageTokenBreakdownMode>('breakdown')
  const theme = buildChartTheme()
  const rows = useMemo(() => toUsageTokenBreakdownRows(ctx.trends), [ctx.trends])
  const hasRows = rows.length > 0

  const chartSeries = useMemo(() => {
    if (activeMode === 'total') {
      return [{
        name: t('usage.dashboard.tokens.totalSeries'),
        data: rows.map((row) => getUsageTokenRowChartTotal(row)),
      }]
    }
    return [
      { name: t('usage.dashboard.chart.input'), data: rows.map((row) => row.inputTokens) },
      { name: t('usage.dashboard.chart.output'), data: rows.map((row) => row.assistantOutputTokens) },
      { name: t('usage.dashboard.chart.cacheRead'), data: rows.map((row) => row.cacheReadTokens) },
    ]
  }, [activeMode, rows, t])

  const chartOptions = useMemo(() => ({
    chart: {
      background: 'transparent',
      stacked: activeMode === 'breakdown',
      toolbar: { show: false },
      animations: buildChartAnimations(),
      redrawOnParentResize: false,
      redrawOnWindowResize: false,
    },
    theme: { mode: theme.mode },
    colors: [theme.inputToken, theme.outputToken, theme.cacheReadToken],
    dataLabels: { enabled: false },
    xaxis: {
      categories: rows.map((row) => row.date),
      labels: { style: { colors: theme.textMuted } },
    },
    yaxis: {
      labels: {
        style: { colors: theme.textMuted },
        formatter: (value: number) => ctx.formatTokens(value),
      },
    },
  }), [activeMode, ctx, rows, theme])

  return (
    <section className="tokens-tab">
      <article className="tokens-tab__chart-card glass-panel">
        <div className="tokens-tab__chart-head">
          <div>
            <p className="tokens-tab__eyebrow">{t('usage.dashboard.tokens.eyebrow')}</p>
            <h3>{t('usage.dashboard.tokens.title')}</h3>
          </div>
          <div className="tokens-tab__mode" role="tablist">
            {MODES.map((mode) => (
              <ModeButton
                key={mode}
                mode={mode}
                active={activeMode === mode}
                label={t(`usage.dashboard.tokens.modes.${mode}`)}
                onSelect={setActiveMode}
              />
            ))}
          </div>
        </div>
        {hasRows ? (
          <ApexChart type="bar" height={320} options={chartOptions} series={chartSeries} />
        ) : (
          <div className="tokens-tab__empty">{t('usage.dashboard.table.noData')}</div>
        )}
      </article>
    </section>
  )
}

const ModeButton = memo(function ModeButton({
  mode,
  active,
  label,
  onSelect,
}: {
  mode: UsageTokenBreakdownMode
  active: boolean
  label: string
  onSelect: (mode: UsageTokenBreakdownMode) => void
}) {
  const handleClick = useCallback(() => onSelect(mode), [mode, onSelect])
  return (
    <button
      type="button"
      aria-selected={active}
      className={['tokens-tab__mode-button', active ? 'tokens-tab__mode-button--active' : ''].filter(Boolean).join(' ')}
      onClick={handleClick}
    >
      {label}
    </button>
  )
})
