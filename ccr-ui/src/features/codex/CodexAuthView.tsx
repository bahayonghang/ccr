import { useCallback } from 'react'
import { Link } from 'react-router'
import { ConfirmModal, PageHeader, PageShell, SIcon } from '@/ui'
import { CodexSubnav } from './CodexSubnav'
import { AuthOffBanners } from './auth/AuthOffBanners'
import { AuthWorkspace } from './auth/AuthWorkspace'
import { AddCodexAccountModal } from './auth/AddCodexAccountModal'
import { RenameCodexAccountModal } from './auth/RenameCodexAccountModal'
import { SaveCodexSessionModal } from './auth/SaveCodexSessionModal'
import { useCodexAuthPage, type ManagerTab } from './auth/useCodexAuthPage'
import { loginStateText } from './auth/loginStateText'
import { getLoginStateIcon, getLoginStateIconClass } from './codexAuthAccounts'
import { panelCardClass, primaryBtnClass, secondaryBtnClass } from './ui-classes'
import { useCodexLocale } from './useCodexLocale'

export function CodexAuthView() {
  const { t, tf } = useCodexLocale()
  const page = useCodexAuthPage(t, tf)
  const statusLabel = loginStateText(page.loginState, t, tf)

  return (
    <PageShell
      className="codex-auth-view"
      header={
        <PageHeader
          title={t('codex.auth.title')}
          description={tf('codex.auth.managerSubtitle', 'Use one surface to add, import, switch, and review Codex accounts and model providers.')}
          leading={
            <div className="codex-auth-view__title-icon-shell">
              <SIcon name="KeyRound" size="w-6 h-6" className="codex-auth-view__title-icon" />
            </div>
          }
          actions={
            <div className="flex flex-wrap gap-2">
              <Link to="/codex" className={secondaryBtnClass}><SIcon name="ArrowLeft" size="w-4 h-4" />{t('codex.auth.backToCodex')}</Link>
              <button type="button" className={secondaryBtnClass} disabled={page.loading} onClick={page.loadAll}>
                <SIcon name="RefreshCw" size="w-4 h-4" className={page.loading ? 'animate-spin' : undefined} />{t('codex.auth.refresh')}
              </button>
              <button type="button" className={secondaryBtnClass} disabled={!page.canSave} onClick={page.handleSave}>
                <SIcon name="Save" size="w-4 h-4" />{tf('codex.auth.actions.saveCurrent', 'Save current session')}
              </button>
              <button type="button" className={primaryBtnClass} onClick={page.openAddAccount}>
                <SIcon name="Plus" size="w-4 h-4" />{tf('codex.auth.actions.addAccount', 'Add account')}
              </button>
            </div>
          }
        />
      }
      subnav={<CodexSubnav />}
    >
      <main className="codex-auth-view__main">
        <AuthOffBanners
          t={t}
          canAuthOff={page.canAuthOff}
          canOff={page.canOff}
          loading={page.loading}
          onAuthOff={page.handleAuthOff}
          onProfileOff={page.handleOff}
        />
        <div className="codex-auth-view__status-grid mb-4 grid gap-3 md:grid-cols-2 xl:grid-cols-4">
          <div className={panelCardClass}>
            <div className="codex-auth-view__status-row">
              <div className={`codex-auth-view__status-icon-shell ${getLoginStateIconClass(page.loginState)}`}>
                <SIcon name={getLoginStateIcon(page.loginState)} size="w-6 h-6" />
              </div>
              <div>
                <p className="codex-auth-view__status-label">{t('codex.auth.status.loginState')}</p>
                <p className="codex-auth-view__status-value">{statusLabel}</p>
              </div>
            </div>
          </div>
          <div className={panelCardClass}>
            <p className="codex-auth-view__status-label">{t('codex.auth.status.totalAccounts')}</p>
            <p className="codex-auth-view__status-value">{page.accounts.length}</p>
          </div>
          <div className={panelCardClass}>
            <p className="codex-auth-view__status-label">{t('codex.auth.status.currentAccount')}</p>
            <p className="codex-auth-view__status-value">{page.currentAccount?.email || page.currentAccount?.name || t('codex.auth.status.noAccount')}</p>
          </div>
          <div className={panelCardClass}>
            <p className="codex-auth-view__status-label">{tf('codex.auth.status.providerCount', 'Model providers')}</p>
            <p className="codex-auth-view__status-value">{page.providersApi.providers.length}</p>
          </div>
        </div>
        <div className="codex-auth-view__segment-row mb-4">
          <TabChip id="accounts" label={t('codex.auth.accountOverview')} count={page.accounts.length} active={page.activeManagerTab === 'accounts'} icon="LayoutGrid" onSelect={page.setActiveManagerTab} />
          <TabChip id="providers" label={tf('codex.auth.providers.title', 'Model providers')} count={page.providersApi.providers.length} active={page.activeManagerTab === 'providers'} icon="Blocks" onSelect={page.setActiveManagerTab} />
        </div>
        <AuthWorkspace page={page} t={t} tf={tf} />
      </main>
      <SaveCodexSessionModal modelValue={page.showSaveForm} currentInfo={page.currentInfo} formatAuthMethod={page.formatAuthMethod} onUpdateModelValue={page.setShowSaveForm} onSaved={page.loadAll} />
      <AddCodexAccountModal modelValue={page.showAddAccountModal} providers={page.providersApi.providers} canManageAuthAccounts={page.canManageAuthAccounts} initialMethod={page.addAccountInitialMethod} presetProvider={page.addAccountPresetProvider} refreshOnMutation={page.loadAll} onUpdateModelValue={page.setShowAddAccountModal} />
      <RenameCodexAccountModal modelValue={page.showRenameDialog} accountName={page.renameTarget} onUpdateModelValue={page.setShowRenameDialog} onRenamed={page.loadAll} />
      <ConfirmModal
        isOpen={Boolean(page.confirmState)}
        title={page.confirmState?.title ?? ''}
        message={page.confirmState?.message ?? ''}
        confirmText={page.confirmState?.confirmText}
        cancelText={t('common.cancel')}
        type={page.confirmState?.type ?? 'info'}
        onConfirm={page.handleConfirm}
        onCancel={page.handleCancelConfirm}
      />
    </PageShell>
  )
}

function TabChip({
  id,
  label,
  count,
  active,
  icon,
  onSelect,
}: {
  id: ManagerTab
  label: string
  count: number
  active: boolean
  icon: string
  onSelect: (id: ManagerTab) => void
}) {
  const handleClick = useCallback(() => onSelect(id), [id, onSelect])
  return (
    <button type="button" className={active ? 'codex-auth-view__segment codex-auth-view__segment--active' : 'codex-auth-view__segment'} onClick={handleClick}>
      <SIcon name={icon} size="w-4 h-4" />
      <span>{label}</span>
      <span className="codex-auth-view__segment-count">{count}</span>
    </button>
  )
}
