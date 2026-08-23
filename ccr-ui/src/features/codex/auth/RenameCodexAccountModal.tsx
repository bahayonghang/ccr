import { useCallback, useEffect } from 'react'
import { useForm } from 'react-hook-form'
import { renameCodexAuth } from '@/api'
import { surfaceNotify } from '@/configs/surfaceNotify'
import { BaseModal, SIcon } from '@/ui'
import { extractErrorMessage } from '@/utils/errorHandler'
import { logger } from '@/utils/logger'
import { canSubmitAccountRename } from '../codexAuthAccounts'
import { fieldInputClass, ghostBtnClass, primaryBtnClass } from '../ui-classes'
import { useCodexLocale } from '../useCodexLocale'

interface RenameForm {
  oldName: string
  newName: string
  force: boolean
}

interface RenameCodexAccountModalProps {
  modelValue: boolean
  accountName: string
  onUpdateModelValue: (value: boolean) => void
  onRenamed: () => void
}

export function RenameCodexAccountModal({
  modelValue,
  accountName,
  onUpdateModelValue,
  onRenamed,
}: RenameCodexAccountModalProps) {
  const { t, tf } = useCodexLocale()
  const form = useForm<RenameForm>({ defaultValues: { oldName: '', newName: '', force: false } })
  const values = form.watch()
  const canSubmit = canSubmitAccountRename(values.oldName, values.newName)

  useEffect(() => {
    if (!modelValue) return
    form.reset({ oldName: accountName, newName: accountName, force: false })
  }, [accountName, form, modelValue])

  const handleClose = useCallback(() => {
    if (form.formState.isSubmitting) return
    onUpdateModelValue(false)
  }, [form.formState.isSubmitting, onUpdateModelValue])
  const handleOpenChange = useCallback(
    (open: boolean) => {
      if (!open) handleClose()
    },
    [handleClose],
  )
  const onSubmit = form.handleSubmit(async (data) => {
    try {
      await renameCodexAuth(data.oldName, data.newName.trim(), data.force)
      onUpdateModelValue(false)
      onRenamed()
      surfaceNotify.success(tf('codex.auth.rename.success', 'Account renamed.'))
    } catch (error) {
      logger.error('Failed to rename auth:', error)
      surfaceNotify.error(extractErrorMessage(error) || t('codex.states.saveFailed'))
    }
  })

  return (
    <BaseModal modelValue={modelValue} title={tf('codex.auth.rename.title', '重命名 Codex 账号')} size="md" surface="glass" onUpdateModelValue={handleOpenChange}>
      <form className="space-y-4 p-5" onSubmit={onSubmit}>
        <div className="space-y-1.5">
          <span className="text-xs font-semibold tracking-wider text-text-muted uppercase">{tf('codex.auth.rename.currentLabel', '当前名称')}</span>
          <div className="rounded-lg border border-border-default/15 bg-bg-elevated px-3 py-2 font-mono text-sm text-text-secondary">{values.oldName || '—'}</div>
        </div>
        <label className="space-y-1.5">
          <span className="text-xs font-semibold tracking-wider text-text-muted uppercase">{tf('codex.auth.rename.newLabel', '新名称')}</span>
          <input id="renameNewName" className={fieldInputClass} {...form.register('newName')} />
        </label>
        <label className="flex cursor-pointer items-center gap-2 text-sm text-text-secondary">
          <input type="checkbox" {...form.register('force')} />
          {tf('codex.auth.rename.forceLabel', '覆盖同名账号 (force)')}
        </label>
        <div className="flex items-center justify-end gap-2">
          <button type="button" className={ghostBtnClass} onClick={handleClose}>{t('common.cancel')}</button>
          <button type="submit" className={primaryBtnClass} disabled={!canSubmit || form.formState.isSubmitting}>
            <SIcon name="Pencil" size="w-4 h-4" />
            {tf('codex.auth.rename.confirm', '重命名')}
          </button>
        </div>
      </form>
    </BaseModal>
  )
}
