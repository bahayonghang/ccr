import { memo, useCallback } from 'react'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  SIcon,
} from '@/ui'
import type { AccountInfo, CheckinProvider } from '@/types/checkin'
import { useCheckinLocale, useCheckinT } from '../hooks/useCheckinT'
import '../styles/accounts.css'

interface AccountsTableProps {
  accounts: AccountInfo[]
  providers: CheckinProvider[]
  checkinLoading: boolean
  onNavigate: (accountId: string) => void
  onCheckin: (accountId: string) => void
  onRefreshBalance: (accountId: string) => void
  onEdit: (account: AccountInfo) => void
  onDelete: (accountId: string) => void
}

interface AccountRowProps {
  account: AccountInfo
  providerName: string
  formattedDate: string
  checkinLoading: boolean
  checkinLabel: string
  checkingLabel: string
  refreshLabel: string
  editLabel: string
  deleteLabel: string
  onNavigate: (accountId: string) => void
  onCheckin: (accountId: string) => void
  onRefreshBalance: (accountId: string) => void
  onEdit: (account: AccountInfo) => void
  onDelete: (accountId: string) => void
}

const formatMoney = (value?: number | null) =>
  value !== undefined && value !== null ? `$${value.toFixed(2)}` : null

const AccountRow = memo(function AccountRow({
  account,
  providerName,
  formattedDate,
  checkinLoading,
  checkinLabel,
  checkingLabel,
  refreshLabel,
  editLabel,
  deleteLabel,
  onNavigate,
  onCheckin,
  onRefreshBalance,
  onEdit,
  onDelete,
}: AccountRowProps) {
  const handleNavigate = useCallback(() => {
    onNavigate(account.id)
  }, [account.id, onNavigate])
  const handleCheckin = useCallback(() => {
    onCheckin(account.id)
  }, [account.id, onCheckin])
  const handleRefresh = useCallback(() => {
    onRefreshBalance(account.id)
  }, [account.id, onRefreshBalance])
  const handleEdit = useCallback(() => {
    onEdit(account)
  }, [account, onEdit])
  const handleDelete = useCallback(() => {
    onDelete(account.id)
  }, [account.id, onDelete])
  const stopRowClick = useCallback((event: { stopPropagation: () => void }) => {
    event.stopPropagation()
  }, [])

  const balance = formatMoney(account.latest_balance)
  const quota = formatMoney(account.total_quota)
  const consumed = formatMoney(account.total_consumed)
  const checkinText = checkinLoading ? checkingLabel : checkinLabel

  return (
    <tr className="checkin-accounts-tab__row" onClick={handleNavigate}>
      <td className="checkin-accounts-tab__cell">
        <div className="checkin-accounts-tab__account">
          <div className="checkin-accounts-tab__account-row">
            <div
              className={`checkin-accounts-tab__status-dot ${
                account.enabled ? 'bg-accent-success' : 'bg-text-muted'
              }`}
            />
            <span className="checkin-accounts-tab__account-name">{account.name}</span>
          </div>
          <span className="checkin-accounts-tab__provider-chip">{providerName}</span>
        </div>
      </td>
      <td className="checkin-accounts-tab__cell checkin-accounts-tab__cell--right">
        {balance ? (
          <span className="checkin-accounts-tab__metric checkin-accounts-tab__metric--balance">
            {balance}
          </span>
        ) : (
          <span className="checkin-accounts-tab__placeholder">-</span>
        )}
      </td>
      <td className="checkin-accounts-tab__cell checkin-accounts-tab__cell--right">
        {quota ? (
          <span className="checkin-accounts-tab__metric checkin-accounts-tab__metric--quota">{quota}</span>
        ) : (
          <span className="checkin-accounts-tab__placeholder">-</span>
        )}
      </td>
      <td className="checkin-accounts-tab__cell checkin-accounts-tab__cell--right">
        {consumed ? (
          <span className="checkin-accounts-tab__metric checkin-accounts-tab__metric--consumed">
            {consumed}
          </span>
        ) : (
          <span className="checkin-accounts-tab__placeholder">-</span>
        )}
      </td>
      <td className="checkin-accounts-tab__cell checkin-accounts-tab__cell--mono">{formattedDate}</td>
      <td className="checkin-accounts-tab__cell" onClick={stopRowClick}>
        <div className="checkin-accounts-tab__row-actions">
          <button
            type="button"
            disabled={checkinLoading}
            className="checkin-accounts-tab__mini-button"
            title={checkinText}
            onClick={handleCheckin}
          >
            <SIcon
              name={checkinLoading ? 'Loader2' : 'Calendar'}
              size="w-3 h-3"
              className={checkinLoading ? 'animate-spin' : undefined}
            />
            <span className="checkin-accounts-tab__mini-button-label">{checkinText}</span>
          </button>
          <div className="checkin-accounts-tab__menu-wrap">
            <DropdownMenu>
              <DropdownMenuTrigger className="checkin-accounts-tab__menu-trigger">
                <svg className="h-4 w-4" fill="currentColor" viewBox="0 0 20 20">
                  <path d="M10 6a2 2 0 110-4 2 2 0 010 4zM10 12a2 2 0 110-4 2 2 0 010 4zM10 18a2 2 0 110-4 2 2 0 010 4z" />
                </svg>
              </DropdownMenuTrigger>
              <DropdownMenuContent
                align="end"
                side="bottom"
                className="checkin-accounts-tab__menu checkin-accounts-tab__menu--floating checkin-accounts-tab__menu--bottom"
              >
                <DropdownMenuItem className="checkin-accounts-tab__menu-item" onSelect={handleRefresh}>
                  {refreshLabel}
                </DropdownMenuItem>
                <DropdownMenuItem className="checkin-accounts-tab__menu-item" onSelect={handleEdit}>
                  {editLabel}
                </DropdownMenuItem>
                <DropdownMenuItem
                  className="checkin-accounts-tab__menu-item checkin-accounts-tab__menu-item--danger"
                  onSelect={handleDelete}
                >
                  {deleteLabel}
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        </div>
      </td>
    </tr>
  )
})

export function AccountsTable({
  accounts,
  providers,
  checkinLoading,
  onNavigate,
  onCheckin,
  onRefreshBalance,
  onEdit,
  onDelete,
}: AccountsTableProps) {
  const t = useCheckinT()
  const locale = useCheckinLocale()
  const getProviderName = useCallback(
    (providerId: string, fallback?: string) =>
      fallback || providers.find((provider) => provider.id === providerId)?.name || providerId,
    [providers],
  )

  return (
    <div className="checkin-accounts-tab__table-shell checkin-surface-card">
      <table className="checkin-accounts-tab__table">
        <thead className="checkin-accounts-tab__table-head">
          <tr>
            <th className="checkin-accounts-tab__th">{t('checkin.accounts.columns.account')}</th>
            <th className="checkin-accounts-tab__th checkin-accounts-tab__th--right">
              {t('checkin.accounts.columns.balance')}
            </th>
            <th className="checkin-accounts-tab__th checkin-accounts-tab__th--right">
              {t('checkin.accounts.columns.totalQuota')}
            </th>
            <th className="checkin-accounts-tab__th checkin-accounts-tab__th--right">
              {t('checkin.accounts.columns.totalConsumed')}
            </th>
            <th className="checkin-accounts-tab__th">{t('checkin.accounts.columns.lastCheckin')}</th>
            <th className="checkin-accounts-tab__th checkin-accounts-tab__th--actions">
              {t('checkin.accounts.columns.actions')}
            </th>
          </tr>
        </thead>
        <tbody className="checkin-accounts-tab__table-body">
          {accounts.map((account) => (
            <AccountRow
              key={account.id}
              account={account}
              providerName={getProviderName(account.provider_id, account.provider_name)}
              formattedDate={
                account.last_checkin_at
                  ? new Date(account.last_checkin_at).toLocaleString(locale)
                  : '-'
              }
              checkinLoading={checkinLoading}
              checkinLabel={t('checkin.actions.checkIn')}
              checkingLabel={t('checkin.actions.checking')}
              refreshLabel={t('checkin.actions.refreshBalance')}
              editLabel={t('checkin.accounts.edit')}
              deleteLabel={t('checkin.accounts.delete')}
              onNavigate={onNavigate}
              onCheckin={onCheckin}
              onRefreshBalance={onRefreshBalance}
              onEdit={onEdit}
              onDelete={onDelete}
            />
          ))}
        </tbody>
      </table>
    </div>
  )
}
