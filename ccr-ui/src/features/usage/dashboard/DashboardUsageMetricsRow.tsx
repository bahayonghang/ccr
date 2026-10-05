import { Link } from 'react-router'
import type { DashboardSessionIndexState } from '@/views/dashboard/dashboardPresentation'
import { DashboardCostMetric, type UsageStackPlatform } from './DashboardCostMetric'

export function UsageMetricCell(props: { label: string; value: string; hero?: boolean; loading: boolean }) {
  const valueClass = props.hero
    ? 'dashboard-usage__metric-value dashboard-usage__metric-value--hero'
    : 'dashboard-usage__metric-value'
  return (
    <div className={props.hero ? 'dashboard-usage__metric dashboard-usage__metric--hero' : 'dashboard-usage__metric'}>
      <span className="dashboard-usage__metric-label">{props.label}</span>
      {props.loading ? <span className="dashboard-usage__metric-skeleton" /> : (
        <span className={valueClass} data-hero={props.hero ? 'true' : undefined} data-zero={props.value === '0' ? 'true' : 'false'}>
          {props.value}
        </span>
      )}
    </div>
  )
}

/**
 * 会话格：session_archive 未索引时 total_sessions 是后端给的占位 0，
 * 渲染"未索引/索引中"诚实态（链接到用量页重建索引），而不是静默的 0。
 */
export function UsageSessionMetricCell(props: {
  label: string
  value: string
  loading: boolean
  sessionIndexState: DashboardSessionIndexState
  stateLabel: string
  stateHint: string
}) {
  return (
    <div className="dashboard-usage__metric">
      <span className="dashboard-usage__metric-label">{props.label}</span>
      {props.loading ? <span className="dashboard-usage__metric-skeleton" /> : props.sessionIndexState ? (
        <Link
          to="/usage"
          className="dashboard-usage__metric-value dashboard-usage__metric-value--notice"
          data-session-state={props.sessionIndexState}
          title={props.stateHint}
        >
          {props.stateLabel}
        </Link>
      ) : (
        <span className="dashboard-usage__metric-value" data-zero={props.value === '0' ? 'true' : 'false'}>
          {props.value}
        </span>
      )}
    </div>
  )
}

export function UsageCostCell(props: { label: string; days: number; ready: boolean }) {
  return (
    <div className="dashboard-usage__metric">
      <span className="dashboard-usage__metric-label">{props.label}</span>
      <span className="dashboard-usage__metric-value">
        {props.ready ? <DashboardCostMetric days={props.days} /> : <span data-dashboard-cost-placeholder>—</span>}
      </span>
    </div>
  )
}

export function UsageMetricsRow(props: {
  loading: boolean
  showLegend: boolean
  requests: string
  tokens: string
  sessions: string
  costLabel: string
  requestLabel: string
  tokenLabel: string
  sessionLabel: string
  sessionStateLabel: string
  sessionStateHint: string
  sessionIndexState: DashboardSessionIndexState
  days: number
  costReady: boolean
  legend: UsageStackPlatform[]
  labels: Record<UsageStackPlatform, string>
}) {
  return (
    <div className="dashboard-usage__metrics">
      <UsageMetricCell label={props.requestLabel} value={props.requests} hero loading={props.loading} />
      <UsageMetricCell label={props.tokenLabel} value={props.tokens} loading={props.loading} />
      <UsageCostCell label={props.costLabel} days={props.days} ready={props.costReady} />
      <UsageSessionMetricCell
        label={props.sessionLabel}
        value={props.sessions}
        loading={props.loading}
        sessionIndexState={props.sessionIndexState}
        stateLabel={props.sessionStateLabel}
        stateHint={props.sessionStateHint}
      />
      {props.showLegend ? (
        <div className="dashboard-usage__legend">
          {props.legend.map((platform) => (
            <span key={platform} className="dashboard-usage__legend-item">
              <span className="dashboard-usage__legend-swatch" data-platform={platform} />
              {props.labels[platform]}
            </span>
          ))}
        </div>
      ) : null}
    </div>
  )
}
