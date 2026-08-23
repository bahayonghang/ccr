import { memo } from 'react'
import { Link } from 'react-router'
import { SIcon } from '@/ui'
import type {
  DashboardMetricValue,
  DashboardPlatformRow,
} from '@/views/dashboard/dashboardPresentation'
import { useUsageT } from '../translate'
import '../styles/dashboard-platform-matrix.css'

interface DashboardPlatformMatrixProps {
  rows: DashboardPlatformRow[]
  installedCliCount: number
  runtimeCliCount: number
  className?: string
}

const PlatformRow = memo(function PlatformRow({
  platform,
  t,
}: {
  platform: DashboardPlatformRow
  t: (key: string) => string
}) {
  const resolveMetric = (metric: DashboardMetricValue) =>
    metric.valueKey ? t(metric.valueKey) : metric.value ?? '…'
  const version = platform.versionKey ? t(platform.versionKey) : platform.version ?? '…'

  return (
    <Link
      to={platform.path}
      className={`dashboard-platform dashboard-platform--${platform.platformKey}`}
    >
      <span className="dashboard-platform__mark" aria-hidden="true" />
      <span className="dashboard-platform__icon">
        <SIcon name={platform.icon} size="w-4 h-4" />
      </span>
      <span className="dashboard-platform__identity">
        <strong>{platform.title}</strong>
        {platform.versionKey === 'dashboard.platforms.stateScanning' ? (
          <span
            className="dashboard-platform__version-skeleton"
            role="status"
            aria-label={t(platform.versionKey)}
          />
        ) : (
          <span>{version}</span>
        )}
      </span>
      <span className="dashboard-platform__status" data-state={platform.state}>
        <span className="dashboard-platform__status-dot" aria-hidden="true" />
        {t(platform.stateKey)}
      </span>
      <span className="dashboard-platform__role">{platform.role}</span>
      <span className="dashboard-platform__desc">{platform.desc}</span>
      {platform.metrics.map((metric) => (
        <span
          key={`${platform.platformKey}-${metric.labelKey}`}
          className="dashboard-platform__metric"
        >
          <span>{t(metric.labelKey)}</span>
          <strong>{resolveMetric(metric)}</strong>
        </span>
      ))}
      <span className="dashboard-platform__cta" aria-hidden="true">
        <SIcon name="ArrowRight" size="w-4 h-4" />
      </span>
    </Link>
  )
})

export function DashboardPlatformMatrix({
  rows,
  installedCliCount,
  runtimeCliCount,
  className,
}: DashboardPlatformMatrixProps) {
  const t = useUsageT()

  return (
    <section
      className={['dashboard-platforms', className].filter(Boolean).join(' ')}
      data-dashboard-platforms
    >
      <header className="dashboard-platforms__header">
        <div className="dashboard-platforms__lede">
          <p className="dashboard-platforms__eyebrow">{t('dashboard.platforms.eyebrow')}</p>
          <h2 className="dashboard-platforms__title">{t('dashboard.platforms.title')}</h2>
          <p className="dashboard-platforms__description">{t('dashboard.platforms.description')}</p>
        </div>
        <span className="dashboard-platforms__count">
          <strong>{installedCliCount}/{runtimeCliCount}</strong>
          <span>{t('dashboard.platforms.detectedLabel')}</span>
        </span>
      </header>
      <div className="dashboard-platforms__matrix">
        {rows.map((platform) => (
          <PlatformRow key={platform.platformKey} platform={platform} t={t} />
        ))}
      </div>
    </section>
  )
}
