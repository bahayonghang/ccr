import { memo } from 'react'
import { Link } from 'react-router'
import { SIcon } from '@/ui'
import type { IconName } from '@/config/icons'
import type { DashboardAction } from '@/views/dashboard/dashboardPresentation'
import { useUsageT } from '../translate'
import '../styles/dashboard-next-actions.css'

interface DashboardNextActionsProps {
  actions: DashboardAction[]
  showOnboarding?: boolean
  className?: string
}

const ONBOARDING_STEPS: Array<{
  id: string
  path: string
  icon: IconName
  titleKey: string
  descKey: string
}> = [
  {
    id: 'create-profile',
    path: '/claude-code',
    icon: 'UserCheck',
    titleKey: 'dashboard.actions.onboardingStep1Title',
    descKey: 'dashboard.actions.onboardingStep1Desc',
  },
  {
    id: 'configure-mcp',
    path: '/mcp-manager',
    icon: 'Plug',
    titleKey: 'dashboard.actions.onboardingStep2Title',
    descKey: 'dashboard.actions.onboardingStep2Desc',
  },
  {
    id: 'import-usage',
    path: '/usage',
    icon: 'Download',
    titleKey: 'dashboard.actions.onboardingStep3Title',
    descKey: 'dashboard.actions.onboardingStep3Desc',
  },
]

const ActionRow = memo(function ActionRow({
  action,
  primary,
  t,
}: {
  action: DashboardAction
  primary: boolean
  t: (key: string) => string
}) {
  return (
    <Link
      to={action.path}
      className={[
        'dashboard-action',
        `dashboard-action--${action.tone}`,
        primary ? 'dashboard-action--primary' : '',
      ].join(' ')}
    >
      <span className="dashboard-action__icon">
        <SIcon name={action.icon} size="w-4 h-4" />
      </span>
      <span className="dashboard-action__copy">
        <strong>{t(action.titleKey)}</strong>
        <span>{t(action.descKey)}</span>
        {action.detail ? <em>{action.detail}</em> : null}
      </span>
      <span className="dashboard-action__cta" aria-hidden="true">
        <SIcon name="ArrowRight" size="w-4 h-4" />
      </span>
    </Link>
  )
})

export function DashboardNextActions({
  actions,
  showOnboarding = false,
  className,
}: DashboardNextActionsProps) {
  const t = useUsageT()

  return (
    <section
      className={['dashboard-actions', className].filter(Boolean).join(' ')}
      data-dashboard-actions
    >
      <header className="dashboard-actions__header">
        <p className="dashboard-actions__eyebrow">{t('dashboard.actions.eyebrow')}</p>
        <h2 className="dashboard-actions__title">{t('dashboard.actions.title')}</h2>
        <p className="dashboard-actions__description">
          {showOnboarding
            ? t('dashboard.actions.onboardingDescription')
            : t('dashboard.actions.description')}
        </p>
      </header>
      {showOnboarding ? (
        <ol className="dashboard-actions__onboarding">
          {ONBOARDING_STEPS.map((step, index) => (
            <li key={step.id}>
              <Link
                to={step.path}
                className={[
                  'dashboard-onboarding-step',
                  index === 0 ? 'dashboard-onboarding-step--primary' : '',
                ].join(' ')}
              >
                <span className="dashboard-onboarding-step__index">{index + 1}</span>
                <span className="dashboard-onboarding-step__icon">
                  <SIcon name={step.icon} size="w-4 h-4" />
                </span>
                <span className="dashboard-onboarding-step__copy">
                  <strong>{t(step.titleKey)}</strong>
                  <span>{t(step.descKey)}</span>
                </span>
                <span className="dashboard-action__cta" aria-hidden="true">
                  <SIcon name="ArrowRight" size="w-4 h-4" />
                </span>
              </Link>
            </li>
          ))}
        </ol>
      ) : (
        <div className="dashboard-actions__queue">
          {actions.map((action, index) => (
            <ActionRow
              key={action.id}
              action={action}
              primary={index === 0}
              t={t}
            />
          ))}
        </div>
      )}
    </section>
  )
}
