import { memo, useCallback } from 'react'
import { SIcon } from '@/ui'
import type { CheckinDisplayResponse, CheckinDisplayResult, CheckinFlowPhase } from '@/types/checkin'
import type { TranslateFunction } from '@/utils/tf'

interface CheckinResultPanelProps {
  result: CheckinDisplayResponse
  phase: CheckinFlowPhase
  resultRef: (node: HTMLElement | null) => void
  wafRunning: boolean
  wafMessage: string | null
  wafProviderName: string | null
  successItems: CheckinDisplayResult[]
  failedItems: CheckinDisplayResult[]
  skippedItems: CheckinDisplayResult[]
  alreadyItems: CheckinDisplayResult[]
  t: TranslateFunction
  getSuccessDetail: (item: CheckinDisplayResult) => string
  getFailedDetail: (item: CheckinDisplayResult) => string
  getSkippedDetail: (item: CheckinDisplayResult) => string
  getAlreadyDetail: (item: CheckinDisplayResult) => string
  getErrorLabel: (code?: string) => string | null
  onOpenProviders: () => void
  onFixCookie: (accountId: string) => void
  onClose: () => void
}

const ResultItem = memo(function ResultItem({
  item,
  tone,
  detail,
  extra,
}: {
  item: CheckinDisplayResult
  tone: 'success' | 'danger' | 'info' | 'muted'
  detail: string
  extra?: string | null
}) {
  const icon =
    tone === 'success' ? 'CheckCircle' : tone === 'danger' ? 'XCircle' : tone === 'info' ? 'Calendar' : 'Circle'
  return (
    <div className={`checkin-view__result-item checkin-view__result-item--${tone}`}>
      <SIcon name={icon} size="w-4 h-4" />
      <div>
        <div className="flex flex-wrap items-center gap-2">
          <span>{item.account_name}</span>
          <span>{item.provider_name}</span>
          {extra ? <span>{extra}</span> : null}
        </div>
        <p className="text-xs">{detail}</p>
      </div>
    </div>
  )
})

const FailedItem = memo(function FailedItem({
  item,
  detail,
  label,
  fixLabel,
  stillFailedLabel,
  onFix,
}: {
  item: CheckinDisplayResult
  detail: string
  label: string | null
  fixLabel: string
  stillFailedLabel: string
  onFix: (accountId: string) => void
}) {
  const handleFix = useCallback(() => onFix(item.account_id), [item.account_id, onFix])
  return (
    <div className="checkin-view__result-item checkin-view__result-item--danger">
      <SIcon name="XCircle" size="w-4 h-4" />
      <div>
        <div className="flex flex-wrap items-center gap-2">
          <span>{item.account_name}</span>
          <span>{item.provider_name}</span>
          {label ? <span>{label}</span> : null}
          {item.waf_recovery_attempted && item.waf_recovered === false ? <span>{stillFailedLabel}</span> : null}
          {item.error_code === 'cookie_expired' ? (
            <button type="button" className="checkin-view__result-fix-button" onClick={handleFix}>
              {fixLabel}
            </button>
          ) : null}
        </div>
        <p className="text-xs">{detail}</p>
      </div>
    </div>
  )
})

export function CheckinResultPanel({
  result,
  phase,
  resultRef,
  wafRunning,
  wafMessage,
  wafProviderName,
  successItems,
  failedItems,
  skippedItems,
  alreadyItems,
  t,
  getSuccessDetail,
  getFailedDetail,
  getSkippedDetail,
  getAlreadyDetail,
  getErrorLabel,
  onOpenProviders,
  onFixCookie,
  onClose,
}: CheckinResultPanelProps) {
  const recovering = phase === 'recovering'
  const tone = recovering ? 'recovering' : result.summary.failed > 0 ? 'warning' : 'success'
  const title = recovering
    ? t('checkin.result.recoveringTitle')
    : result.summary.failed > 0
      ? t('checkin.result.completedWithFailures')
      : t('checkin.result.completed')
  const showWaf = failedItems.some((item) => item.error_code === 'waf_blocked')

  return (
    <div ref={resultRef} className={`checkin-view__result checkin-view__result--${tone}`}>
      <div className="checkin-view__result-header">
        <div>
          <h3>{title}</h3>
          <div className="checkin-view__result-summary">
            <span className="checkin-view__result-badge checkin-view__result-badge--success">
              {t('checkin.result.summarySuccess', { count: result.summary.success })}
            </span>
            <span className="checkin-view__result-badge checkin-view__result-badge--info">
              {t('checkin.result.summaryAlready', { count: result.summary.already_checked_in })}
            </span>
            <span className="checkin-view__result-badge checkin-view__result-badge--danger">
              {t('checkin.result.summaryFailed', { count: result.summary.failed })}
            </span>
            {(result.summary.skipped ?? 0) > 0 ? (
              <span className="checkin-view__result-badge">
                {t('checkin.result.summarySkipped', { count: result.summary.skipped })}
              </span>
            ) : null}
            <span className="checkin-view__result-badge">
              {t('checkin.result.summaryTotal', { count: result.summary.total })}
            </span>
          </div>
          {wafRunning && wafMessage ? (
            <div className="checkin-view__callout">
              <p>{t('checkin.result.recoveringTitle')}</p>
              <p>{wafMessage}</p>
              {wafProviderName ? (
                <p>{t('checkin.result.currentProvider', { provider: wafProviderName })}</p>
              ) : null}
            </div>
          ) : null}
          {showWaf ? (
            <div className="checkin-view__callout checkin-view__callout--waf">
              <p>{wafRunning ? t('checkin.waf.runningTitle') : t('checkin.waf.detectedTitle')}</p>
              <p>{wafRunning ? t('checkin.waf.runningMessage') : t('checkin.waf.detectedMessage')}</p>
              <button type="button" className="checkin-view__callout-action" onClick={onOpenProviders}>
                {t('checkin.actions.openProviders')}
              </button>
            </div>
          ) : null}
          {successItems.map((item) => (
            <ResultItem
              key={item.account_id}
              item={item}
              tone="success"
              detail={getSuccessDetail(item)}
              extra={item.waf_recovery_attempted && item.waf_recovered ? t('checkin.result.recoverySuccess') : null}
            />
          ))}
          {failedItems.map((item) => (
            <FailedItem
              key={item.account_id}
              item={item}
              detail={getFailedDetail(item)}
              label={getErrorLabel(item.error_code)}
              fixLabel={t('checkin.actions.updateCookie')}
              stillFailedLabel={t('checkin.result.recoveryStillFailed')}
              onFix={onFixCookie}
            />
          ))}
          {skippedItems.map((item) => (
            <ResultItem key={item.account_id} item={item} tone="muted" detail={getSkippedDetail(item)} />
          ))}
          {alreadyItems.map((item) => (
            <ResultItem
              key={item.account_id}
              item={item}
              tone="info"
              detail={getAlreadyDetail(item)}
              extra={item.waf_recovery_attempted && item.waf_recovered ? t('checkin.result.recoveryCompleted') : null}
            />
          ))}
        </div>
        <button type="button" className="checkin-view__result-close" onClick={onClose}>
          ×
        </button>
      </div>
    </div>
  )
}
