import { memo, useCallback } from 'react'
import type { CodexTrayAccountRow, CodexTraySnapshot } from '@/types'
import { SIcon, StatTile } from '@/ui'
import { useTrayT } from './locale'
import { formatReset, quotaScale, quotaToneClass } from './tray-format'

function headlineCopy(
  snapshot: CodexTraySnapshot,
  currentAccount: CodexTrayAccountRow | null,
) {
  return {
    accountHeadline: currentAccount?.email || currentAccount?.name || snapshot.auth_label,
    profileLine: snapshot.current_profile_name || snapshot.profile_label,
    support: currentAccount?.last_refresh ? `最近刷新 ${currentAccount.last_refresh}` : snapshot.auth_label,
  }
}

function quotaHint(t: ReturnType<typeof useTrayT>, timestamp?: number, detailed = false) {
  if (!timestamp) return undefined
  return `${t('codex.auth.tray.resetIn')} ${formatReset(t, timestamp, detailed)}`
}

function QuotaBlock({
  quota,
  quotaError,
  t,
}: {
  quota: CodexTrayAccountRow['quota']
  quotaError?: string | null
  t: ReturnType<typeof useTrayT>
}) {
  if (!quota) {
    return (
      <div className="tray-overview__quota-status">
        <SIcon name={quotaError ? 'AlertCircle' : 'Clock3'} size="w-4 h-4" />
        <p>{quotaError || t('codex.auth.quotaNotQueried')}</p>
      </div>
    )
  }
  return (
    <section className="tray-overview__quota-grid" data-testid="tray-overview-quotas">
      <article className={`tray-overview__quota-card ${quotaToneClass(quota.hourly_percentage)}`}>
        <StatTile label={t('codex.auth.hourlyQuota')} value={`${quota.hourly_percentage}%`} hint={quotaHint(t, quota.hourly_reset_time)} />
        <div className="tray-overview__progress">
          <span className="tray-overview__progress-fill" style={{ transform: `scaleX(${quotaScale(quota.hourly_percentage)})` }} />
        </div>
      </article>
      <article className={`tray-overview__quota-card ${quotaToneClass(quota.weekly_percentage)}`}>
        <StatTile label={t('codex.auth.weeklyQuota')} value={`${quota.weekly_percentage}%`} hint={quotaHint(t, quota.weekly_reset_time, true)} />
        <div className="tray-overview__progress">
          <span className="tray-overview__progress-fill" style={{ transform: `scaleX(${quotaScale(quota.weekly_percentage)})` }} />
        </div>
      </article>
    </section>
  )
}

interface TrayOverviewProps {
  snapshot: CodexTraySnapshot
  currentAccount: CodexTrayAccountRow | null
  canManageAccounts: boolean
  onOpenMain: () => void
  onOpenSwitch: () => void
  onOpenUsage: () => void
  onOpenAuth: () => void
  onQuit: () => void
}

export const TrayOverview = memo(function TrayOverview({
  snapshot,
  currentAccount,
  canManageAccounts,
  onOpenMain,
  onOpenSwitch,
  onOpenUsage,
  onOpenAuth,
  onQuit,
}: TrayOverviewProps) {
  const t = useTrayT()
  const copy = headlineCopy(snapshot, currentAccount)
  const quota = currentAccount?.quota

  const handleSwitch = useCallback(() => {
    onOpenSwitch()
  }, [onOpenSwitch])
  const handleUsage = useCallback(() => {
    onOpenUsage()
  }, [onOpenUsage])
  const handleMain = useCallback(() => {
    onOpenMain()
  }, [onOpenMain])
  const handleAuth = useCallback(() => {
    onOpenAuth()
  }, [onOpenAuth])
  const handleQuit = useCallback(() => {
    onQuit()
  }, [onQuit])

  return (
    <section className="tray-overview" data-testid="tray-overview">
      <article className="tray-overview__hero">
        <div className="tray-overview__hero-main">
          <div className="tray-overview__hero-lead">
            <div className="tray-overview__hero-icon">
              <SIcon name="KeyRound" size="w-5 h-5" />
            </div>
            <div className="min-w-0">
              <p className="tray-overview__eyebrow">{copy.profileLine}</p>
              <div className="tray-overview__title-row">
                <p className="tray-overview__headline">{copy.accountHeadline}</p>
                {quota?.plan_type ? <span className="tray-overview__plan-badge">{quota.plan_type}</span> : null}
              </div>
              <p className="tray-overview__support">{copy.support}</p>
            </div>
          </div>
          <div className="tray-overview__route-grid">
            <div className="tray-overview__route-item">
              <span className="tray-overview__route-label">{t('codex.auth.tray.runtimeLabel')}</span>
              <strong className="tray-overview__route-value">{snapshot.runtime_description}</strong>
            </div>
            <div className="tray-overview__route-item">
              <span className="tray-overview__route-label">{t('codex.auth.tray.authRouteLabel')}</span>
              <strong className="tray-overview__route-value">{snapshot.auth_label}</strong>
            </div>
          </div>
        </div>
      </article>

      <QuotaBlock quota={quota} quotaError={currentAccount?.quota_error} t={t} />

      <section className="tray-overview__actions">
        <button type="button" className="tray-overview__action tray-overview__action--primary" data-testid="tray-action-switch" disabled={!canManageAccounts} onClick={handleSwitch}>
          <SIcon name="ArrowLeftRight" size="w-4 h-4" />
          <span>{t('codex.auth.tray.switchAccount')}</span>
        </button>
        <button type="button" className="tray-overview__action" data-testid="tray-action-open-usage" onClick={handleUsage}>
          <SIcon name="BarChart3" size="w-4 h-4" />
          <span>{t('codex.auth.tray.openUsage')}</span>
        </button>
        <button type="button" className="tray-overview__action" data-testid="tray-action-open-main" onClick={handleMain}>
          <SIcon name="PanelLeftOpen" size="w-4 h-4" />
          <span>{t('codex.auth.tray.openMain')}</span>
        </button>
      </section>

      {!canManageAccounts ? (
        <div className="tray-overview__hint">
          <span>{t('codex.auth.tray.switchUnavailable')}</span>
          <button type="button" className="tray-overview__link" data-testid="tray-action-open-auth" onClick={handleAuth}>
            {t('codex.auth.tray.openAuth')}
          </button>
        </div>
      ) : null}

      <footer className="tray-overview__footer">
        <span className="tray-overview__footer-note">
          <span className="tray-overview__footer-dot" />
          {currentAccount?.last_refresh ? `最近刷新 ${currentAccount.last_refresh}` : snapshot.auth_label}
        </span>
        <button type="button" className="tray-overview__secondary" onClick={handleQuit}>
          <SIcon name="Power" size="w-4 h-4" />
          <span>{t('codex.auth.tray.quit')}</span>
        </button>
      </footer>
    </section>
  )
})
