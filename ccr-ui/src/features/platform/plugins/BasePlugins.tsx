import { useCallback, useMemo, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useForm } from 'react-hook-form'
import type { PluginDraft, PluginsConfig } from '@/configs/plugins'
import { NamedItemCard } from '@/features/platform/NamedItemCard'
import { SurfacePage } from '@/features/platform/SurfacePage'
import { defaultSurfaceT } from '@/features/platform/translate'
import { EmptyState } from '@/ui'
import type { TranslateFunction } from '@/utils/tf'

interface BasePluginsProps {
  config: PluginsConfig
  t?: TranslateFunction
}

export function BasePlugins({ config, t = defaultSurfaceT }: BasePluginsProps) {
  const [showForm, setShowForm] = useState(false)
  const query = useQuery({
    queryKey: ['platform-plugins', config.cacheKey],
    queryFn: config.list,
  })
  const form = useForm<PluginDraft>({ defaultValues: { id: '', name: '', version: '1.0.0', enabled: true } })

  const items = useMemo(
    () =>
      (query.data ?? []).map((item) => ({
        id: item.id,
        name: item.name,
        description: item.localPath ?? item.version,
        enabled: item.enabled,
      })),
    [query.data],
  )

  const openForm = useCallback(() => {
    setShowForm(true)
  }, [])
  const closeForm = useCallback(() => {
    setShowForm(false)
  }, [])
  const handleDelete = useCallback(
    (id: string) => {
      void config.remove(id).then(() => query.refetch())
    },
    [config, query],
  )
  const handleToggle = useCallback(
    (id: string) => {
      void config.toggle?.(id).then(() => query.refetch())
    },
    [config, query],
  )
  const onCreate = useMemo(
    () =>
      form.handleSubmit((values) => {
        void config.create(values).then(async () => {
          setShowForm(false)
          form.reset()
          await query.refetch()
        })
      }),
    [config, form, query],
  )

  if (query.isPending) {
    return <SurfacePage title={t(config.titleKey)} description={t(config.subtitleKey)} state="loading" />
  }

  return (
    <SurfacePage
      title={t(config.titleKey)}
      description={t(config.subtitleKey)}
      actions={
        <button type="button" className="rounded-lg bg-accent-primary px-4 py-2 text-sm text-[color:var(--color-accent-primary-contrast)]" onClick={openForm}>
          {t(`${config.i18nPrefix}.addPlugin`)}
        </button>
      }
    >
      {items.length === 0 ? (
        <EmptyState title={t(`${config.i18nPrefix}.emptyState`)} />
      ) : (
        <div className="grid gap-3">
          {items.map((item) => (
            <NamedItemCard
              key={item.id}
              item={item}
              onDelete={handleDelete}
              onToggle={config.features.toggle ? handleToggle : undefined}
              editLabel={t('common.edit')}
              deleteLabel={t('common.delete')}
              toggleLabel={t('common.toggle')}
            />
          ))}
        </div>
      )}
      {showForm ? (
        <form className="mt-4 grid gap-3 rounded-xl border border-border-default p-4" onSubmit={onCreate}>
          <input className="rounded-xl border border-border-default bg-bg-base px-3 py-2" placeholder={t('common.name')} {...form.register('name')} />
          {config.features.version ? (
            <input className="rounded-xl border border-border-default bg-bg-base px-3 py-2" placeholder="version" {...form.register('version')} />
          ) : null}
          {config.features.configJson ? (
            <textarea className="rounded-xl border border-border-default bg-bg-base px-3 py-2 font-mono text-xs" {...form.register('configJson')} />
          ) : null}
          <div className="flex gap-2">
            <button type="submit" className="rounded-lg bg-accent-primary px-4 py-2 text-sm text-[color:var(--color-accent-primary-contrast)]">
              {t('common.save')}
            </button>
            <button type="button" className="rounded-lg border border-border-default px-4 py-2 text-sm" onClick={closeForm}>
              {t('common.cancel')}
            </button>
          </div>
        </form>
      ) : null}
    </SurfacePage>
  )
}
