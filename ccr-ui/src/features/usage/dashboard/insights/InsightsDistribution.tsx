import { useCallback, useId, useMemo, useState, type CSSProperties } from 'react'
import type { HomeInsightsResponse } from '@/types/generated/usage/HomeInsightsResponse'
import { useUsageT } from '../../translate'
import { InsightsSegmented } from './InsightsSegmented'
import {
  buildDistribution,
  busiestHour,
  fullNumber,
  type DistributionView,
} from './insightsPresentation'

const VIEW_KEYS: Record<DistributionView, string> = {
  hour: 'dashboard.insights.distribution.hour',
  weekday: 'dashboard.insights.distribution.weekday',
  month: 'dashboard.insights.distribution.month',
}
const GRIDLINES = [25, 50, 75] as const

export function InsightsDistribution({ data }: { data: HomeInsightsResponse }) {
  const t = useUsageT()
  const titleId = useId()
  const [view, setView] = useState<DistributionView>('hour')
  const handleView = useCallback((next: DistributionView) => setView(next), [])
  const columns = useMemo(
    () =>
      buildDistribution(data, view).map((bar) => ({
        ...bar,
        text: bar.translate ? t(bar.label) : bar.label,
      })),
    [data, view, t]
  )
  const max = Math.max(0, ...columns.map((column) => column.value))
  // 说明行只描述小时口径；星期/月份视图下图上数据已换口径，说明行随之隐藏。
  const peakHour = useMemo(() => (view === 'hour' ? busiestHour(data) : null), [data, view])
  const options = useMemo(
    () =>
      (Object.keys(VIEW_KEYS) as DistributionView[]).map((value) => ({
        value,
        label: t(VIEW_KEYS[value]),
      })),
    [t]
  )

  return (
    <section className="insights-panel insights-distribution" aria-labelledby={titleId}>
      <header className="insights-panel__header">
        <h3 id={titleId} className="insights-panel__title">{t('dashboard.insights.distribution.title')}</h3>
        <InsightsSegmented
          options={options}
          value={view}
          onChange={handleView}
          ariaLabel={t('dashboard.insights.distribution.viewLabel')}
        />
      </header>
      <div
        className="insights-distribution__chart"
        role="img"
        aria-label={t('dashboard.insights.distribution.ariaLabel', { view: t(VIEW_KEYS[view]) })}
      >
        <div className="insights-distribution__plot">
          {GRIDLINES.map((line) => (
            <span
              key={line}
              className="insights-gridline"
              style={{ bottom: `${line}%` } as CSSProperties}
              aria-hidden="true"
            />
          ))}
          {columns.map((column) => (
            <span key={column.label} className="insights-distribution__track">
              <span
                className="insights-distribution__bar"
                data-peak={column.peak || undefined}
                style={{ height: `${max > 0 ? (column.value / max) * 100 : 0}%` } as CSSProperties}
                title={t('dashboard.insights.distribution.barTitle', {
                  label: column.text,
                  count: fullNumber(column.value),
                })}
              />
            </span>
          ))}
        </div>
        <div className="insights-distribution__labels" aria-hidden="true">
          {columns.map((column) => (
            <span key={column.label} className="insights-distribution__label">
              {column.text}
            </span>
          ))}
        </div>
      </div>
      {peakHour !== null ? (
        <p className="insights-caption" data-testid="insights-distribution-caption">
          {t('dashboard.insights.distribution.busiestHour', { start: peakHour, end: peakHour + 1 })}
        </p>
      ) : null}
    </section>
  )
}
