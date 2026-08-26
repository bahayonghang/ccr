import { useCallback, useEffect, useState } from 'react'
import { useForm } from 'react-hook-form'
import { detectCodexProcess, saveCodexAuth } from '@/api'
import { surfaceNotify } from '@/configs/surfaceNotify'
import { BaseModal, SIcon, buttonClass } from '@/ui'
import type { CodexAuthCurrentInfo } from '@/types'
import { extractErrorMessage } from '@/utils/errorHandler'
import { logger } from '@/utils/logger'
import { fieldInputClass } from '../ui-classes'
import { useCodexLocale } from '../useCodexLocale'

interface SaveForm {
  name: string
  description: string
  force: boolean
}

interface SaveCodexSessionModalProps {
  modelValue: boolean
  currentInfo: CodexAuthCurrentInfo | null
  formatAuthMethod: (method?: string | null) => string
  onUpdateModelValue: (value: boolean) => void
  onSaved: () => void
}

export function SaveCodexSessionModal({
  modelValue,
  currentInfo,
  formatAuthMethod,
  onUpdateModelValue,
  onSaved,
}: SaveCodexSessionModalProps) {
  const { t, tf } = useCodexLocale()
  const form = useForm<SaveForm>({ defaultValues: { name: '', description: '', force: false } })
  const [processWarning, setProcessWarning] = useState<string | null>(null)
  const values = form.watch()

  useEffect(() => {
    if (!modelValue) return
    form.reset({ name: currentInfo?.email?.split('@')[0] || '', description: '', force: false })
    void detectCodexProcess()
      .then((info) => {
        setProcessWarning(info.has_running_process ? info.warning || t('codex.auth.processDetected', { pids: info.pids.join(', ') }) : null)
      })
      .catch(() => setProcessWarning(null))
  }, [currentInfo, form, modelValue, t])

  const handleClose = useCallback(() => {
    onUpdateModelValue(false)
    setProcessWarning(null)
  }, [onUpdateModelValue])
  const handleOpenChange = useCallback(
    (open: boolean) => {
      if (!open) handleClose()
    },
    [handleClose],
  )
  const onSubmit = form.handleSubmit(async (data) => {
    if (!data.name.trim()) {
      surfaceNotify.error(t('codex.auth.validation.nameRequired'))
      return
    }
    try {
      await saveCodexAuth({ name: data.name.trim(), description: data.description.trim() || undefined, force: data.force })
      handleClose()
      onSaved()
      surfaceNotify.success(tf('codex.auth.feedback.saveCurrentSuccess', 'Current session saved as an account.'))
    } catch (error) {
      logger.error('Failed to save auth:', error)
      surfaceNotify.error(extractErrorMessage(error) || t('codex.states.saveFailed'))
    }
  })

  return (
    <BaseModal
      modelValue={modelValue}
      title={tf('codex.auth.actions.saveCurrent', 'Save current session')}
      size="full"
      surface="glass"
      contentClass="w-full max-w-[min(48.75rem,calc(100vw-2rem))] max-h-[90vh] overflow-y-auto"
      onUpdateModelValue={handleOpenChange}
    >
      <form className="codex-auth-view__save-shell" onSubmit={onSubmit}>
        <p className="codex-auth-view__save-lede">
          {tf('codex.auth.saveModal.lede', 'Store the current Codex login as a reusable CCR account entry with a clearer label, optional notes, and an expiration reminder.')}
        </p>
        <div className="codex-auth-view__save-meta">
          <span className="codex-auth-view__meta-pill">{currentInfo?.email || tf('codex.auth.saveModal.meta.runtimeOnly', 'Current runtime session')}</span>
          <span className="codex-auth-view__meta-pill">{formatAuthMethod(currentInfo?.auth_method)}</span>
        </div>
        {processWarning ? (
          <div className="rounded-lg border border-accent-warning/30 bg-accent-warning/10 p-4 text-accent-warning">
            <p className="font-medium">{t('codex.auth.processWarning')}</p>
            <p className="mt-1 text-sm">{processWarning}</p>
          </div>
        ) : null}
        <label className="space-y-1.5">
          <span className="text-sm font-semibold text-text-primary">{t('codex.auth.fields.accountName')}</span>
          <input className={fieldInputClass} {...form.register('name')} />
        </label>
        <label className="space-y-1.5">
          <span className="text-sm font-semibold text-text-primary">{t('codex.auth.fields.description')}</span>
          <input className={fieldInputClass} {...form.register('description')} />
        </label>
        <label className="flex items-center gap-2">
          <input type="checkbox" {...form.register('force')} />
          <span>{t('codex.auth.forceOverwrite')}</span>
        </label>
        <div className="flex justify-end gap-3">
          <button type="button" className={buttonClass({ variant: 'ghost' })} onClick={handleClose}>{t('codex.actions.cancel')}</button>
          <button type="submit" className={buttonClass({ variant: 'primary' })} disabled={form.formState.isSubmitting || !values.name.trim()}>
            <SIcon name="Save" size="w-4 h-4" />
            {form.formState.isSubmitting ? t('codex.states.saving') : t('codex.actions.save')}
          </button>
        </div>
      </form>
    </BaseModal>
  )
}
