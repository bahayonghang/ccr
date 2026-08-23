import { useEffect, useState } from 'react'
import { checkUpdate, getVersion, updateCCR } from '@/api'
import type { VersionInfo } from '@/types/generated/system/VersionInfo'
import { useShellT } from '@/shell/i18n'
import { logger } from '@/utils/logger'
import { SIcon } from '@/ui/s-icon'
import { UpdateModal, type UpdateStage } from './UpdateModal'

interface UpdateResult {
  success: boolean
  output?: string
  error?: string
}

export function VersionManager() {
  const t = useShellT()
  const [versionInfo, setVersionInfo] = useState<VersionInfo | null>(null)
  const [updateInfo, setUpdateInfo] = useState<VersionInfo | null>(null)
  const [isCheckingUpdate, setIsCheckingUpdate] = useState(false)
  const [showUpdateModal, setShowUpdateModal] = useState(false)
  const [updateStage, setUpdateStage] = useState<UpdateStage>('confirm')
  const [updateOutput, setUpdateOutput] = useState('')
  const [updateError, setUpdateError] = useState('')

  const loadVersionInfo = async () => {
    try {
      setVersionInfo(await getVersion())
    } catch (error) {
      logger.error('Failed to load version info:', error)
    }
  }

  useEffect(() => {
    void loadVersionInfo()
  }, [])

  const handleCheckUpdate = async () => {
    setIsCheckingUpdate(true)
    try {
      setUpdateInfo(await checkUpdate())
    } catch (error) {
      logger.error('Failed to check for updates:', error)
    } finally {
      setIsCheckingUpdate(false)
    }
  }

  const handleConfirmUpdate = async () => {
    setUpdateStage('updating')
    setUpdateOutput(t('common.updateModal.outputStart'))
    try {
      const result = await updateCCR<UpdateResult>()
      if (result.success) {
        setUpdateOutput(result.output || t('common.updateModal.outputCompleted'))
        setUpdateStage('success')
        window.setTimeout(() => {
          void loadVersionInfo()
          setUpdateInfo(null)
        }, 1000)
        return
      }
      setUpdateOutput(result.output || '')
      setUpdateError(result.error || t('common.updateModal.outputError'))
      setUpdateStage('error')
    } catch (error) {
      logger.error('Failed to update CCR:', error)
      setUpdateError(error instanceof Error ? error.message : t('common.updateModal.outputUnexpectedError'))
      setUpdateStage('error')
    }
  }

  return (
    <div className="rounded-lg border border-border-default p-4">
      <div className="mb-3 flex items-center justify-between">
        <span className="text-xs font-semibold tracking-wider text-text-secondary uppercase">
          {t('common.versionManager.title')}
        </span>
        <SIcon name="Zap" size="w-4 h-4" className="text-accent-primary" />
      </div>
      {versionInfo ? (
        <div className="mb-3">
          <div className="mb-1 text-xs text-text-muted">{t('common.versionManager.currentVersion')}</div>
          <div className="font-mono text-2xl font-bold tracking-wide text-accent-primary">
            {t('common.versionPrefix')}
            {versionInfo.current}
          </div>
        </div>
      ) : null}
      {updateInfo?.update_available ? (
        <div className="mb-3 rounded-lg border border-success/40 p-2.5 text-success">
          {t('common.versionManager.updateAvailable')} {t('common.versionPrefix')}
          {updateInfo.latest ?? updateInfo.current}
        </div>
      ) : null}
      {updateInfo && !updateInfo.update_available ? (
        <div className="mb-3 inline-flex w-full items-center justify-center gap-1.5 py-1.5 text-xs text-text-muted">
          <SIcon name="Check" size="w-3.5 h-3.5" />
          <span>{t('common.versionManager.upToDate')}</span>
        </div>
      ) : null}
      <div className="grid grid-cols-2 gap-2">
        <button
          type="button"
          disabled={isCheckingUpdate}
          className="flex items-center justify-center gap-1.5 rounded-lg border border-border-default px-3 py-2 text-xs font-semibold disabled:opacity-50"
          onClick={() => void handleCheckUpdate()}
        >
          <SIcon name="RefreshCw" size="w-3.5 h-3.5" className={isCheckingUpdate ? 'animate-spin' : undefined} />
          <span>
            {isCheckingUpdate ? t('common.versionManager.checking') : t('ccrControl.checkUpdate')}
          </span>
        </button>
        <button
          type="button"
          className="flex items-center justify-center gap-1.5 rounded-lg bg-accent-primary px-3 py-2 text-xs font-semibold text-[color:var(--color-accent-primary-contrast)]"
          onClick={() => {
            setUpdateStage('confirm')
            setUpdateOutput('')
            setUpdateError('')
            setShowUpdateModal(true)
          }}
        >
          <SIcon name="Zap" size="w-3.5 h-3.5" />
          <span>{t('ccrControl.updateNow')}</span>
        </button>
      </div>
      <UpdateModal
        isOpen={showUpdateModal}
        stage={updateStage}
        output={updateOutput}
        error={updateError}
        onClose={() => setShowUpdateModal(false)}
        onConfirm={() => void handleConfirmUpdate()}
      />
    </div>
  )
}
