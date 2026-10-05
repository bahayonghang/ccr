import { memo, useCallback, useEffect, useRef, useState } from 'react'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { BaseModal, SIcon } from '@/ui'
import { copyText } from '@/utils/clipboard'
import { logger } from '@/utils/logger'
import {
  llmusageInstallCheck,
  llmusageInstallDetect,
  llmusageInstallExecute,
  llmusageInstallManualCatalog,
  llmusageInstallPlan,
  type DetectionResult,
  type HostCapabilities,
  type InstallEvent,
  type ManualCatalog,
  type PlanOutcome,
  type ProgressStage,
} from '@/api/domains/install'
import { useUsageT } from '../translate'

type DialogState = 'checking' | 'absent-choice' | 'auto-running' | 'auto-terminal' | 'manual'

interface LlmusageInstallDialogProps {
  isOpen: boolean
  onOpenChange: (open: boolean) => void
  onRetryImport: () => void
}

export function LlmusageInstallDialog({
  isOpen,
  onOpenChange,
  onRetryImport,
}: LlmusageInstallDialogProps) {
  const t = useUsageT()
  const [state, setState] = useState<DialogState>('checking')
  const [capabilities, setCapabilities] = useState<HostCapabilities | null>(null)
  const [planOutcome, setPlanOutcome] = useState<PlanOutcome | null>(null)
  const [manualCatalog, setManualCatalog] = useState<ManualCatalog | null>(null)
  const [recentLogs, setRecentLogs] = useState<Array<{ line: string; seq: number }>>([])
  const [currentStage, setCurrentStage] = useState<ProgressStage | null>(null)
  const [terminalOutcome, setTerminalOutcome] = useState<'succeeded' | 'failed' | 'cancelled' | null>(null)
  const [failureMessage, setFailureMessage] = useState('')
  const attemptIdRef = useRef<string | null>(null)
  const unlistenRef = useRef<UnlistenFn | null>(null)

  const cleanup = useCallback(() => {
    unlistenRef.current?.()
    unlistenRef.current = null
    setState('checking')
    setRecentLogs([])
    setCurrentStage(null)
    setTerminalOutcome(null)
    attemptIdRef.current = null
  }, [])

  const handleClose = useCallback(() => {
    onOpenChange(false)
    cleanup()
  }, [cleanup, onOpenChange])

  const startCheck = useCallback(async () => {
    setState('checking')
    try {
      const [det, caps] = await llmusageInstallCheck()
      setCapabilities(caps)
      if (det.status === 'available') {
        onRetryImport()
        handleClose()
        return
      }
      setPlanOutcome(await llmusageInstallPlan(det, caps))
      try {
        setManualCatalog(await llmusageInstallManualCatalog())
      } catch {
        // 手册目录失败不阻断主流程。
      }
      setState('absent-choice')
    } catch (caught) {
      logger.error('[install-dialog] check failed', caught)
      setState('absent-choice')
    }
  }, [handleClose, onRetryImport])

  useEffect(() => {
    if (isOpen) void startCheck()
    else cleanup()
  }, [cleanup, isOpen, startCheck])

  const startAutoInstall = useCallback(async () => {
    if (!planOutcome || planOutcome.kind !== 'plan') return
    setState('auto-running')
    setRecentLogs([])
    setTerminalOutcome(null)
    unlistenRef.current = await listen<InstallEvent>('llmusage.install', (event) => {
      const payload = event.payload
      if (payload.type === 'log') {
        setRecentLogs((previous) => [...previous, { line: payload.line, seq: payload.seq }].slice(-100))
        return
      }
      if (payload.type === 'progress') {
        setCurrentStage(payload.stage)
        return
      }
      if (payload.type === 'succeeded') {
        setState('auto-terminal')
        setTerminalOutcome('succeeded')
        window.setTimeout(() => {
          onRetryImport()
          handleClose()
        }, 1500)
        return
      }
      if (payload.type === 'failed') {
        setState('auto-terminal')
        setTerminalOutcome('failed')
        setFailureMessage(payload.error_message ?? 'install failed')
      }
    })
    try {
      attemptIdRef.current = await llmusageInstallExecute(planOutcome.plan_id)
    } catch (caught) {
      setState('auto-terminal')
      setTerminalOutcome('failed')
      setFailureMessage(String(caught))
    }
  }, [handleClose, onRetryImport, planOutcome])

  // 只渲染 i18n 文案，无用户输入。
  const showManual = useCallback(() => setState('manual'), [])
  const recheckInstallation = useCallback(() => {
    void llmusageInstallDetect().then((det: DetectionResult) => {
      if (det.status === 'available') {
        onRetryImport()
        handleClose()
      }
    })
  }, [handleClose, onRetryImport])
  const descriptionHtml = t('usage.install.dialog.description').replace('llmusage', '<strong>llmusage</strong>')

  return (
    <BaseModal
      modelValue={isOpen}
      title={t('usage.install.dialog.title')}
      closeOnBackdrop={false}
      closeOnEscape={state !== 'auto-running'}
      showClose={state !== 'auto-running'}
      size="md"
      onUpdateModelValue={handleClose}
      onClose={handleClose}
    >
      {state === 'checking' ? (
        <p className="py-8 text-center text-sm text-text-secondary">{t('usage.install.dialog.detecting')}</p>
      ) : null}
      {state === 'absent-choice' ? (
        <div className="space-y-4">
          <p
            className="text-sm text-text-primary"
            dangerouslySetInnerHTML={{ __html: descriptionHtml }}
          />
          <div className="flex gap-3">
            <button
              type="button"
              className="flex-1 rounded-xl bg-accent-primary px-4 py-3 text-sm text-text-inverted"
              disabled={planOutcome?.kind !== 'plan'}
              onClick={startAutoInstall}
            >
              {t('usage.install.dialog.autoInstall')}
            </button>
            <button
              type="button"
              className="flex-1 rounded-xl border border-border-default px-4 py-3 text-sm"
              onClick={showManual}
            >
              {t('usage.install.dialog.manualInstall')}
            </button>
          </div>
        </div>
      ) : null}
      {state === 'auto-running' ? (
        <div className="space-y-3">
          <p>{t('usage.install.dialog.installing')}</p>
          {currentStage ? <p className="text-xs text-text-secondary">{String(currentStage)}</p> : null}
          <div className="max-h-48 overflow-y-auto rounded-lg bg-bg-overlay p-3 font-mono text-xs">
            {recentLogs.map((log) => (
              <div key={log.seq}>{log.line}</div>
            ))}
          </div>
        </div>
      ) : null}
      {state === 'auto-terminal' && terminalOutcome === 'succeeded' ? (
        <div className="py-4 text-center">
          <SIcon name="Check" size="w-10 h-10" className="mx-auto text-accent-success" />
          <p>{t('usage.install.dialog.succeeded')}</p>
        </div>
      ) : null}
      {state === 'auto-terminal' && terminalOutcome === 'failed' ? (
        <div className="space-y-3">
          <p>{t('usage.install.dialog.failed')}</p>
          <p className="font-mono text-xs text-accent-danger">{failureMessage}</p>
        </div>
      ) : null}
      {state === 'manual' ? (
        <div className="space-y-3">
          <p>{t('usage.install.dialog.manualIntro')}</p>
          {manualCatalog?.entries
            .filter((entry) => !capabilities || entry.platform === capabilities.platform)
            .map((cmd) => (
              <ManualCommand key={cmd.command_line} command={cmd.command_line} copyLabel={t('usage.install.dialog.copy')} />
            ))}
          <button
            type="button"
            className="rounded-xl bg-accent-primary px-4 py-2 text-sm text-text-inverted"
            onClick={recheckInstallation}
          >
            {t('usage.install.dialog.recheck')}
          </button>
        </div>
      ) : null}
    </BaseModal>
  )
}

const ManualCommand = memo(function ManualCommand({
  command,
  copyLabel,
}: {
  command: string
  copyLabel: string
}) {
  const handleCopy = useCallback(() => {
    void copyText(command)
  }, [command])
  return (
    <div className="rounded-lg bg-bg-overlay p-3">
      <code className="text-xs">{command}</code>
      <button type="button" className="ml-2 text-xs text-accent-primary" onClick={handleCopy}>
        {copyLabel}
      </button>
    </div>
  )
})
