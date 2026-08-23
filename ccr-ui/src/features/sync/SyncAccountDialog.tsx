import { useCallback, useEffect, useState } from 'react'
import { useForm } from 'react-hook-form'
import { setWebdavConfig, testWebdavConfig } from '@/api'
import {
  WEBDAV_PROVIDER_PRESETS,
  detectProvider,
  type WebDavProvider,
  type WebDavTestResult,
} from '@/types/sync'
import type { SyncStatusView } from '@/types/syncSelection'
import { getErrorMessage } from '@/utils/errorHandler'
import { logger } from '@/utils/logger'
import { BaseModal, Checkbox, Select, SelectContent, SelectItem, SelectTrigger, SelectValue, SIcon, Switch } from '@/ui'
import { useSyncT } from './locale'

interface FormState {
  provider: WebDavProvider
  webdavUrl: string
  username: string
  password: string
  remotePath: string
  autoSync: boolean
  changePassword: boolean
}

interface SyncAccountDialogProps {
  open: boolean
  mode: 'add' | 'edit'
  initial: SyncStatusView | null
  onOpenChange: (open: boolean) => void
  onSaved: () => void
}

function emptyForm(initial: SyncStatusView | null): FormState {
  const provider = detectProvider(initial?.webdav_url)
  const url = initial?.webdav_url ?? (provider === 'custom' ? '' : WEBDAV_PROVIDER_PRESETS[provider])
  return {
    provider,
    webdavUrl: url ?? WEBDAV_PROVIDER_PRESETS.nutstore,
    username: initial?.username ?? '',
    password: '',
    remotePath: initial?.remote_path?.trim() ? initial.remote_path : '/ccr/',
    autoSync: initial?.auto_sync ?? false,
    changePassword: false,
  }
}

export function SyncAccountDialog({ open, mode, initial, onOpenChange, onSaved }: SyncAccountDialogProps) {
  const t = useSyncT()
  const form = useForm<FormState>({ defaultValues: emptyForm(initial) })
  const values = form.watch()
  const [testing, setTesting] = useState(false)
  const [saving, setSaving] = useState(false)
  const [banner, setBanner] = useState<WebDavTestResult | null>(null)
  const passwordRequired = mode === 'add' || values.changePassword
  const canSubmit = Boolean(values.webdavUrl.trim() && values.username.trim() && (!passwordRequired || values.password))
  const passwordPlaceholder = mode === 'edit' && !values.changePassword
    ? t('sync.account.passwordMaskPlaceholder')
    : t('sync.account.passwordPlaceholder')

  useEffect(() => {
    if (!open) return
    form.reset(emptyForm(initial))
    setBanner(null)
    setTesting(false)
    setSaving(false)
  }, [form, initial, open])

  const close = useCallback(() => {
    if (saving) return
    onOpenChange(false)
  }, [onOpenChange, saving])

  const handleProvider = useCallback((next: string) => {
    const provider = next as WebDavProvider
    form.setValue('provider', provider)
    if (provider !== 'custom') form.setValue('webdavUrl', WEBDAV_PROVIDER_PRESETS[provider])
  }, [form])
  const handleAutoSync = useCallback((checked: boolean) => {
    form.setValue('autoSync', checked)
  }, [form])
  const handleChangePassword = useCallback((value: boolean | 'indeterminate') => {
    form.setValue('changePassword', value === true)
  }, [form])

  const payloadOf = useCallback(() => ({
    webdavUrl: form.getValues('webdavUrl').trim(),
    username: form.getValues('username').trim(),
    password: form.getValues('password'),
    remotePath: form.getValues('remotePath').trim() || '/ccr/',
    autoSync: form.getValues('autoSync'),
  }), [form])

  const onTest = useCallback(async () => {
    if (!canSubmit) {
      setBanner({ ok: false, message: t('sync.account.validationError') })
      return
    }
    setTesting(true)
    setBanner(null)
    try {
      setBanner(await testWebdavConfig(payloadOf()))
    } catch (err) {
      logger.error('test_webdav_config failed:', err)
      setBanner({ ok: false, message: getErrorMessage(err) })
    } finally {
      setTesting(false)
    }
  }, [canSubmit, payloadOf, t])

  const onSave = useCallback(async () => {
    if (!canSubmit) {
      setBanner({ ok: false, message: t('sync.account.validationError') })
      return
    }
    setSaving(true)
    try {
      await setWebdavConfig(payloadOf())
      onSaved()
      onOpenChange(false)
    } catch (err) {
      logger.error('set_webdav_config failed:', err)
      setBanner({ ok: false, message: getErrorMessage(err) })
    } finally {
      setSaving(false)
    }
  }, [canSubmit, onOpenChange, onSaved, payloadOf, t])

  const handleTest = useCallback(() => {
    void onTest()
  }, [onTest])
  const handleSave = useCallback(() => {
    void onSave()
  }, [onSave])

  return (
    <BaseModal
      modelValue={open}
      title={mode === 'edit' ? t('sync.account.editTitle') : t('sync.account.addTitle')}
      size="lg"
      surface="solid"
      closeOnBackdrop={!saving}
      closeOnEscape={!saving}
      onUpdateModelValue={onOpenChange}
      onClose={close}
      footer={
        <div className="flex w-full flex-wrap gap-2">
          <button type="button" className="rounded-lg border border-border-default px-4 py-2 text-sm" disabled={saving || !canSubmit} onClick={handleTest}>
            {testing ? t('sync.account.testing') : t('sync.account.testBtn')}
          </button>
          <span className="flex-1" />
          <button type="button" className="rounded-lg border border-border-default px-4 py-2 text-sm" disabled={saving} onClick={close}>{t('sync.account.cancelBtn')}</button>
          <button type="button" className="rounded-lg bg-accent-primary px-4 py-2 text-sm text-[color:var(--color-accent-primary-contrast)]" disabled={saving || !canSubmit} onClick={handleSave}>
            {saving ? t('sync.account.saving') : t('sync.account.saveBtn')}
          </button>
        </div>
      }
    >
      <div className="grid gap-4">
        <AccountFields
          t={t}
          form={form}
          values={values}
          mode={mode}
          saving={saving}
          passwordPlaceholder={passwordPlaceholder}
          banner={banner}
          onProvider={handleProvider}
          onAutoSync={handleAutoSync}
          onChangePassword={handleChangePassword}
        />
      </div>
    </BaseModal>
  )
}

function AccountFields(props: {
  t: ReturnType<typeof useSyncT>
  form: ReturnType<typeof useForm<FormState>>
  values: FormState
  mode: 'add' | 'edit'
  saving: boolean
  passwordPlaceholder: string
  banner: WebDavTestResult | null
  onProvider: (next: string) => void
  onAutoSync: (checked: boolean) => void
  onChangePassword: (value: boolean | 'indeterminate') => void
}) {
  const { t, form, values, mode, saving, passwordPlaceholder, banner } = props
  return (
    <>
      <label className="grid gap-1 text-sm text-text-secondary">
        {t('sync.account.provider')}
        <Select value={values.provider} onValueChange={props.onProvider}>
          <SelectTrigger><SelectValue /></SelectTrigger>
          <SelectContent>
            <SelectItem value="nutstore">{t('sync.account.providerNutstore')}</SelectItem>
            <SelectItem value="nextcloud">{t('sync.account.providerNextcloud')}</SelectItem>
            <SelectItem value="owncloud">{t('sync.account.providerOwncloud')}</SelectItem>
            <SelectItem value="custom">{t('sync.account.providerCustom')}</SelectItem>
          </SelectContent>
        </Select>
      </label>
      {values.provider === 'nutstore' ? <p className="text-xs text-text-muted">{t('sync.account.nutstoreHint')}</p> : null}
      <label className="grid gap-1 text-sm text-text-secondary">
        {t('sync.account.webdavUrlLabel')}
        <input type="url" className="rounded-lg border border-border-default bg-bg-surface px-3 py-2" disabled={saving || values.provider === 'nutstore'} placeholder={t('sync.account.webdavUrlPlaceholder')} {...form.register('webdavUrl')} />
      </label>
      <label className="grid gap-1 text-sm text-text-secondary">
        {t('sync.account.usernameLabel')}
        <input type="text" className="rounded-lg border border-border-default bg-bg-surface px-3 py-2" disabled={saving} placeholder={t('sync.account.usernamePlaceholder')} {...form.register('username')} />
      </label>
      <div className="grid gap-1">
        <div className="flex items-center justify-between">
          <span className="text-sm text-text-secondary">{t('sync.account.passwordLabel')}</span>
          {mode === 'edit' ? (
            <label className="inline-flex items-center gap-2 text-xs text-text-muted">
              <Checkbox checked={values.changePassword} disabled={saving} onCheckedChange={props.onChangePassword} />
              {t('sync.account.passwordChangeBtn')}
            </label>
          ) : null}
        </div>
        <input type="password" className="rounded-lg border border-border-default bg-bg-surface px-3 py-2" placeholder={passwordPlaceholder} disabled={saving || (mode === 'edit' && !values.changePassword)} {...form.register('password')} />
      </div>
      <label className="grid gap-1 text-sm text-text-secondary">
        {t('sync.account.remotePathLabel')}
        <input type="text" className="rounded-lg border border-border-default bg-bg-surface px-3 py-2" disabled={saving} {...form.register('remotePath')} />
      </label>
      <label className="flex items-center justify-between gap-3">
        <span>
          <span className="block text-sm text-text-primary">{t('sync.account.autoSyncLabel')}</span>
          <span className="block text-xs text-text-muted">{t('sync.account.autoSyncHint')}</span>
        </span>
        <Switch checked={values.autoSync} disabled={saving} onCheckedChange={props.onAutoSync} />
      </label>
      {banner ? (
        <div className={`flex items-start gap-2 rounded-xl border px-3 py-2 text-sm ${banner.ok ? 'border-accent-success/30 bg-accent-success/10 text-accent-success' : 'border-accent-danger/30 bg-accent-danger/10 text-accent-danger'}`}>
          <SIcon name={banner.ok ? 'CheckCircle' : 'AlertCircle'} size="w-4 h-4" />
          <div>
            <strong>{banner.ok ? t('sync.account.testOk') : t('sync.account.testFail')}</strong>
            {!banner.ok && banner.message ? <span> {banner.message}</span> : null}
          </div>
        </div>
      ) : null}
    </>
  )
}
