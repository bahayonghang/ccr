import { useCallback, useMemo, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useForm } from 'react-hook-form'
import type { CommandDraft, CommandsConfig } from '@/configs/commands'
import { NamedItemCard } from '@/features/platform/NamedItemCard'
import { SurfacePage } from '@/features/platform/SurfacePage'
import { useResolvedT } from '@/i18n'
import { EmptyState } from '@/ui'
import type { TranslateFunction } from '@/utils/tf'

interface BaseCommandsProps {
  config: CommandsConfig
  t?: TranslateFunction
}

export function BaseCommands({ config, t: tProp }: BaseCommandsProps) {
  const t = useResolvedT(tProp)
  const [showForm, setShowForm] = useState(false)
  const query = useQuery({
    queryKey: ['platform-commands', config.cacheKey],
    queryFn: () => config.list(),
  })
  const form = useForm<CommandDraft>({ defaultValues: { name: '', description: '', template: '' } })

  const items = useMemo(
    () =>
      (query.data ?? []).map((item) => ({
        id: item.id,
        name: item.name,
        description: item.description,
        badge: item.enabled === false ? 'off' : undefined,
      })),
    [query.data],
  )

  const openForm = useCallback(() => {
    setShowForm(true)
  }, [])
  const closeForm = useCallback(() => {
    setShowForm(false)
  }, [])

  const handleRun = useCallback(
    (id: string) => {
      void config.execute?.(id)
    },
    [config],
  )
  const handleDelete = useCallback(
    (id: string) => {
      void config.remove?.(id).then(() => query.refetch())
    },
    [config, query],
  )
  const onCreate = useMemo(
    () =>
      form.handleSubmit((values) => {
        void config.create?.(values).then(async () => {
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
        config.features.templateCrud ? (
          <button type="button" className="rounded-lg bg-accent-primary px-4 py-2 text-sm text-[color:var(--color-accent-primary-contrast)]" onClick={openForm}>
            {t('common.add')}
          </button>
        ) : null
      }
    >
      {config.features.builtinOverrideHint ? (
        <p className="mb-4 text-sm text-text-secondary">{t(`${config.i18nPrefix}.builtinHint`)}</p>
      ) : null}
      {items.length === 0 ? (
        <EmptyState title={t(`${config.i18nPrefix}.emptyTitle`)} />
      ) : (
        <div className="grid gap-3">
          {items.map((item) => (
            <NamedItemCard
              key={item.id}
              item={item}
              onRun={config.features.execute ? handleRun : undefined}
              onDelete={config.features.templateCrud ? handleDelete : undefined}
              editLabel={t('common.edit')}
              deleteLabel={t('common.delete')}
              runLabel={t('common.run')}
            />
          ))}
        </div>
      )}
      {showForm ? (
        <form className="mt-4 grid gap-3 rounded-xl border border-border-default p-4" onSubmit={onCreate}>
          <input className="rounded-xl border border-border-default bg-bg-base px-3 py-2" placeholder={t('common.name')} {...form.register('name')} />
          <textarea className="rounded-xl border border-border-default bg-bg-base px-3 py-2" placeholder={t('common.description')} {...form.register('template')} />
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
