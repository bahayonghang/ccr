import { useCallback, useMemo } from 'react'
import { useNavigate } from 'react-router'
import { ConfirmModal, PageHeader, PageShell, PillToggleGroup, StatTile } from '@/ui'
import { CheckinProgressModal } from './components/CheckinProgressModal'
import { CheckinResultPanel } from './components/CheckinResultPanel'
import { OAuthWizardModal } from './components/OAuthWizardModal'
import { useCheckinState } from './hooks/useCheckinState'
import { CheckinAccountsTab } from './tabs/CheckinAccountsTab'
import { CheckinImportExportTab } from './tabs/CheckinImportExportTab'
import { CheckinProvidersTab } from './tabs/CheckinProvidersTab'
import { CheckinRecordsTab } from './tabs/CheckinRecordsTab'
import { CHECKIN_TABS, type CheckinTabId } from './lib/checkinFormat'
import './styles/view.css'

export function CheckinView() {
  const navigate = useNavigate()
  const state = useCheckinState()
  const translate = state.t
  const tabOptions = useMemo(
    () => CHECKIN_TABS.map((tab) => ({ value: tab.id, label: translate(tab.nameKey) })),
    [translate],
  )

  const openDashboard = useCallback(
    (accountId: string) => {
      navigate(`/checkin/manage/${accountId}`)
    },
    [navigate],
  )
  const openConfirm = useCallback(() => state.setShowCheckinConfirm(true), [state])
  const closeConfirm = useCallback(() => state.setShowCheckinConfirm(false), [state])
  const openOAuth = useCallback(() => state.setShowOAuthWizard(true), [state])
  const closeOAuth = useCallback(() => state.setShowOAuthWizard(false), [state])
  const openProviders = useCallback(() => state.setActiveTab('providers'), [state])
  const onTabChange = useCallback(
    (value: string | number) => {
      state.setActiveTab(value as CheckinTabId)
    },
    [state],
  )
  const onConfirmOpenChange = useCallback(
    (open: boolean) => {
      state.setShowCheckinConfirm(open)
    },
    [state],
  )

  return (
    <PageShell
      className="checkin-view"
      header={
        <PageHeader
          title={state.t('checkin.title')}
          description={state.t('checkin.description')}
          actions={
            <>
              <button
                type="button"
                disabled={state.loading || state.checkinLoading || state.enabledAccounts.length === 0}
                className="checkin-view__action-button checkin-view__action-button--checkin"
                onClick={openConfirm}
              >
                {state.checkinLoading ? state.t('checkin.actions.checkingAll') : state.t('checkin.actions.checkAll')}
              </button>
              <button
                type="button"
                disabled={state.balanceRefreshing || state.accounts.length === 0}
                className="checkin-view__action-button checkin-view__action-button--balance"
                onClick={state.refreshAllBalances}
              >
                {state.balanceRefreshing
                  ? state.t('checkin.actions.refreshing')
                  : state.t('checkin.actions.refreshBalances')}
              </button>
            </>
          }
        />
      }
    >
      {state.loading ? <div className="checkin-view__loading" /> : null}
      {state.error ? (
        <div className="checkin-view__error">
          <h3>{state.t('checkin.errors.loadFailed')}</h3>
          <p>{state.error}</p>
        </div>
      ) : null}
      {state.checkinResult ? (
        <CheckinResultPanel
          result={state.checkinResult}
          phase={state.checkinFlowPhase}
          resultRef={state.setCheckinResultRef}
          wafRunning={state.wafRecoveryRunning}
          wafMessage={state.wafRecoveryMessage}
          wafProviderName={state.wafRecoveryProviderName}
          successItems={state.successCheckinResults}
          failedItems={state.failedCheckinResults}
          skippedItems={state.skippedCheckinResults}
          alreadyItems={state.alreadyCheckedInResults}
          t={state.t}
          getSuccessDetail={state.getSuccessDetail}
          getFailedDetail={state.getFailedDetail}
          getSkippedDetail={state.getSkippedDetail}
          getAlreadyDetail={state.getAlreadyCheckedInDetail}
          getErrorLabel={state.getErrorLabel}
          onOpenProviders={openProviders}
          onFixCookie={state.openAccountCookieFix}
          onClose={state.setCheckinResultNull}
        />
      ) : null}
      {!state.loading && !state.error ? (
        <div className="checkin-view__content">
          <div className="checkin-view__stats">
            <StatTile
              label={state.t('checkin.stats.currentBalance')}
              value={`$${state.totalStatistics.currentBalance.toFixed(2)}`}
            />
            <StatTile
              label={state.t('checkin.stats.totalQuota')}
              value={`$${state.totalStatistics.totalQuota.toFixed(2)}`}
            />
            <StatTile
              label={state.t('checkin.stats.totalConsumed')}
              value={`$${state.totalStatistics.totalConsumed.toFixed(2)}`}
            />
          </div>
          <div className="checkin-view__tabs-shell">
            <PillToggleGroup options={tabOptions} value={state.activeTab} onValueChange={onTabChange} />
          </div>
          {state.activeTab === 'providers' ? (
            <CheckinProvidersTab
              providers={state.providers}
              builtinProviders={state.builtinProviders}
              onAddBuiltin={state.addBuiltinProvider}
              onRefresh={state.loadAllData}
            />
          ) : null}
          {state.activeTab === 'accounts' ? (
            <CheckinAccountsTab
              accounts={state.accounts}
              checkinLoading={state.checkinLoading}
              providers={state.providers}
              builtinProviders={state.builtinProviders}
              pendingEditAccountId={state.pendingEditAccountId}
              onRefresh={state.loadAllData}
              onCheckin={state.executeCheckinSingle}
              onRefreshBalance={state.refreshAccountBalance}
              onNavigate={openDashboard}
              onShowOauthWizard={openOAuth}
              onPendingEditConsumed={state.clearPendingEditAccount}
            />
          ) : null}
          {state.activeTab === 'records' ? (
            <CheckinRecordsTab
              records={state.records}
              recordsLoadError={state.recordsLoadError}
              providers={state.providers}
              accounts={state.accounts}
              todayStats={state.todayStats}
              onUpdateCookie={state.openAccountCookieFix}
            />
          ) : null}
          {state.activeTab === 'import-export' ? (
            <CheckinImportExportTab onRefresh={state.loadAllData} />
          ) : null}
        </div>
      ) : null}

      <ConfirmModal
        isOpen={state.showCheckinConfirm}
        title={state.t('checkin.dialog.confirmAllTitle')}
        message={state.t('checkin.dialog.confirmAllMessage', { count: state.enabledAccounts.length })}
        confirmText={state.t('checkin.dialog.startCheckin')}
        cancelText={state.t('common.cancel')}
        type="info"
        surface="solid"
        onConfirm={state.handleCheckinConfirm}
        onCancel={closeConfirm}
        onOpenChange={onConfirmOpenChange}
      />
      <CheckinProgressModal
        isOpen={state.showProgressModal}
        total={state.checkinProgress.total}
        current={state.checkinProgress.completed}
        currentAccountName={state.checkinProgress.currentAccountName}
        logs={state.checkinLogs}
        phase={state.checkinFlowPhase}
        recoveryMessage={state.wafRecoveryMessage}
        recoveryProviderName={state.wafRecoveryProviderName}
        onClose={state.closeCheckinModal}
      />
      <OAuthWizardModal
        isOpen={state.showOAuthWizard}
        builtinProviders={state.builtinProviders}
        onUpdateIsOpen={state.setShowOAuthWizard}
        onClose={closeOAuth}
        onSuccess={state.handleOAuthSuccess}
      />
    </PageShell>
  )
}
