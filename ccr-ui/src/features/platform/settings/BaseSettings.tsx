import { useCallback, useEffect, useMemo, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useForm } from 'react-hook-form'
import type { SettingsConfig, SettingsValues } from '@/configs/settings'
import { SurfacePage } from '@/features/platform/SurfacePage'
import { TabButton } from '@/features/platform/TabButton'
import { defaultSurfaceT } from '@/features/platform/translate'
import {
  fieldsForTab,
  saveSettingsValues,
  settingsDefaultValues,
} from '@/features/platform/settings-model'
import { SettingsFieldControl } from '@/features/platform/settings/SettingsFieldControl'
import type { TranslateFunction } from '@/utils/tf'

interface BaseSettingsProps {
  config: SettingsConfig
  t?: TranslateFunction
}

export function BaseSettings({ config, t = defaultSurfaceT }: BaseSettingsProps) {
  const [tab, setTab] = useState(config.tabs[0]?.id ?? 'model')
  const probeQuery = useQuery({
    queryKey: ['platform-settings-probe', config.cacheKey],
    queryFn: config.probe ?? (async () => 'ok' as const),
  })
  const enabled = probeQuery.data === 'ok'
  const valuesQuery = useQuery({
    queryKey: ['platform-settings', config.cacheKey],
    queryFn: config.load,
    enabled,
  })

  const form = useForm<SettingsValues>({ defaultValues: settingsDefaultValues(config) })
  const { register, handleSubmit, reset } = form

  useEffect(() => {
    if (valuesQuery.data) reset(valuesQuery.data)
  }, [reset, valuesQuery.data])

  const tabFields = useMemo(() => fieldsForTab(config, tab), [config, tab])

  const onValid = useCallback(
    async (values: SettingsValues) => {
      const dirtyKeys = Object.keys(form.formState.dirtyFields)
      try {
        await saveSettingsValues(config, values, dirtyKeys)
        config.notify.success(t(`${config.i18nPrefix}.messages.saveSuccess`))
        await valuesQuery.refetch()
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error)
        config.notify.error(message)
      }
    },
    [config, form.formState.dirtyFields, t, valuesQuery],
  )

  const onSubmit = useMemo(() => handleSubmit(onValid), [handleSubmit, onValid])
  const onRetry = useCallback(() => {
    void valuesQuery.refetch()
  }, [valuesQuery])

  if (probeQuery.data === 'unsupported_environment') {
    return (
      <SurfacePage
        title={t(config.titleKey)}
        description={t(config.subtitleKey)}
        state="runtime-unavailable"
        stateTitle={t('settingsRaw.unsupportedEnvironment')}
      />
    )
  }

  if (valuesQuery.isPending) {
    return <SurfacePage title={t(config.titleKey)} description={t(config.subtitleKey)} state="loading" />
  }

  if (valuesQuery.isError) {
    return (
      <SurfacePage
        title={t(config.titleKey)}
        description={t(config.subtitleKey)}
        state="error"
        stateDescription={valuesQuery.error instanceof Error ? valuesQuery.error.message : undefined}
        onRetry={onRetry}
      />
    )
  }

  return (
    <SurfacePage
      title={t(config.titleKey)}
      description={t(config.subtitleKey)}
      actions={
        <button type="submit" form="platform-settings-form" className="rounded-lg bg-accent-primary px-4 py-2 text-sm text-[color:var(--color-accent-primary-contrast)]">
          {t(`${config.i18nPrefix}.save`)}
        </button>
      }
    >
      <div className="mb-4 flex flex-wrap gap-2">
        {config.tabs.map((item) => (
          <TabButton
            key={item.id}
            id={item.id}
            label={t(item.labelKey)}
            active={item.id === tab}
            onSelect={setTab}
          />
        ))}
      </div>
      <form id="platform-settings-form" className="grid gap-4" onSubmit={onSubmit}>
        {tabFields.map((field) => (
          <SettingsFieldControl key={field.id} field={field} register={register} t={t} />
        ))}
      </form>
    </SurfacePage>
  )
}
