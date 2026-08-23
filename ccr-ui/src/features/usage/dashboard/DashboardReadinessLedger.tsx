import { StatTile, SIcon } from '@/ui'
import type {
  DashboardReadiness,
  DashboardStatusMetric,
} from '@/views/dashboard/dashboardPresentation'
import { useUsageT } from '../translate'
import '../styles/dashboard-readiness-ledger.css'

interface DashboardReadinessLedgerProps {
  readiness: DashboardReadiness
  statusMetrics: DashboardStatusMetric[]
  className?: string
}

const stripTrailingPeriod = (text: string) => text.replace(/[。.]$/, '')

export function DashboardReadinessLedger({
  readiness,
  statusMetrics,
  className,
}: DashboardReadinessLedgerProps) {
  const t = useUsageT()

  return (
    <section
      className={['dashboard-ledger', className].filter(Boolean).join(' ')}
      data-dashboard-readiness
      data-status={readiness.status}
      aria-label={t('dashboard.readiness.label')}
    >
      <div className="dashboard-ledger__main">
        <p className="dashboard-ledger__eyebrow">
          <span className="dashboard-ledger__status-dot" aria-hidden="true" />
          {t(readiness.labelKey)}
        </p>
        <h2 className="dashboard-ledger__title">{t(readiness.titleKey)}</h2>
        <p className="dashboard-ledger__description">{t(readiness.descriptionKey)}</p>
        <ul className="dashboard-ledger__reasons">
          {readiness.reasons.map((reason) => (
            <li
              key={reason.key}
              className="dashboard-ledger__reason"
              data-ok={reason.ok}
            >
              <SIcon
                name={reason.ok ? 'Check' : 'AlertTriangle'}
                size="w-3.5 h-3.5"
                className="dashboard-ledger__reason-icon"
              />
              {stripTrailingPeriod(t(reason.key))}
            </li>
          ))}
        </ul>
      </div>
      <div className="dashboard-ledger__metrics">
        {statusMetrics.map((metric) => (
          <StatTile
            key={metric.id}
            label={t(metric.labelKey)}
            value={metric.valueKey ? t(metric.valueKey) : metric.value ?? '…'}
            hint={metric.hint ?? (metric.hintKey ? t(metric.hintKey) : '')}
            tone={metric.tone}
          />
        ))}
      </div>
    </section>
  )
}
