import { useCallback, useMemo, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useForm } from 'react-hook-form'
import type { AgentDraft, AgentsConfig } from '@/configs/agents'
import { NamedItemCard } from '@/features/platform/NamedItemCard'
import { SurfacePage } from '@/features/platform/SurfacePage'
import { useResolvedT } from '@/i18n'
import { EmptyState } from '@/ui'
import type { TranslateFunction } from '@/utils/tf'

interface BaseAgentsProps {
  config: AgentsConfig
  t?: TranslateFunction
}

export function BaseAgents({ config, t: tProp }: BaseAgentsProps) {
  const t = useResolvedT(tProp)
  const [showForm, setShowForm] = useState(false)
  const query = useQuery({
    queryKey: ['platform-agents', config.cacheKey],
    queryFn: config.list,
  })
  const form = useForm<AgentDraft>({ defaultValues: { name: '', description: '', model: '', body: '' } })

  const items = useMemo(
    () =>
      (query.data ?? []).map((item) => ({
        id: item.id,
        name: item.name,
        description: item.description,
        badge: item.folder,
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
          {t(`${config.i18nPrefix}.addAgent`)}
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
          <input className="rounded-xl border border-border-default bg-bg-base px-3 py-2" placeholder={t('common.description')} {...form.register('description')} />
          {config.features.tomlValidate ? (
            <textarea className="rounded-xl border border-border-default bg-bg-base px-3 py-2 font-mono text-xs" {...form.register('body')} />
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
