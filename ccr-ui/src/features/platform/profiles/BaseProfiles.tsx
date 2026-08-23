import { useCallback, useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'
import type { ProfilesConfig } from '@/configs/profiles'
import { SurfacePage } from '@/features/platform/SurfacePage'
import { defaultSurfaceT } from '@/features/platform/translate'
import { makeProfileRowDescriptor } from '@/features/platform/profiles/profiles-model'
import {
  ProfileListRow,
  ProfilesHeader,
  ProfilesSection,
} from '@/features/platform/profiles/shared'
import { EmptyState } from '@/ui'
import type { TranslateFunction } from '@/utils/tf'

interface BaseProfilesProps {
  config: ProfilesConfig
  t?: TranslateFunction
}

const noop = (): void => {}

export function BaseProfiles({ config, t = defaultSurfaceT }: BaseProfilesProps) {
  const probeQuery = useQuery({
    queryKey: ['platform-profiles-probe', config.cacheKey],
    queryFn: config.probe ?? (async () => 'ok' as const),
  })
  const enabled = probeQuery.data === 'ok'
  const query = useQuery({
    queryKey: ['platform-profiles', config.cacheKey],
    queryFn: config.list,
    enabled,
  })

  const descriptor = useMemo(() => makeProfileRowDescriptor(config, t), [config, t])
  const profiles = query.data?.profiles ?? []
  const current = query.data?.current ?? null

  const handleApply = useCallback(
    async (name: string) => {
      await config.apply(name)
      config.notify.success(t(`${config.i18nPrefix}.messages.applySuccess`))
      await query.refetch()
    },
    [config, query, t],
  )

  const handleDelete = useCallback(
    async (name: string) => {
      const ok = await config.notify.confirm({
        title: t(`${config.i18nPrefix}.confirm.deleteTitle`),
        message: t(`${config.i18nPrefix}.confirm.deleteMessage`),
        type: 'danger',
      })
      if (!ok) return
      await config.remove(name)
      await query.refetch()
    },
    [config, query, t],
  )

  const handleReload = useCallback(() => {
    void query.refetch()
  }, [query])

  const handleOff = useCallback(async () => {
    if (!config.profileOff) return
    const ok = await config.notify.confirm({
      title: t(`${config.i18nPrefix}.confirm.offTitle`),
      message: t(`${config.i18nPrefix}.confirm.offMessage`),
      type: 'danger',
    })
    if (!ok) return
    await config.profileOff()
    await query.refetch()
  }, [config, query, t])

  const handleExport = useCallback(() => {
    void config.exportAll?.()
  }, [config])

  const onApply = useCallback(
    (name: string) => {
      void handleApply(name)
    },
    [handleApply],
  )
  const onDelete = useCallback(
    (name: string) => {
      void handleDelete(name)
    },
    [handleDelete],
  )
  const onOffClick = useCallback(() => {
    void handleOff()
  }, [handleOff])

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

  if (query.isPending) {
    return <SurfacePage title={t(config.titleKey)} description={t(config.subtitleKey)} state="loading" />
  }

  return (
    <SurfacePage title={t(config.titleKey)} description={t(config.subtitleKey)}>
      <ProfilesHeader
        icon={config.icon}
        backTo={config.backTo}
        labels={{
          title: t(config.titleKey),
          subtitle: t(config.subtitleKey),
          back: t('common.back'),
          reload: t('common.refresh'),
          export: t('common.export'),
          add: t(`${config.i18nPrefix}.actions.add`),
        }}
        loading={query.isFetching}
        onAdd={noop}
        onExport={handleExport}
        onReload={handleReload}
        onOpenPalette={noop}
        onEditSource={noop}
      />
      {config.features.profileOff ? (
        <div className="mb-4">
          <button type="button" className="rounded-lg border border-border-default px-3 py-2 text-sm" onClick={onOffClick}>
            {t(`${config.i18nPrefix}.actions.off`)}
          </button>
        </div>
      ) : null}
      {profiles.length === 0 ? (
        <EmptyState title={t(`${config.i18nPrefix}.emptyTitle`)} />
      ) : (
        <ProfilesSection title={t(config.titleKey)} count={profiles.length}>
          {profiles.map((profile) => (
            <ProfileListRow
              key={profile.name}
              profile={profile}
              descriptor={descriptor}
              isCurrent={profile.name === current}
              onApply={onApply}
              onEdit={noop}
              onDelete={onDelete}
            />
          ))}
        </ProfilesSection>
      )}
    </SurfacePage>
  )
}
