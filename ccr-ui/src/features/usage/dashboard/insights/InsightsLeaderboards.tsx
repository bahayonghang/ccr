import { useCallback, useId, useMemo, useState, type CSSProperties } from 'react'
import type { HomeInsightsResponse } from '@/types/generated/usage/HomeInsightsResponse'
import { useUsageT } from '../../translate'
import { compactLabel } from '../DashboardCostMetric'
import { InsightsSegmented } from './InsightsSegmented'
import {
  buildLeaderboard,
  defaultLeaderMetric,
  fullNumber,
  resolveLeaderMetric,
  type LeaderDimension,
  type LeaderMetric,
} from './insightsPresentation'
import { sourceLabel } from './insightsSources'

const DIMENSION_KEYS: Record<LeaderDimension, string> = {
  agents: 'dashboard.insights.leaderboard.agents',
  projects: 'dashboard.insights.leaderboard.projects',
  models: 'dashboard.insights.leaderboard.models',
}
const METRIC_KEYS: Record<LeaderMetric, string> = {
  requests: 'dashboard.insights.leaderboard.requests',
  tokens: 'dashboard.insights.leaderboard.tokens',
  sessions: 'dashboard.insights.leaderboard.sessions',
}

const sessionsDisabledTitle = (dimension: LeaderDimension, indexed: boolean) => {
  if (!indexed) return 'dashboard.insights.unindexedTitle'
  if (dimension !== 'agents') return 'dashboard.insights.leaderboard.sessionsAgentsOnly'
  return null
}

export function InsightsLeaderboards({ data }: { data: HomeInsightsResponse }) {
  const t = useUsageT()
  const titleId = useId()
  const [dimension, setDimension] = useState<LeaderDimension>('agents')
  // 用户未选择度量时跟随数据推导默认度量
  const [chosenMetric, setMetric] = useState<LeaderMetric | null>(null)
  const metric = chosenMetric ?? defaultLeaderMetric(data)
  const handleDimension = useCallback((next: LeaderDimension) => setDimension(next), [])
  const handleMetric = useCallback((next: LeaderMetric) => setMetric(next), [])
  const resolveLabel = useCallback((source: string) => sourceLabel(source, t), [t])
  const effective = resolveLeaderMetric(dimension, metric, data.sessions_indexed)
  const rows = useMemo(
    () => buildLeaderboard(data, { dimension, metric }, resolveLabel),
    [data, dimension, metric, resolveLabel]
  )
  const disabledKey = sessionsDisabledTitle(dimension, data.sessions_indexed)
  const dimensionOptions = useMemo(
    () =>
      (Object.keys(DIMENSION_KEYS) as LeaderDimension[]).map((value) => ({
        value,
        label: t(DIMENSION_KEYS[value]),
      })),
    [t]
  )
  const metricOptions = useMemo(
    () =>
      (Object.keys(METRIC_KEYS) as LeaderMetric[]).map((value) => ({
        value,
        label: t(METRIC_KEYS[value]),
        disabled: value === 'sessions' && disabledKey !== null,
        title: value === 'sessions' && disabledKey ? t(disabledKey) : undefined,
      })),
    [t, disabledKey]
  )

  return (
    <section className="insights-panel insights-leaderboard" aria-labelledby={titleId}>
      <header className="insights-panel__header insights-panel__header--wrap">
        <h3 id={titleId} className="insights-panel__title">{t('dashboard.insights.leaderboard.title')}</h3>
        <div className="insights-leaderboard__controls">
          <InsightsSegmented
            options={dimensionOptions}
            value={dimension}
            onChange={handleDimension}
            ariaLabel={t('dashboard.insights.leaderboard.dimensionLabel')}
          />
          <InsightsSegmented
            options={metricOptions}
            value={effective}
            onChange={handleMetric}
            ariaLabel={t('dashboard.insights.leaderboard.metricLabel')}
          />
        </div>
      </header>
      {rows.length === 0 ? (
        <p className="insights-caption">{t('dashboard.insights.leaderboard.empty')}</p>
      ) : (
        <ol className="insights-leaderboard__list" aria-label={t(DIMENSION_KEYS[dimension])}>
          {rows.map((row) => (
            <li
              key={row.key}
              className="insights-leaderboard__row"
              data-agent={row.color !== null || undefined}
              style={row.color ? ({ '--insights-series': row.color } as CSSProperties) : undefined}
            >
              <span className="insights-leaderboard__name" title={row.label}>
                {row.label}
              </span>
              <span className="insights-leaderboard__track" aria-hidden="true">
                <span
                  className="insights-leaderboard__bar"
                  style={{ width: `${row.ratio * 100}%` } as CSSProperties}
                />
              </span>
              <span
                className="insights-num insights-leaderboard__value"
                title={fullNumber(row.value)}
              >
                {compactLabel(null, row.value)}
              </span>
            </li>
          ))}
        </ol>
      )}
    </section>
  )
}
