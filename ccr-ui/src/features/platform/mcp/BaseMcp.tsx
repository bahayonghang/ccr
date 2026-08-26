import { useCallback, useMemo, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useForm } from 'react-hook-form'
import type { McpConfig, McpDraft } from '@/configs/mcp'
import { NamedItemCard } from '@/features/platform/NamedItemCard'
import { SurfacePage } from '@/features/platform/SurfacePage'
import { useResolvedT } from '@/i18n'
import { EmptyState, Button } from '@/ui'
import type { TranslateFunction } from '@/utils/tf'

interface BaseMcpProps {
  config: McpConfig
  t?: TranslateFunction
}

const emptyDraft: McpDraft = { name: '', command: '', url: '' }

export function BaseMcp({ config, t: tProp }: BaseMcpProps) {
  const t = useResolvedT(tProp)
  const [showForm, setShowForm] = useState(false)
  const [transport, setTransport] = useState<'stdio' | 'http'>('stdio')
  const query = useQuery({
    queryKey: ['platform-mcp', config.cacheKey],
    queryFn: config.list,
  })
  const form = useForm<McpDraft>({ defaultValues: emptyDraft })

  const items = useMemo(
    () =>
      (query.data ?? []).map((server) => ({
        id: server.id,
        name: server.name,
        description: server.command ?? server.url,
        badge: server.transport ?? (server.url ? 'HTTP' : 'STDIO'),
      })),
    [query.data],
  )

  const stats = useMemo(() => {
    const servers = query.data ?? []
    const httpCount = servers.filter((server) => server.transport === 'http' || Boolean(server.url)).length
    return { total: servers.length, httpCount, live: servers.filter((server) => server.enabled !== false).length }
  }, [query.data])

  const openStdio = useCallback(() => {
    setTransport('stdio')
    setShowForm(true)
  }, [])
  const openHttp = useCallback(() => {
    setTransport('http')
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
  const onCreate = useMemo(
    () =>
      form.handleSubmit((values) => {
        const draft: McpDraft = { ...values, transport }
        void config.create(draft).then(async () => {
          setShowForm(false)
          form.reset(emptyDraft)
          await query.refetch()
        })
      }),
    [config, form, query, transport],
  )

  if (query.isPending) {
    return <SurfacePage title={t(config.titleKey)} description={t(config.subtitleKey)} state="loading" />
  }

  return (
    <SurfacePage
      title={t(config.titleKey)}
      description={t(config.subtitleKey)}
      actions={
        <div className="flex gap-2">
          {config.features.stdioCreate ? (
            <Button type="button" variant="ghost" className="rounded-lg px-3 py-2 text-sm" onClick={openStdio}>
              {t(`${config.i18nPrefix}.newStdio`)}
            </Button>
          ) : null}
          {config.features.httpCreate ? (
            <Button type="button" variant="ghost" className="rounded-lg px-3 py-2 text-sm" onClick={openHttp}>
              {t(`${config.i18nPrefix}.newHttp`)}
            </Button>
          ) : null}
        </div>
      }
    >
      {config.features.statsStrip ? (
        <div className="mb-4 grid gap-3 md:grid-cols-3">
          <div className="rounded-xl border border-border-default p-3 text-sm">{stats.total}</div>
          <div className="rounded-xl border border-border-default p-3 text-sm">{stats.httpCount}</div>
          <div className="rounded-xl border border-border-default p-3 text-sm">{stats.live}</div>
        </div>
      ) : null}
      {items.length === 0 ? (
        <EmptyState title={t(`${config.i18nPrefix}.emptyState`)} />
      ) : (
        <div className="grid gap-3">
          {items.map((item) => (
            <NamedItemCard
              key={item.id}
              item={item}
              onDelete={handleDelete}
              editLabel={t('common.edit')}
              deleteLabel={t('common.delete')}
            />
          ))}
        </div>
      )}
      {showForm ? (
        <form className="mt-4 grid gap-3 rounded-xl border border-border-default p-4" onSubmit={onCreate}>
          <input className="rounded-xl border border-border-default bg-bg-base px-3 py-2" placeholder={t('common.name')} {...form.register('name')} />
          {transport === 'http' ? (
            <input className="rounded-xl border border-border-default bg-bg-base px-3 py-2" placeholder="url" {...form.register('url')} />
          ) : (
            <input className="rounded-xl border border-border-default bg-bg-base px-3 py-2" placeholder={t('common.command')} {...form.register('command')} />
          )}
          {config.features.startupPolicy ? (
            <input className="rounded-xl border border-border-default bg-bg-base px-3 py-2" placeholder="startupTimeoutMs" {...form.register('startupTimeoutMs')} />
          ) : null}
          {config.features.authInjection ? (
            <input className="rounded-xl border border-border-default bg-bg-base px-3 py-2" placeholder="bearerTokenEnv" {...form.register('bearerTokenEnv')} />
          ) : null}
          <div className="flex gap-2">
            <Button type="submit" variant="primary" className="rounded-lg text-sm">
              {t('common.save')}
            </Button>
            <Button type="button" variant="ghost" className="rounded-lg text-sm" onClick={closeForm}>
              {t('common.cancel')}
            </Button>
          </div>
        </form>
      ) : null}
    </SurfacePage>
  )
}
