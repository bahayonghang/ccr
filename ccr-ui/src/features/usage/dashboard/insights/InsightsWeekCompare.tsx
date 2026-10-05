import { useId } from 'react'
import type { HomeInsightsResponse } from '@/types/generated/usage/HomeInsightsResponse'
import { useUsageT } from '../../translate'
import {
  buildWeekCompare,
  EM_DASH,
  formatChange,
  fullNumber,
  type CompareKey,
} from './insightsPresentation'

const ROW_KEYS: Record<CompareKey, string> = {
  sessions: 'dashboard.insights.compare.sessions',
  requests: 'dashboard.insights.compare.requests',
  activeDays: 'dashboard.insights.compare.activeDays',
}

export function InsightsWeekCompare({ data }: { data: HomeInsightsResponse }) {
  const t = useUsageT()
  const titleId = useId()
  const rows = buildWeekCompare(data)
  const unindexedTitle = t('dashboard.insights.unindexedTitle')
  const cellTitle = (key: CompareKey, value: number | null) =>
    value === null && key === 'sessions' && !data.sessions_indexed ? unindexedTitle : undefined

  return (
    <section className="insights-panel insights-compare" aria-labelledby={titleId}>
      <h3 id={titleId} className="insights-panel__title">{t('dashboard.insights.compare.title')}</h3>
      <table className="insights-compare__table">
        <thead>
          <tr>
            <th scope="col">{t('dashboard.insights.compare.metric')}</th>
            <th scope="col">{t('dashboard.insights.compare.last')}</th>
            <th scope="col">{t('dashboard.insights.compare.previous')}</th>
            <th scope="col">{t('dashboard.insights.compare.change')}</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => (
            <tr key={row.key} data-row={row.key}>
              <th scope="row">{t(ROW_KEYS[row.key])}</th>
              <td className="insights-num" title={cellTitle(row.key, row.last)}>
                {row.last === null ? EM_DASH : fullNumber(row.last)}
              </td>
              <td className="insights-num" title={cellTitle(row.key, row.previous)}>
                {row.previous === null ? EM_DASH : fullNumber(row.previous)}
              </td>
              <td className="insights-num insights-compare__change">
                {formatChange(row.change, t)}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  )
}
