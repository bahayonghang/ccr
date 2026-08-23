import { BaseModal } from '@/ui/base-modal'
import { SIcon } from '@/ui/s-icon'
import { useShellT } from '@/shell/i18n'

export type UpdateStage = 'confirm' | 'updating' | 'success' | 'error'

interface UpdateModalProps {
  isOpen: boolean
  stage: UpdateStage
  output?: string
  error?: string
  onClose: () => void
  onConfirm: () => void
}

const TITLE_KEY: Record<UpdateStage, string> = {
  confirm: 'common.updateModal.confirmTitle',
  updating: 'common.updateModal.updatingTitle',
  success: 'common.updateModal.successTitle',
  error: 'common.updateModal.errorTitle',
}

const STAGE_ICON: Record<UpdateStage, string> = {
  confirm: 'AlertTriangle',
  updating: 'Loader2',
  success: 'CheckCircle',
  error: 'AlertCircle',
}

export function UpdateModal({
  isOpen,
  stage,
  output = '',
  error = '',
  onClose,
  onConfirm,
}: UpdateModalProps) {
  const t = useShellT()
  const persistent = stage === 'updating'

  return (
    <BaseModal
      modelValue={isOpen}
      title={t(TITLE_KEY[stage])}
      size="2xl"
      persistent={persistent}
      closeOnBackdrop={!persistent}
      closeOnEscape={!persistent}
      showClose={!persistent}
      onUpdateModelValue={(open) => {
        if (!open && !persistent) onClose()
      }}
      onClose={() => {
        if (!persistent) onClose()
      }}
      footer={<UpdateFooter stage={stage} onClose={onClose} onConfirm={onConfirm} />}
    >
      <div className="flex items-center gap-3 pb-4">
        <SIcon
          name={STAGE_ICON[stage]}
          size="w-6 h-6"
          className={stage === 'updating' ? 'animate-spin text-accent-primary' : undefined}
        />
        <h2 className="text-xl font-bold">{t(TITLE_KEY[stage])}</h2>
      </div>
      {stage === 'confirm' ? (
        <div className="space-y-4">
          <p className="text-base leading-relaxed">{t('common.updateModal.confirmMessage')}</p>
          <ul className="list-disc space-y-1.5 ml-6 text-sm text-text-muted">
            <li>{t('common.updateModal.noteDuration')}</li>
            <li>{t('common.updateModal.noteDoNotClose')}</li>
            <li>{t('common.updateModal.noteRefresh')}</li>
            <li>{t('common.updateModal.noteSaveWork')}</li>
          </ul>
        </div>
      ) : null}
      {stage === 'updating' ? (
        <div className="space-y-4">
          <p className="text-base font-medium">{t('common.updateModal.runningMessage')}</p>
          {output ? <pre className="max-h-72 overflow-auto rounded-lg bg-bg-elevated p-4 font-mono text-xs">{output}</pre> : null}
        </div>
      ) : null}
      {stage === 'success' ? (
        <div className="space-y-2">
          <p className="font-semibold text-success">{t('common.updateModal.successMessage')}</p>
          <p className="text-sm text-text-secondary">{t('common.updateModal.successHint')}</p>
        </div>
      ) : null}
      {stage === 'error' ? (
        <div className="space-y-2">
          <p className="font-semibold text-danger">{t('common.updateModal.errorTitle')}</p>
          <p className="text-sm text-text-secondary">{t('common.updateModal.errorMessage')}</p>
          {error ? <pre className="max-h-48 overflow-auto font-mono text-xs text-danger">{error}</pre> : null}
        </div>
      ) : null}
    </BaseModal>
  )
}

function UpdateFooter({
  stage,
  onClose,
  onConfirm,
}: {
  stage: UpdateStage
  onClose: () => void
  onConfirm: () => void
}) {
  const t = useShellT()
  if (stage === 'updating') {
    return <p className="text-sm text-text-muted">{t('common.updateModal.runningHint')}</p>
  }
  if (stage === 'confirm') {
    return (
      <div className="flex justify-end gap-3">
        <button type="button" className="rounded-lg border border-border-default px-5 py-2.5 text-sm" onClick={onClose}>
          {t('common.cancel')}
        </button>
        <button
          type="button"
          className="rounded-lg bg-accent-primary px-5 py-2.5 text-sm text-[color:var(--color-accent-primary-contrast)]"
          onClick={onConfirm}
        >
          {t('common.updateModal.confirmAction')}
        </button>
      </div>
    )
  }
  return (
    <div className="flex justify-end gap-3">
      <button type="button" className="rounded-lg border border-border-default px-5 py-2.5 text-sm" onClick={onClose}>
        {t('common.close')}
      </button>
      {stage === 'success' ? (
        <button
          type="button"
          className="rounded-lg bg-success px-5 py-2.5 text-sm text-[color:var(--color-success-contrast)]"
          onClick={() => window.location.reload()}
        >
          {t('common.updateModal.refreshPage')}
        </button>
      ) : null}
    </div>
  )
}
