import { useCallback, useEffect, type FormEvent } from 'react'
import { useForm } from 'react-hook-form'
import { BaseModal, Checkbox, SIcon } from '@/ui'
import { useSyncT } from './locale'
import './styles/sync-passphrase.css'

interface PassphraseForm {
  passphrase: string
  migratePlaintextV1: boolean
}

interface SyncPassphraseModalProps {
  open: boolean
  assetName?: string
  onOpenChange: (open: boolean) => void
  onSubmit: (payload: { passphrase: string; migratePlaintextV1: boolean }) => void
}

export function SyncPassphraseModal({ open, assetName, onOpenChange, onSubmit }: SyncPassphraseModalProps) {
  const t = useSyncT()
  const form = useForm<PassphraseForm>({ defaultValues: { passphrase: '', migratePlaintextV1: false } })
  const passphrase = form.watch('passphrase')
  const migrate = form.watch('migratePlaintextV1')

  const clear = useCallback(() => {
    form.reset({ passphrase: '', migratePlaintextV1: false })
  }, [form])

  const close = useCallback(() => {
    clear()
    onOpenChange(false)
  }, [clear, onOpenChange])

  const handleOpenChange = useCallback((next: boolean) => {
    if (!next) close()
  }, [close])

  const submit = useCallback(() => {
    const values = form.getValues()
    if (!values.passphrase) return
    const payload = { passphrase: values.passphrase, migratePlaintextV1: values.migratePlaintextV1 }
    clear()
    onOpenChange(false)
    onSubmit(payload)
  }, [clear, form, onOpenChange, onSubmit])

  const handleFormSubmit = useCallback((event: FormEvent) => {
    event.preventDefault()
    submit()
  }, [submit])

  const handleMigrate = useCallback((value: boolean | 'indeterminate') => {
    form.setValue('migratePlaintextV1', value === true)
  }, [form])

  useEffect(() => {
    if (!open) clear()
  }, [clear, open])

  return (
    <BaseModal
      modelValue={open}
      size="sm"
      surface="solid"
      title={t('sync.passphrase.title')}
      description={t('sync.passphrase.description')}
      onUpdateModelValue={handleOpenChange}
      onClose={close}
      footer={
        <div className="flex w-full gap-3">
          <button type="button" className="sync-passphrase-button sync-passphrase-button--secondary" onClick={close}>
            {t('common.cancel')}
          </button>
          <button type="button" className="sync-passphrase-button sync-passphrase-button--primary" disabled={passphrase.length === 0} onClick={submit}>
            <SIcon name="KeyRound" size="w-4 h-4" />
            {t('sync.passphrase.continue')}
          </button>
        </div>
      }
    >
      <form className="sync-passphrase-form" onSubmit={handleFormSubmit}>
        <p className="sync-passphrase-target">
          <SIcon name="ShieldCheck" size="w-4 h-4" />
          <span>{assetName || t('sync.passphrase.allAssets')}</span>
        </p>
        <label className="grid gap-1 text-sm text-text-secondary">
          {t('sync.passphrase.label')}
          <input type="password" className="rounded-lg border border-border-default bg-bg-surface px-3 py-2" placeholder={t('sync.passphrase.placeholder')} {...form.register('passphrase')} />
        </label>
        <label className="sync-passphrase-migration">
          <Checkbox checked={migrate} onCheckedChange={handleMigrate} />
          <span>
            <strong>{t('sync.passphrase.migrateTitle')}</strong>
            <small>{t('sync.passphrase.migrateDescription')}</small>
          </span>
        </label>
      </form>
    </BaseModal>
  )
}
