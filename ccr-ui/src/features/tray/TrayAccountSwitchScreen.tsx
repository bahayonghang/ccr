import { memo, useCallback } from 'react'
import type { CodexTrayAccountRow, CodexTraySnapshot } from '@/types'
import { SIcon } from '@/ui'
import { useTrayT } from './locale'

interface TrayAccountSwitchScreenProps {
  snapshot: CodexTraySnapshot
  currentAccount: CodexTrayAccountRow | null
  accounts: CodexTrayAccountRow[]
  busyAccount: string | null
  canManageAccounts: boolean
  onBack: () => void
  onSwitch: (name: string) => void
  onOpenAuth: () => void
}

const TrayAccountRow = memo(function TrayAccountRow({
  account,
  busyAccount,
  canManageAccounts,
  onSwitch,
  t,
}: {
  account: CodexTrayAccountRow
  busyAccount: string | null
  canManageAccounts: boolean
  onSwitch: (name: string) => void
  t: (key: string) => string
}) {
  const handleSwitch = useCallback(() => {
    onSwitch(account.name)
  }, [account.name, onSwitch])
  const status = account.is_current
    ? t('codex.auth.currentBadge')
    : !canManageAccounts
      ? t('codex.auth.tray.unavailableInCurrentProfile')
      : !account.can_switch
        ? t('settings.disabled')
        : t('codex.auth.tray.available')
  const statusClass = account.is_current
    ? 'tray-switch__status--current'
    : !account.can_switch || !canManageAccounts
      ? 'tray-switch__status--muted'
      : 'tray-switch__status--available'
  const busy = busyAccount === account.name

  return (
    <article
      className={`tray-switch__row${account.is_current ? ' tray-switch__row--current' : ''}`}
      data-testid={`tray-switch-row-${account.name}`}
    >
      <div className="tray-switch__row-main min-w-0">
        <div className="tray-switch__row-title-line">
          <p className="tray-switch__row-title">{account.email || account.name}</p>
          <span className={`tray-switch__status ${statusClass}`}>{status}</span>
        </div>
        <p className="tray-switch__row-subtitle">
          {account.name}
          {account.last_refresh ? t('codex.auth.tray.lastRefreshInline') : null}
        </p>
      </div>
      {account.can_switch ? (
        <button type="button" className="tray-switch__action" disabled={busy} onClick={handleSwitch}>
          <SIcon name={busy ? 'RefreshCw' : 'ArrowLeftRight'} size="w-4 h-4" className={busy ? 'animate-spin' : ''} />
          <span>{t('codex.auth.switch')}</span>
        </button>
      ) : (
        <span className="tray-switch__row-placeholder" />
      )}
    </article>
  )
})

export const TrayAccountSwitchScreen = memo(function TrayAccountSwitchScreen({
  snapshot,
  currentAccount,
  accounts,
  busyAccount,
  canManageAccounts,
  onBack,
  onSwitch,
  onOpenAuth,
}: TrayAccountSwitchScreenProps) {
  const t = useTrayT()
  const handleBack = useCallback(() => {
    onBack()
  }, [onBack])
  const handleAuth = useCallback(() => {
    onOpenAuth()
  }, [onOpenAuth])

  return (
    <section className="tray-switch" data-testid="tray-switch-screen">
      <header className="tray-switch__header">
        <button type="button" className="tray-switch__back" data-testid="tray-switch-back" onClick={handleBack}>
          <SIcon name="ArrowLeft" size="w-4 h-4" />
          <span>{t('common.back')}</span>
        </button>
        <div>
          <p className="tray-switch__eyebrow">{t('codex.auth.tray.currentSession')}</p>
          <h2 className="tray-switch__title">{t('codex.auth.tray.switchAccount')}</h2>
        </div>
      </header>

      <article className="tray-switch__current">
        <div className="tray-switch__current-main min-w-0">
          <p className="tray-switch__current-title">{currentAccount?.email || currentAccount?.name || snapshot.auth_label}</p>
          <p className="tray-switch__current-subtitle">{snapshot.current_profile_name || snapshot.profile_label}</p>
        </div>
        <div className="tray-switch__current-meta">
          <span className="tray-switch__badge">{snapshot.runtime_description}</span>
          <span className="tray-switch__badge tray-switch__badge--soft">{snapshot.auth_label}</span>
        </div>
      </article>

      {accounts.length === 0 ? (
        <div className="tray-switch__empty">
          <SIcon name="Users" size="w-4 h-4" />
          <div>
            <p>{t('codex.auth.tray.noAccountsTitle')}</p>
            <p>{t('codex.auth.tray.noAccountsHint')}</p>
          </div>
        </div>
      ) : (
        <section className="tray-switch__list" data-testid="tray-switch-list">
          {accounts.map((account) => (
            <TrayAccountRow
              key={account.name}
              account={account}
              busyAccount={busyAccount}
              canManageAccounts={canManageAccounts}
              onSwitch={onSwitch}
              t={t}
            />
          ))}
        </section>
      )}

      <footer className="tray-switch__footer">
        <button type="button" className="tray-switch__footer-action" onClick={handleAuth}>
          <SIcon name="Users" size="w-4 h-4" />
          <span>{t('codex.auth.tray.openAuth')}</span>
        </button>
      </footer>
    </section>
  )
})
