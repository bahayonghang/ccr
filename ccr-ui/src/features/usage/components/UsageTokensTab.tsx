import { memo, useCallback, useMemo, useRef, useState } from 'react'
import {
  getUsageTokenRowChartTotal,
  toUsageTokenBreakdownRows,
  type UsageTokenBreakdownMode,
} from '@/views/usage/usageTokenBreakdown'
import { buildChartTheme, getTrendTickAmount } from '@/views/usage/usageChartOptions'
import {
  buildDailyBarChartOptions,
  stabilizeDailyBarSeries,
  toDailyBarPoints,
  type DailyBarSeries,
} from '@/views/usage/usageDailyBarChart'
import { ApexChart } from '../charts/ApexChart'
import { useUsageDashboardContext } from '../UsageDashboardContext'
import { useUsageT } from '../translate'
import { UsageLedger } from './UsageLedger'
import { tokenLedgerColumns, tokenLedgerRows } from './usageLedgerRows'
import '../styles/usage-tokens-tab.css'

const MODES: UsageTokenBreakdownMode[] = ['breakdown', 'total']

export function UsageTokensTab() {
  const ctx = useUsageDashboardContext()
  const t = useUsageT()
  const [activeMode, setActiveMode] = useState<UsageTokenBreakdownMode>('breakdown')
  const theme = ctx.chartTheme ?? buildChartTheme()
  const locale = ctx.locale || 'zh-CN'
  const stacked = activeMode === 'breakdown'
  const rows = useMemo(() => toUsageTokenBreakdownRows(ctx.trends), [ctx.trends])
  const hasRows = rows.length > 0
  const previousSeries = useRef<DailyBarSeries[] | undefined>(undefined)

  const chartSeries = useMemo(() => {
    const next: DailyBarSeries[] = stacked
      ? [
          { name: t('usage.dashboard.chart.input'), data: toDailyBarPoints(rows, (row) => row.inputTokens) },
          {
            name: t('usage.dashboard.chart.output'),
            data: toDailyBarPoints(rows, (row) => row.assistantOutputTokens),
          },
          {
            name: t('usage.dashboard.chart.cacheRead'),
            data: toDailyBarPoints(rows, (row) => row.cacheReadTokens),
          },
        ]
      : [{
          name: t('usage.dashboard.tokens.totalSeries'),
          data: toDailyBarPoints(rows, getUsageTokenRowChartTotal),
        }]
    const stable = stabilizeDailyBarSeries(previousSeries.current, next)
    previousSeries.current = stable
    return stable
  }, [rows, stacked, t])

  const chartOptions = useMemo(
    () =>
      buildDailyBarChartOptions({
        theme,
        locale,
        granularity: 'day',
        tickAmount: getTrendTickAmount(rows.length),
        stacked,
        palette: 'tokens',
        formatY: ctx.formatTokens,
      }),
    [ctx.formatTokens, locale, rows.length, stacked, theme],
  )

  const ledgerColumns = useMemo(() => tokenLedgerColumns(t), [t])
  const ledgerRows = useMemo(
    () => tokenLedgerRows(rows, ctx.formatTokens),
    [ctx.formatTokens, rows],
  )

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
      {hasRows ? (
        <article className="tokens-tab__table-card glass-panel">
          <UsageLedger columns={ledgerColumns} maxHeight="34rem" rows={ledgerRows} />
        </article>
      ) : null}
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
