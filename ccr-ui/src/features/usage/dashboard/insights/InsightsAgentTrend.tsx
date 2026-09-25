import { useCallback, useId, useMemo, useState, type CSSProperties } from 'react'
import type { HomeInsightsResponse } from '@/types/generated/usage/HomeInsightsResponse'
import { useUsageT } from '../../translate'
import { InsightsSegmented } from './InsightsSegmented'
import { buildTrend, buildTrendTable, fullNumber, type TrendModel } from './insightsPresentation'
import { INSIGHTS_OTHER_KEY, sourceColor, sourceLabel } from './insightsSources'

type TrendView = 'chart' | 'table'
const DIRECT_LABEL_LIMIT = 4
const GRIDLINES = [25, 50, 75] as const

const seriesStyle = (key: string) =>
  ({
    '--insights-series': key === INSIGHTS_OTHER_KEY ? 'var(--color-chart-other)' : sourceColor(key),
  }) as CSSProperties

export function InsightsAgentTrend({ data }: { data: HomeInsightsResponse }) {
  const t = useUsageT()
  const titleId = useId()
  const [view, setView] = useState<TrendView>('chart')
  const model = useMemo(() => buildTrend(data), [data])
  const nameOf = useCallback(
    (key: string) =>
      key === INSIGHTS_OTHER_KEY ? t('dashboard.insights.trend.other') : sourceLabel(key, t),
    [t]
  )
  const viewOptions = useMemo(
    () => [
      { value: 'chart' as const, label: t('dashboard.insights.views.chart') },
      { value: 'table' as const, label: t('dashboard.insights.views.table') },
    ],
    [t]
  )
  const handleView = useCallback((next: TrendView) => setView(next), [])

  return (
    <section className="insights-panel insights-trend" aria-labelledby={titleId}>
      <header className="insights-panel__header">
        <h3 id={titleId} className="insights-panel__title">{t('dashboard.insights.trend.title')}</h3>
        <InsightsSegmented
          options={viewOptions}
          value={view}
          onChange={handleView}
          ariaLabel={t('dashboard.insights.trend.viewLabel')}
        />
      </header>
      <ul className="insights-legend" aria-label={t('dashboard.insights.trend.legendAriaLabel')}>
        {model.series.map((series) => (
          <li key={series.key} className="insights-legend__item" style={seriesStyle(series.key)}>
            <span
              className="insights-legend__swatch"
              data-other={series.key === INSIGHTS_OTHER_KEY || undefined}
              aria-hidden="true"
            />
            {nameOf(series.key)}
          </li>
        ))}
      </ul>
      {view === 'chart' ? (
        <TrendChart model={model} nameOf={nameOf} />
      ) : (
        <TrendTable model={model} nameOf={nameOf} />
      )}
    </section>
  )
}

interface TrendPartProps {
  model: TrendModel
  nameOf: (key: string) => string
}

function TrendChart({ model, nameOf }: TrendPartProps) {
  const t = useUsageT()
  // 直接标签只看系列数（DESIGN.md：4 个及以下系列加直接标签），与末周是否有数据无关。
  const direct = model.series.length <= DIRECT_LABEL_LIMIT
  return (
    <div className="insights-trend__body">
      <div
        className="insights-trend__chart"
        role="img"
        aria-label={t('dashboard.insights.trend.ariaLabel', { count: model.series.length })}
      >
        {GRIDLINES.map((line) => (
          <span
            key={line}
            className="insights-gridline"
            style={{ bottom: `${line}%` } as CSSProperties}
            aria-hidden="true"
          />
        ))}
        {model.weeks.map((week) => (
          <div
            key={week.weekStart}
            className="insights-trend__bar"
            style={
              { height: `${model.max > 0 ? (week.total / model.max) * 100 : 0}%` } as CSSProperties
            }
            title={t('dashboard.insights.trend.weekTitle', {
              week: week.weekStart,
              count: fullNumber(week.total),
            })}
          >
            {week.values.map((value, index) =>
              value > 0 ? (
                <span
                  key={model.series[index].key}
                  className="insights-trend__segment"
                  data-other={model.series[index].key === INSIGHTS_OTHER_KEY || undefined}
                  style={{ ...seriesStyle(model.series[index].key), flexGrow: value }}
                  title={t('dashboard.insights.trend.segmentTitle', {
                    week: week.weekStart,
                    source: nameOf(model.series[index].key),
                    count: fullNumber(value),
                  })}
                />
              ) : null
            )}
          </div>
        ))}
      </div>
      {direct ? (
        <ul className="insights-trend__direct" aria-hidden="true">
          {model.series.map((series) => (
            <li key={series.key} style={seriesStyle(series.key)}>
              {nameOf(series.key)}
            </li>
          ))}
        </ul>
      ) : null}
    </div>
  )
}

function TrendTable({ model, nameOf }: TrendPartProps) {
  const t = useUsageT()
  const rows = useMemo(() => buildTrendTable(model), [model])
  return (
    <div className="insights-table-scroll insights-table-scroll--chart">
      <table className="insights-table" aria-label={t('dashboard.insights.trend.tableAriaLabel')}>
        <thead>
          <tr>
            <th scope="col">{t('dashboard.insights.weekStart')}</th>
            {model.series.map((series) => (
              <th
                key={series.key}
                scope="col"
                className="insights-table__source"
                style={seriesStyle(series.key)}
              >
                {nameOf(series.key)}
              </th>
            ))}
            <th scope="col">{t('dashboard.insights.weekTotal')}</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => (
            <tr key={row.weekStart}>
              <th scope="row" className="insights-num">
                {row.weekStart}
              </th>
              {row.values.map((value, index) => (
                <td key={model.series[index].key} className="insights-num">
                  {fullNumber(value)}
                </td>
              ))}
              <td className="insights-num">{fullNumber(row.total)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}
