import { lazy, Suspense, useCallback, useMemo, useState } from 'react'
import { useForm, useWatch } from 'react-hook-form'
import type { SettingsConfig, SettingsValues } from '@/configs/settings'
import { SettingsUnavailableError } from '@/configs/settings-types'
import { SurfacePage } from '@/features/platform/SurfacePage'
import { useResolvedT } from '@/i18n'
import { fieldsForTab, invalidSettingsField, settingsDefaultValues } from '@/features/platform/settings-model'
import { SettingsFormBody } from './SettingsFormBody'
import { useSettingsSession } from './useSettingsSession'
import { SettingsSessionNotice } from './SettingsSessionNotice'
import { Button } from '@/ui'
import type { TranslateFunction } from '@/utils/tf'

const SettingsSource = lazy(() => import('./SettingsSource').then((module) => ({ default: module.SettingsSource })))

interface BaseSettingsProps {
  config: SettingsConfig
  t?: TranslateFunction
}

export function BaseSettings({ config, t: tProp }: BaseSettingsProps) {
  const t = useResolvedT(tProp)
  const [tab, setTab] = useState(config.tabs[0]?.id ?? 'model')
  const [sourceMode, setSourceMode] = useState(false)
  const [sourceDirty, setSourceDirty] = useState(false)
  const form = useForm<SettingsValues>({ defaultValues: settingsDefaultValues(config) })
  const { handleSubmit, reset, control, formState } = form
  const values = useWatch({ control }) as SettingsValues
  const session = useSettingsSession(config, form, t)
  const dirtyKeys = Object.keys(formState.dirtyFields)
  const invalid = invalidSettingsField(config, values, dirtyKeys)
  const tabFields = useMemo(() => fieldsForTab(config, tab), [config, tab])
  const onSubmit = useMemo(() => handleSubmit(session.save), [handleSubmit, session.save])

  const confirmDiscard = useCallback(async () => !(formState.isDirty || sourceDirty) || config.notify.confirm({
    title: t('settingsRaw.discardTitle'), message: t('settingsRaw.discardMessage'),
    confirmText: t('settingsRaw.discard'), cancelText: t('common.cancel'), type: 'warning',
  }), [config.notify, formState.isDirty, sourceDirty, t])
  const onRetry = useCallback(async () => {
    if (!(await confirmDiscard())) return
    if (await session.reload()) setSourceMode(false)
  }, [confirmDiscard, session])
  const openSource = useCallback(async () => {
    if (!(await confirmDiscard()) || !session.isCurrent() || session.busy) return
    if (session.session) reset(session.session.snapshot.values)
    setSourceMode(true)
  }, [confirmDiscard, reset, session])
  const closeSource = useCallback(() => setSourceMode(false), [])
  const sourceSaved = useCallback(() => {
    setSourceMode(false)
    void session.reload()
  }, [session])

  const error = session.environmentQuery.error ?? session.probeQuery.error ?? session.valuesQuery.error
  if (!session.session) return <SettingsInitialState config={config} probe={session.probeQuery.data} error={error} onRetry={onRetry} t={t} />
  const disabled = session.frozen || session.busy
  const saveDisabled = isSaveDisabled({ disabled, dirty: formState.isDirty, invalid, session, error })

  return (
    <SurfacePage title={t(config.titleKey)} description={t(config.subtitleKey)} actions={sourceMode ? undefined : (
      <>
        {config.features.rawSource && config.rawSource ? <Button disabled={disabled} onClick={openSource}>{t('settingsRaw.sourceTab')}</Button> : null}
        <Button type="submit" form="platform-settings-form" variant="primary" disabled={saveDisabled}>
          {t(config.i18nPrefix + '.save')}
        </Button>
      </>
    )}>
      <SettingsSessionNotice frozen={session.frozen} changed={session.serverChanged} reloadRequired={session.reloadRequired} error={error} actionError={session.actionError} busy={session.busy} onReturn={session.returnToEnvironment} onReload={onRetry} t={t} />
      {sourceMode && config.rawSource ? (
        <Suspense fallback={<p role="status">{t('settingsRaw.editorLoading')}</p>}>
          <SettingsSource source={config.rawSource} environment={session.session.environment} disabled={disabled} isCurrent={session.isCurrent} assertCurrent={session.assertCurrent} onDirtyChange={setSourceDirty} onClose={closeSource} onSaved={sourceSaved} />
        </Suspense>
      ) : (
        <SettingsFormBody config={config} snapshot={session.session.snapshot} saveResult={session.saveResult} disabled={disabled} onRetry={onRetry} tabFields={tabFields} tab={tab} setTab={setTab} form={form} onSubmit={onSubmit} t={t} />
      )}
    </SurfacePage>
  )
}

function isSaveDisabled({ disabled, dirty, invalid, session, error }: {
  disabled: boolean; dirty: boolean; invalid: string | null
  session: ReturnType<typeof useSettingsSession>; error: unknown
}) {
  return disabled || !dirty || Boolean(invalid) || session.serverChanged
    || session.reloadRequired || !session.snapshotReady || Boolean(error) || session.saveResult?.status === 'conflict' || session.saveResult?.status === 'unsupported_environment'
}

function SettingsInitialState({ config, probe, error, onRetry, t }: {
  config: SettingsConfig; probe: unknown; error: unknown
  onRetry: () => Promise<void>; t: TranslateFunction
}) {
  const unavailable = probe === 'unsupported_environment' || error instanceof SettingsUnavailableError
  if (unavailable) return <SurfacePage title={t(config.titleKey)} description={t(config.subtitleKey)} state="runtime-unavailable" stateTitle={t('settingsRaw.unsupportedEnvironment')} />
  if (error) return <SurfacePage title={t(config.titleKey)} description={t(config.subtitleKey)} state="error" retryLabel={t('common.retry')} stateDescription={error instanceof Error ? error.message : String(error)} onRetry={onRetry} />
  return <SurfacePage title={t(config.titleKey)} description={t(config.subtitleKey)} state="loading" />
}
