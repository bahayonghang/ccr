import { memo, useCallback, useEffect, useMemo, useState } from 'react'
import { useForm } from 'react-hook-form'
import { listCheckinRecords, exportCheckinRecords } from '@/api'
import { getErrorMessage } from '@/types/api'
import type {
  AccountInfo,
  CheckinProvider,
  CheckinRecordInfo,
  CheckinRecordsQuery,
  CheckinRecordsResponse,
  TodayCheckinStats,
} from '@/types/checkin'
import { logger } from '@/utils/logger'
import { SIcon } from '@/ui'
import { getSkipReasonText, getStatusText } from '../lib/checkinFormat'
import { checkinNotify } from '../lib/checkinNotify'
import { useCheckinLocale, useCheckinT } from '../hooks/useCheckinT'
import '../styles/records.css'

interface CheckinRecordsTabProps {
  records: CheckinRecordInfo[]
  recordsLoadError: string | null
  providers: CheckinProvider[]
  accounts: AccountInfo[]
  todayStats: TodayCheckinStats | null
  onUpdateCookie?: (accountId: string) => void
}

interface HistoryFilters {
  provider: string
  keyword: string
}

const FailedHistoryItem = memo(function FailedHistoryItem({
  accountName,
  accountId,
  showFix,
  fixLabel,
  onFix,
}: {
  accountName: string
  accountId: string
  showFix: boolean
  fixLabel: string
  onFix: (accountId: string) => void
}) {
  const handleFix = useCallback(() => onFix(accountId), [accountId, onFix])
  return (
    <div className="checkin-records__history-item">
      <div>{accountName}</div>
      {showFix ? (
        <button type="button" className="checkin-records__history-button checkin-records__fix-button" onClick={handleFix}>
          {fixLabel}
        </button>
      ) : null}
    </div>
  )
})

const statusClass = (status: string) => {
  if (status === 'success') return 'checkin-records__status-badge--success'
  if (status === 'already_checked_in') return 'checkin-records__status-badge--warning'
  if (status === 'failed') return 'checkin-records__status-badge--danger'
  return 'checkin-records__status-badge--neutral'
}

const RecordRow = memo(function RecordRow({
  record,
  accountName,
  reason,
  statusLabel,
  formattedDate,
  detailsLabel,
  expanded,
  showFix,
  fixLabel,
  onToggle,
  onFix,
}: {
  record: CheckinRecordInfo
  accountName: string
  reason: string
  statusLabel: string
  formattedDate: string
  detailsLabel: string
  expanded: boolean
  showFix: boolean
  fixLabel: string
  onToggle: (id: string) => void
  onFix: (accountId: string) => void
}) {
  const handleToggle = useCallback(() => onToggle(record.id), [onToggle, record.id])
  const handleFix = useCallback(() => onFix(record.account_id), [onFix, record.account_id])
  return (
    <>
      <tr className="checkin-records__table-row">
        <td className="checkin-records__table-cell checkin-records__table-cell--muted">{formattedDate}</td>
        <td className="checkin-records__table-cell checkin-records__table-cell--primary">{accountName}</td>
        <td className="checkin-records__table-cell">
          <span className={`checkin-records__status-badge checkin-badge-pill ${statusClass(record.status)}`}>
            {statusLabel}
          </span>
        </td>
        <td className="checkin-records__table-cell checkin-records__table-cell--success">
          {record.reward || '-'}
        </td>
        <td className="checkin-records__table-cell checkin-records__table-cell--muted">
          {record.balance_after !== undefined && record.balance_after !== null
            ? `$${record.balance_after.toFixed(2)}`
            : '-'}
        </td>
        <td className="checkin-records__table-cell checkin-records__table-cell--muted">{reason}</td>
        <td className="checkin-records__table-cell checkin-records__table-cell--right">
          <div className="checkin-records__row-actions">
            {showFix ? (
              <button type="button" className="checkin-records__detail-toggle checkin-records__fix-button" onClick={handleFix}>
                {fixLabel}
              </button>
            ) : null}
            <button type="button" className="checkin-records__detail-toggle" onClick={handleToggle}>
              {detailsLabel}
            </button>
          </div>
        </td>
      </tr>
      {expanded ? (
        <tr>
          <td colSpan={7} className="checkin-records__table-cell">
            {record.message || '-'}
          </td>
        </tr>
      ) : null}
    </>
  )
})

export function CheckinRecordsTab({
  records,
  recordsLoadError,
  providers,
  accounts,
  onUpdateCookie,
}: CheckinRecordsTabProps) {
  const t = useCheckinT()
  const locale = useCheckinLocale()
  const [expandedIds, setExpandedIds] = useState<string[]>([])
  const [failedRecords, setFailedRecords] = useState<CheckinRecordInfo[]>([])
  const [failedTotal, setFailedTotal] = useState(0)
  const [failedLoading, setFailedLoading] = useState(false)
  const [failedPage, setFailedPage] = useState(1)
  const filters = useForm<HistoryFilters>({ defaultValues: { provider: 'all', keyword: '' } })
  const getFilters = filters.getValues

  const getAccountName = useCallback(
    (accountId: string) => accounts.find((account) => account.id === accountId)?.name || accountId,
    [accounts],
  )

  const getRecordReason = useCallback(
    (record: CheckinRecordInfo) => {
      if (record.message) return record.message
      if (record.status === 'skipped') {
        return getSkipReasonText(record.error_code, t) || t('checkin.detail.skipped')
      }
      if (record.status === 'success') return t('checkin.detail.checkinSuccess')
      if (record.status === 'already_checked_in') return t('checkin.detail.todayAlreadyCheckedIn')
      if (record.status === 'failed') return t('checkin.errors.unknownReason')
      return '-'
    },
    [t],
  )

  const loadFailedHistory = useCallback(async (page: number) => {
    setFailedLoading(true)
    try {
      const { provider, keyword } = getFilters()
      const params: CheckinRecordsQuery = { status: 'failed', page, page_size: 5 }
      if (provider !== 'all') params.provider_id = provider
      if (keyword.trim()) params.keyword = keyword.trim()
      const response = await listCheckinRecords<CheckinRecordsResponse>(params)
      setFailedRecords(response.records)
      setFailedTotal(response.total)
    } catch (error: unknown) {
      logger.error('Failed to load failed history:', error)
    } finally {
      setFailedLoading(false)
    }
  }, [getFilters])

  useEffect(() => {
    void loadFailedHistory(1)
  }, [loadFailedHistory])

  const applyFilters = useCallback(() => {
    setFailedPage(1)
    void loadFailedHistory(1)
  }, [loadFailedHistory])

  const resetFilters = useCallback(() => {
    filters.reset({ provider: 'all', keyword: '' })
    setFailedPage(1)
    void loadFailedHistory(1)
  }, [filters, loadFailedHistory])

  const toggleExpanded = useCallback((id: string) => {
    setExpandedIds((current) => (current.includes(id) ? current.filter((item) => item !== id) : [...current, id]))
  }, [])

  const handleFix = useCallback(
    (accountId: string) => {
      onUpdateCookie?.(accountId)
    },
    [onUpdateCookie],
  )

  const exportFailed = useCallback(async () => {
    try {
      const { provider, keyword } = getFilters()
      const params: CheckinRecordsQuery = { status: 'failed' }
      if (provider !== 'all') params.provider_id = provider
      if (keyword.trim()) params.keyword = keyword.trim()
      const { blob, filename } = await exportCheckinRecords<{ blob: Blob; filename: string }>(params)
      const url = URL.createObjectURL(blob)
      const link = document.createElement('a')
      link.href = url
      link.download = filename
      document.body.appendChild(link)
      link.click()
      link.remove()
      URL.revokeObjectURL(url)
    } catch (error: unknown) {
      checkinNotify.error(
        t('checkin.records.exportFailed', { error: getErrorMessage(error, t('checkin.errors.unknown')) }),
      )
    }
  }, [getFilters, t])

  const failedHistoryTotalPages = useMemo(() => Math.max(1, Math.ceil(failedTotal / 5)), [failedTotal])

  if (recordsLoadError) {
    return (
      <div className="checkin-records">
        <h2 className="checkin-records__title">{t('checkin.records.title')}</h2>
        <div className="checkin-records__error">
          <SIcon name="AlertCircle" size="w-4 h-4" />
          <span>{recordsLoadError}</span>
        </div>
      </div>
    )
  }

  if (records.length === 0) {
    return (
      <div className="checkin-records">
        <h2 className="checkin-records__title">{t('checkin.records.title')}</h2>
        <div className="checkin-records__empty">{t('checkin.records.empty')}</div>
      </div>
    )
  }

  return (
    <div className="checkin-records">
      <h2 className="checkin-records__title">{t('checkin.records.title')}</h2>
      <details className="checkin-records__history">
        <summary className="checkin-records__history-summary">
          {t('checkin.records.failedHistoryTitle', { count: failedTotal })}
        </summary>
        <div className="checkin-records__history-body">
          <div className="checkin-records__history-filters">
            <select className="checkin-records__history-input" {...filters.register('provider')}>
              <option value="all">{t('checkin.records.allProviders')}</option>
              {providers.map((provider) => (
                <option key={provider.id} value={provider.id}>
                  {provider.name}
                </option>
              ))}
            </select>
            <input
              className="checkin-records__history-input"
              placeholder={t('checkin.records.keywordPlaceholder')}
              {...filters.register('keyword')}
            />
            <button type="button" className="checkin-records__history-button" disabled={failedLoading} onClick={applyFilters}>
              {t('checkin.records.filter')}
            </button>
            <button type="button" className="checkin-records__history-button" disabled={failedLoading} onClick={resetFilters}>
              {t('checkin.records.reset')}
            </button>
            <button type="button" className="checkin-records__history-button" disabled={failedLoading} onClick={exportFailed}>
              {t('checkin.records.export')}
            </button>
          </div>
          {failedRecords.map((record) => (
            <FailedHistoryItem
              key={record.id}
              accountName={getAccountName(record.account_id)}
              accountId={record.account_id}
              showFix={record.error_code === 'cookie_expired'}
              fixLabel={t('checkin.records.cookieExpiredFix')}
              onFix={handleFix}
            />
          ))}
        </div>
      </details>
      <div className="checkin-records__table-shell">
        <table className="checkin-records__table">
          <thead>
            <tr>
              <th className="checkin-records__table-heading">{t('checkin.records.time')}</th>
              <th className="checkin-records__table-heading">{t('checkin.records.account')}</th>
              <th className="checkin-records__table-heading">{t('checkin.records.status')}</th>
              <th className="checkin-records__table-heading">{t('checkin.records.reward')}</th>
              <th className="checkin-records__table-heading">{t('checkin.records.balance')}</th>
              <th className="checkin-records__table-heading">{t('checkin.records.reason')}</th>
              <th className="checkin-records__table-heading checkin-records__table-heading--right">
                {t('checkin.records.details')}
              </th>
            </tr>
          </thead>
          <tbody>
            {records.map((record) => (
              <RecordRow
                key={record.id}
                record={record}
                accountName={getAccountName(record.account_id)}
                reason={getRecordReason(record)}
                statusLabel={getStatusText(record.status, t)}
                formattedDate={new Date(record.checked_in_at).toLocaleString(locale)}
                detailsLabel={t('checkin.records.details')}
                expanded={expandedIds.includes(record.id)}
                showFix={record.error_code === 'cookie_expired'}
                fixLabel={t('checkin.records.cookieExpiredFix')}
                onToggle={toggleExpanded}
                onFix={handleFix}
              />
            ))}
          </tbody>
        </table>
      </div>
      <span>{t('checkin.records.pageLabel', { page: failedPage, total: failedHistoryTotalPages })}</span>
    </div>
  )
}
