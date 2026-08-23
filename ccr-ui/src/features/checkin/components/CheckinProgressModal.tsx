import { memo, useCallback, useEffect, useMemo, useRef } from 'react'
import { BaseModal, SIcon } from '@/ui'
import type { CheckinFlowPhase, CheckinLogEntry } from '@/types/checkin'
import type { TranslateFunction } from '@/utils/tf'
import { useCheckinT } from '../hooks/useCheckinT'

const progressTitle = ({
  isRecovering,
  needsManualWaf,
  isFinished,
  t,
}: {
  isRecovering: boolean
  needsManualWaf: boolean
  isFinished: boolean
  t: TranslateFunction
}): string => {
  if (isRecovering) return t('checkin.result.recoveringTitle')
  if (needsManualWaf) return t('checkin.progress.manualWafTitle')
  if (isFinished) return t('checkin.result.completed')
  return t('checkin.progress.runningTitle')
}

interface CheckinProgressModalProps {
  isOpen: boolean
  total: number
  current: number
  currentAccountName: string
  logs: CheckinLogEntry[]
  phase: CheckinFlowPhase
  recoveryMessage?: string | null
  recoveryProviderName?: string | null
  onClose?: () => void
}

const LOG_TEXT: Record<CheckinLogEntry['status'], string> = {
  success: 'text-accent-success',
  already_checked_in: 'text-accent-warning',
  failed: 'text-accent-danger',
  processing: 'text-accent-info',
  pending: 'text-text-secondary',
  skipped: 'text-text-secondary',
}

const LOG_ICON: Record<CheckinLogEntry['status'], { name: string; className: string }> = {
  processing: { name: 'Loader2', className: 'animate-spin text-accent-info' },
  success: { name: 'CheckCircle', className: 'text-accent-success' },
  already_checked_in: { name: 'Clock', className: 'text-accent-warning' },
  failed: { name: 'XCircle', className: 'text-accent-danger' },
  pending: { name: 'Circle', className: 'text-text-muted' },
  skipped: { name: 'Circle', className: 'text-text-muted' },
}

const LogRow = memo(function LogRow({
  log,
  recoveryLabel,
}: {
  log: CheckinLogEntry
  recoveryLabel: string | null
}) {
  const statusClass = LOG_TEXT[log.status] ?? 'text-text-secondary'
  const icon = LOG_ICON[log.status] ?? LOG_ICON.pending

  return (
    <div className="flex items-start gap-2 text-sm">
      <span className="mt-0.5 flex-shrink-0">
        <SIcon name={icon.name} size="h-4 w-4" className={icon.className} />
      </span>
      <div className="min-w-0 flex-1">
        <span className={`font-medium ${statusClass}`}>{log.accountName}</span>
        <span className="ml-1 text-text-muted">({log.providerName})</span>
        {recoveryLabel ? (
          <span
            className={`ml-2 inline-flex rounded-full px-2 py-0.5 text-xs font-medium ${
              log.wafRecovered
                ? 'bg-accent-info/15 text-accent-info'
                : 'bg-accent-warning/15 text-accent-warning'
            }`}
          >
            {recoveryLabel}
          </span>
        ) : null}
        {log.message ? (
          <p
            className={`mt-0.5 break-all text-xs ${
              log.status === 'failed' ? 'text-accent-danger' : 'text-text-secondary'
            }`}
          >
            {log.message}
          </p>
        ) : null}
        {log.wafRecoveryAttempted && log.wafRecovered === false && log.wafRecoveryError ? (
          <p className="mt-0.5 break-all text-xs text-accent-warning">{log.wafRecoveryError}</p>
        ) : null}
      </div>
    </div>
  )
})

export function CheckinProgressModal({
  isOpen,
  total,
  current,
  currentAccountName,
  logs,
  phase,
  recoveryMessage,
  recoveryProviderName,
  onClose,
}: CheckinProgressModalProps) {
  const t = useCheckinT()
  const logContainerRef = useRef<HTMLDivElement | null>(null)
  const radius = 42
  const circumference = 2 * Math.PI * radius
  const isRecovering = phase === 'recovering'
  const isFinished = phase === 'finished'
  const unresolvedWafCount = logs.filter(
    (log) => log.status === 'failed' && log.errorCode === 'waf_blocked',
  ).length
  const needsManualWaf = isFinished && unresolvedWafCount > 0

  const modalTitle = progressTitle({ isRecovering, needsManualWaf, isFinished, t })

  const progressPercent = total === 0 ? 0 : Math.round((current / total) * 100)
  const progressOffset = circumference * (1 - (total === 0 ? 0 : current / total))

  const recoveryLabels = useMemo(
    () =>
      logs.map((log) => {
        if (!log.wafRecoveryAttempted) return null
        if (log.wafRecovered) {
          return log.status === 'already_checked_in'
            ? t('checkin.result.recoveryCompleted')
            : t('checkin.result.recoverySuccess')
        }
        return t('checkin.progress.recoveryFailed')
      }),
    [logs, t],
  )

  useEffect(() => {
    const node = logContainerRef.current
    if (node) node.scrollTop = node.scrollHeight
  }, [logs.length])

  const handleClose = useCallback(() => {
    onClose?.()
  }, [onClose])

  return (
    <BaseModal
      modelValue={isOpen}
      title={modalTitle}
      closeOnBackdrop={isFinished}
      closeOnEscape={isFinished}
      showClose={false}
      persistent={!isFinished}
      surface="solid"
      size="md"
    >
      <div className="space-y-6 py-4">
        <ProgressHero
          circumference={circumference}
          progressOffset={progressOffset}
          progressPercent={progressPercent}
          isFinished={isFinished}
          isRecovering={isRecovering}
          needsManualWaf={needsManualWaf}
          current={current}
          total={total}
          currentAccountName={currentAccountName}
          recoveryMessage={recoveryMessage}
          recoveryProviderName={recoveryProviderName}
          unresolvedWafCount={unresolvedWafCount}
          t={t}
        />

        <div className="space-y-2">
          <h4 className="flex items-center gap-2 text-sm font-medium text-text-secondary">
            <SIcon name="FileText" size="h-4 w-4" />
            {t('checkin.progress.logTitle')}
          </h4>
          <div
            ref={logContainerRef}
            className="h-48 space-y-1.5 overflow-y-auto rounded-lg border border-border-default bg-bg-base p-3 shadow-inner"
          >
            {logs.map((log, index) => (
              <LogRow
                key={`${log.accountId}-${log.timestamp.getTime()}`}
                log={log}
                recoveryLabel={recoveryLabels[index] ?? null}
              />
            ))}
            {logs.length === 0 ? (
              <div className="flex h-full items-center justify-center text-sm text-text-muted">
                {t('checkin.progress.waiting')}
              </div>
            ) : null}
          </div>
        </div>

        {isFinished ? (
          <div className="pt-2">
            <button
              type="button"
              className="flex w-full items-center justify-center gap-2 rounded-lg bg-accent-primary px-4 py-2.5 font-medium text-text-inverted transition-colors hover:bg-accent-primary/90"
              onClick={handleClose}
            >
              <SIcon name="CheckCircle" size="h-5 w-5" />
              {t('common.confirm')}
            </button>
          </div>
        ) : null}
      </div>
    </BaseModal>
  )
}

function ProgressHero({
  circumference,
  progressOffset,
  progressPercent,
  isFinished,
  isRecovering,
  needsManualWaf,
  current,
  total,
  currentAccountName,
  recoveryMessage,
  recoveryProviderName,
  unresolvedWafCount,
  t,
}: {
  circumference: number
  progressOffset: number
  progressPercent: number
  isFinished: boolean
  isRecovering: boolean
  needsManualWaf: boolean
  current: number
  total: number
  currentAccountName: string
  recoveryMessage?: string | null
  recoveryProviderName?: string | null
  unresolvedWafCount: number
  t: TranslateFunction
}) {
  const centerIcon = isFinished ? (
    <SIcon
      name={needsManualWaf ? 'AlertTriangle' : 'CheckCircle'}
      size="h-10 w-10"
      className={needsManualWaf ? 'text-accent-warning' : 'text-accent-success'}
    />
  ) : null
  const recoveringIcon = isRecovering ? (
    <SIcon name="Loader2" size="h-10 w-10" className="animate-spin text-accent-info" />
  ) : null
  return (
    <div className="space-y-3 text-center">
      <div className="relative inline-flex h-24 w-24 items-center justify-center">
        <svg className="h-24 w-24 -rotate-90 transform">
          <circle cx="48" cy="48" r="42" stroke="currentColor" strokeWidth="6" fill="none" className="text-border-default" />
          <circle
            cx="48"
            cy="48"
            r="42"
            stroke="currentColor"
            strokeWidth="6"
            fill="none"
            className="text-accent-primary transition-[stroke-dashoffset,stroke-dasharray] duration-300"
            strokeDasharray={circumference}
            strokeDashoffset={progressOffset}
            strokeLinecap="round"
          />
        </svg>
        <span className="absolute text-2xl font-bold text-text-primary">
          {centerIcon ?? recoveringIcon ?? `${progressPercent}%`}
        </span>
      </div>
      <ProgressCaption
        current={current}
        total={total}
        currentAccountName={currentAccountName}
        isFinished={isFinished}
        isRecovering={isRecovering}
        needsManualWaf={needsManualWaf}
        recoveryMessage={recoveryMessage}
        recoveryProviderName={recoveryProviderName}
        unresolvedWafCount={unresolvedWafCount}
        t={t}
      />
    </div>
  )
}

function ProgressCaption({
  current,
  total,
  currentAccountName,
  isFinished,
  isRecovering,
  needsManualWaf,
  recoveryMessage,
  recoveryProviderName,
  unresolvedWafCount,
  t,
}: {
  current: number
  total: number
  currentAccountName: string
  isFinished: boolean
  isRecovering: boolean
  needsManualWaf: boolean
  recoveryMessage?: string | null
  recoveryProviderName?: string | null
  unresolvedWafCount: number
  t: TranslateFunction
}) {
  const finishedText = needsManualWaf
    ? t('checkin.progress.manualWafSummary', { count: unresolvedWafCount })
    : t('checkin.progress.allTasksCompleted')
  return (
    <div className="space-y-1">
      <p className="text-sm text-text-secondary">{t('checkin.progress.accountProgress', { current, total })}</p>
      {currentAccountName && !isFinished && !isRecovering ? (
        <p className="text-sm font-medium text-accent-primary">
          {t('checkin.progress.currentAccount', { account: currentAccountName })}
        </p>
      ) : null}
      {isRecovering && recoveryMessage ? (
        <p className="text-sm font-medium text-accent-info">{recoveryMessage}</p>
      ) : null}
      {isFinished ? (
        <p className={`text-sm font-medium ${needsManualWaf ? 'text-accent-warning' : 'text-accent-success'}`}>
          {finishedText}
        </p>
      ) : null}
      {isRecovering && recoveryProviderName ? (
        <p className="text-xs text-text-secondary">
          {t('checkin.result.currentProvider', { provider: recoveryProviderName })}
        </p>
      ) : null}
    </div>
  )
}
