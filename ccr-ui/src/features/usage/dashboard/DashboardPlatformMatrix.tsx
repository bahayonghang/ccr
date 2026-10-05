import { memo, type CSSProperties } from 'react'
import { Link } from 'react-router'
import type {
  DashboardMetricValue,
  DashboardPlatformRow,
  DashboardSessionIndexState,
} from '@/views/dashboard/dashboardPresentation'
import { useUsageT } from '../translate'
import '../styles/dashboard-platform-matrix.css'

interface DashboardPlatformMatrixProps {
  rows: DashboardPlatformRow[]
  installedCliCount: number
  runtimeCliCount: number
  sessionIndexState?: DashboardSessionIndexState
  className?: string
}

const resolveMetric = (metric: DashboardMetricValue | undefined, t: (key: string) => string) => {
  if (!metric) return '—'
  return metric.valueKey ? t(metric.valueKey) : metric.value ?? '—'
}

const cellMetric = (
  metric: DashboardMetricValue | undefined,
  fallbackLabelKey: string,
  t: (key: string) => string,
) => ({
  label: metric ? t(metric.labelKey) : t(fallbackLabelKey),
  text: resolveMetric(metric, t),
})

/** 会话格：未索引时显示 –（未知），索引正常才显示真实数字（含合法的 0）。 */
const sessionsCellMetric = (
  metric: DashboardMetricValue | undefined,
  sessionIndexState: DashboardSessionIndexState | undefined,
  t: (key: string) => string,
) => {
  const base = cellMetric(metric, 'dashboard.platforms.metrics.sessions', t)
  if (!sessionIndexState) return { ...base, state: undefined, hint: undefined }
  return {
    label: base.label,
    text: '–',
    state: sessionIndexState,
    hint: t('dashboard.usage.sessionsUnindexedHint'),
  }
}

const metricBySuffix = (platform: DashboardPlatformRow, suffix: string) =>
  platform.metrics.find((metric) => metric.labelKey.endsWith(suffix))

/** 用累计签名当 key，避免 sparkline 柱用数组下标。 */
const sparkBarItems = (platformKey: string, values: number[]) => {
  const items: Array<{ signature: string; value: number }> = []
  let signature = platformKey
  for (const value of values) {
    signature = `${signature}|${value}`
    items.push({ signature, value })
  }
  return items
}

const PlatformSparkline = memo(function PlatformSparkline({
  values,
  platformKey,
  label,
}: {
  values: number[]
  platformKey: string
  label: string
}) {
  const peak = Math.max(0, ...values)

  return (
    <span
      className="dashboard-platform__spark"
      role="img"
      aria-label={label}
      data-testid={`dashboard-platform-spark-${platformKey}`}
    >
      {sparkBarItems(platformKey, values).map((bar) => {
        const ratio = peak > 0 ? bar.value / peak : 0
        const isPeak = peak > 0 && bar.value === peak
        return (
          <span
            key={bar.signature}
            className={
              isPeak
                ? 'dashboard-platform__spark-bar is-peak'
                : 'dashboard-platform__spark-bar'
            }
            style={{ '--spark-ratio': String(ratio) } as CSSProperties}
          />
        )
      })}
    </span>
  )
})

const PlatformCard = memo(function PlatformCard({
  platform,
  sessionIndexState,
  t,
}: {
  platform: DashboardPlatformRow
  sessionIndexState: DashboardSessionIndexState | undefined
  t: (key: string) => string
}) {
  const version = platform.versionKey ? t(platform.versionKey) : platform.version ?? '…'
  const isMissing = platform.trackingHealth === 'missing'
  const requests = cellMetric(metricBySuffix(platform, '.requests'), 'dashboard.platforms.metrics.requests', t)
  const tokens = cellMetric(metricBySuffix(platform, '.tokens'), 'dashboard.platforms.metrics.tokens', t)
  const sessions = sessionsCellMetric(metricBySuffix(platform, '.sessions'), sessionIndexState, t)
  const statusLabel = isMissing ? t('dashboard.platforms.untracked') : t(platform.stateKey)

  return (
    <Link
      to={platform.path}
      className={`dashboard-platform dashboard-platform--${platform.platformKey}`}
      data-state={platform.state}
      data-tracking={platform.trackingHealth ?? 'unknown'}
      data-testid={`dashboard-platform-${platform.platformKey}`}
    >
      <span className="dashboard-platform__head">
        <span className="dashboard-platform__identity">
          <strong>{platform.title}</strong>
          {platform.versionKey === 'dashboard.platforms.stateScanning' ? (
            <span
              className="dashboard-platform__version-skeleton"
              role="status"
              aria-label={t(platform.versionKey)}
            />
          ) : (
            <span className="dashboard-platform__version">{version}</span>
          )}
        </span>
        <span
          className="dashboard-platform__status"
          data-state={isMissing ? 'attention' : platform.state}
          data-tracking={platform.trackingHealth ?? 'unknown'}
        >
          <span className="dashboard-platform__status-dot" aria-hidden="true" />
          {statusLabel}
        </span>
      </span>
      {isMissing ? (
        <span
          className="dashboard-platform__spark dashboard-platform__spark--empty"
          data-testid={`dashboard-platform-placeholder-${platform.platformKey}`}
        />
      ) : platform.sparkline ? (
        <PlatformSparkline
          values={platform.sparkline}
          platformKey={platform.platformKey}
          label={t('dashboard.usage.title')}
        />
      ) : (
        <span className="dashboard-platform__spark dashboard-platform__spark--idle" />
      )}
      <span className="dashboard-platform__rule" aria-hidden="true" />
      {isMissing ? (
        <span className="dashboard-platform__missing">
          <span>{t('dashboard.platforms.untrackedHint')}</span>
          <strong>{t('dashboard.platforms.configureAction')}</strong>
        </span>
      ) : (
        <span className="dashboard-platform__metrics">
          <span className="dashboard-platform__metric">
            <span>{requests.label}</span>
            <strong>{requests.text}</strong>
          </span>
          <span className="dashboard-platform__metric">
            <span>{tokens.label}</span>
            <strong>{tokens.text}</strong>
          </span>
          <span
            className="dashboard-platform__metric dashboard-platform__metric--sessions"
            data-session-state={sessions.state}
            title={sessions.hint}
          >
            <span>{sessions.label}</span>
            <strong>{sessions.text}</strong>
          </span>
        </span>
      )}
    </Link>
  )
})

export function DashboardPlatformMatrix({
  rows,
  installedCliCount,
  runtimeCliCount,
  sessionIndexState,
  className,
}: DashboardPlatformMatrixProps) {
  const t = useUsageT()

  return (
    <section
      className={['dashboard-platforms', className].filter(Boolean).join(' ')}
      data-dashboard-platforms
    >
      <h2 className="sr-only">{t('dashboard.platforms.title')}</h2>
      <p className="sr-only">
        {installedCliCount}/{runtimeCliCount} {t('dashboard.platforms.detectedLabel')}
      </p>
      <div className="dashboard-platforms__matrix">
        {rows.map((platform) => (
          <PlatformCard
            key={platform.platformKey}
            platform={platform}
            sessionIndexState={sessionIndexState}
            t={t}
          />
        ))}
      </div>
    </section>
  )
}
