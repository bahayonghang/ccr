import { Link } from 'react-router'
import type { UseFormReturn } from 'react-hook-form'
import type { SettingsConfig, SettingsField, SettingsSaveResult, SettingsSnapshot, SettingsValues } from '@/configs/settings'
import { TabButton } from '@/features/platform/TabButton'
import { SettingsFieldControl } from './SettingsFieldControl'
import { Button } from '@/ui'
import type { TranslateFunction } from '@/utils/tf'

export function SettingsFormBody({ config, snapshot, saveResult, disabled, onRetry, tabFields, tab, setTab, form, onSubmit, t }: {
  config: SettingsConfig
  snapshot: SettingsSnapshot | undefined
  saveResult: SettingsSaveResult | null
  disabled: boolean
  onRetry: () => Promise<void>
  tabFields: SettingsField[]
  tab: string
  setTab: (tab: string) => void
  form: UseFormReturn<SettingsValues>
  onSubmit: (event?: React.BaseSyntheticEvent) => Promise<void>
  t: TranslateFunction
}) {
  const { register, control } = form
  const locks = config.features.managedLocks ? snapshot?.managedLocks ?? {} : {}
  const managed = Object.keys(locks).length > 0 || saveResult?.status === 'managed_locked'
  return (
        <>
          {config.rawSource?.backupNoticeKey ? <p className="mb-4 text-sm text-text-secondary" role="note">{t(config.rawSource.backupNoticeKey)}</p> : null}
          {managed && config.managedNotice ? (
            <div role="status" className="mb-4 rounded-xl border border-border-default p-4 text-sm">
              <strong>{t(config.managedNotice.titleKey)}</strong>
              <p>{t(config.managedNotice.descriptionKey)}</p>
              <Link to={config.managedNotice.href} className="underline">{t(config.managedNotice.actionKey)}</Link>
            </div>
          ) : null}
          {saveResult?.status === 'conflict' ? (
            <div role="alert" className="mb-4 text-sm">
              <p>{t('settingsRaw.conflictMessage')}</p>
              <Button onClick={onRetry}>{t('settingsRaw.reload')}</Button>
            </div>
          ) : null}
          {saveResult?.status === 'managed_locked' ? <p role="alert">{saveResult.message}</p> : null}
          <div className="mb-4 flex flex-wrap gap-2">
            {config.tabs.map((item) => <TabButton key={item.id} id={item.id} label={t(item.labelKey)} active={item.id === tab} onSelect={setTab} />)}
          </div>
          <form id="platform-settings-form" className="grid gap-4" onSubmit={onSubmit}>
            {tabFields.map((field) => (
              <SettingsFieldControl key={field.id} field={field} register={register} control={control} disabled={disabled || Boolean(locks[field.id])} disabledReason={locks[field.id] ? t(locks[field.id]) : undefined} t={t} />
            ))}
          </form>
        </>
  )
}
