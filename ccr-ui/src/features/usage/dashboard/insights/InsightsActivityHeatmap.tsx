import { useCallback, useEffect, useId, useMemo, useRef, useState, type CSSProperties } from 'react'
import type { HomeInsightsResponse } from '@/types/generated/usage/HomeInsightsResponse'
import { useUsageT } from '../../translate'
import { InsightsSegmented } from './InsightsSegmented'
import {
  buildHeatmap,
  buildHeatTable,
  buildStreakCaption,
  fullNumber,
  MONTH_KEYS,
  WEEKDAY_KEYS,
  type HeatmapModel,
} from './insightsPresentation'

type HeatView = 'chart' | 'table'
const LEVELS = [0, 1, 2, 3, 4] as const

interface InsightsActivityHeatmapProps {
  data: HomeInsightsResponse
  onImport: () => void
  importing: boolean
}

export function InsightsActivityHeatmap({
  data,
  onImport,
  importing,
}: InsightsActivityHeatmapProps) {
  const t = useUsageT()
  const titleId = useId()
  const [view, setView] = useState<HeatView>('chart')
  const scrollRef = useRef<HTMLDivElement>(null)
  const model = useMemo(() => buildHeatmap(data), [data])
  const caption = buildStreakCaption(data)
  const hasRequests = data.totals.requests > 0
  const viewOptions = useMemo(
    () => [
      { value: 'chart' as const, label: t('dashboard.insights.views.chart') },
      { value: 'table' as const, label: t('dashboard.insights.views.table') },
    ],
    [t]
  )
  const handleView = useCallback((next: HeatView) => setView(next), [])

  // 格宽不足时容器横向滚动，打开时定位到最新一周
  useEffect(() => {
    const node = scrollRef.current
    if (node) node.scrollLeft = node.scrollWidth
  }, [view, model])

  return (
    <section className="insights-panel insights-heatmap" aria-labelledby={titleId}>
      <header className="insights-panel__header">
        <h3 id={titleId} className="insights-panel__title">{t('dashboard.insights.heatmap.title')}</h3>
        <InsightsSegmented
          options={viewOptions}
          value={view}
          onChange={handleView}
          ariaLabel={t('dashboard.insights.heatmap.viewLabel')}
        />
      </header>
      {view === 'chart' ? (
        <div ref={scrollRef} className="insights-heatmap__scroll">
          <div
            className="insights-heatmap__grid"
            role="img"
            aria-label={t('dashboard.insights.heatmap.ariaLabel', { days: model.activeDays })}
          >
            {model.months.map((label) => (
              <span
                key={`m-${label.column}`}
                className="insights-heatmap__month"
                style={{ gridColumn: label.column + 1 } as CSSProperties}
                aria-hidden="true"
              >
                {t(MONTH_KEYS[label.month - 1])}
              </span>
            ))}
            {model.weeks.flatMap((week, column) =>
              week.cells.map((cell, row) => (
                <span
                  key={cell.date}
                  className="insights-heatmap__cell"
                  data-level={cell.level}
                  data-future={cell.future || undefined}
                  style={{ gridColumn: column + 1, gridRow: row + 2 } as CSSProperties}
                  title={
                    cell.future
                      ? undefined
                      : t('dashboard.insights.heatmap.cellTitle', {
                          date: cell.date,
                          count: fullNumber(cell.requests),
                        })
                  }
                />
              ))
            )}
          </div>
        </div>
      ) : (
        <HeatTable model={model} />
      )}
      <footer className="insights-heatmap__footer">
        {hasRequests ? (
          <p className="insights-caption">
            {caption.map((part) => t(part.key, part.values)).join(' · ')}
          </p>
        ) : (
          <p className="insights-caption">
            {t('dashboard.insights.heatmap.emptyCaption')}
            <button
              type="button"
              className="insights-import"
              onClick={onImport}
              disabled={importing}
            >
              {importing
                ? t('dashboard.insights.empty.importing')
                : t('dashboard.insights.empty.import')}
            </button>
          </p>
        )}
        <span className="insights-heatmap__legend" aria-hidden="true">
          {t('dashboard.insights.heatmap.less')}
          {LEVELS.map((level) => (
            <span key={level} className="insights-heatmap__cell" data-level={level} />
          ))}
          {t('dashboard.insights.heatmap.more')}
        </span>
      </footer>
    </section>
  )
}

function HeatTable({ model }: { model: HeatmapModel }) {
  const t = useUsageT()
  const rows = useMemo(() => buildHeatTable(model), [model])
  return (
    <div className="insights-table-scroll">
      <table className="insights-table" aria-label={t('dashboard.insights.heatmap.tableAriaLabel')}>
        <thead>
          <tr>
            <th scope="col">{t('dashboard.insights.weekStart')}</th>
            {WEEKDAY_KEYS.map((key) => (
              <th key={key} scope="col">
                {t(key)}
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
              {row.days.map((value, index) => (
                <td key={WEEKDAY_KEYS[index]} className="insights-num">
                  {value === null ? '' : fullNumber(value)}
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
