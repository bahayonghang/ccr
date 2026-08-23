import { useCallback } from 'react'
import { useQuery } from '@tanstack/react-query'
import type { AuthSessionConfig } from '@/configs/auth'
import { SurfacePage } from '@/features/platform/SurfacePage'
import { useResolvedT } from '@/i18n'
import type { TranslateFunction } from '@/utils/tf'

interface BaseAuthProps {
  config: AuthSessionConfig
  t?: TranslateFunction
}

export function BaseAuth({ config, t: tProp }: BaseAuthProps) {
  const t = useResolvedT(tProp)
  const probeQuery = useQuery({
    queryKey: ['platform-auth-probe', config.cacheKey],
    queryFn: config.probe ?? (async () => 'ok' as const),
  })
  const enabled = probeQuery.data === 'ok'
  const query = useQuery({
    queryKey: ['platform-auth', config.cacheKey],
    queryFn: config.load,
    enabled,
  })

  const handleRefresh = useCallback(() => {
    void query.refetch()
  }, [query])

  const handleOff = useCallback(async () => {
    const ok = await config.notify.confirm({
      title: t('auth.confirmOffTitle'),
      message: t(config.confirmOffKey),
      confirmText: t('auth.off'),
      cancelText: t('common.cancel'),
      type: 'danger',
    })
    if (!ok) return
    const result = await config.authOff()
    if (result.unsupported) return
    config.notify.success(result.changed ? t('auth.offSuccess') : t('auth.offUnchanged'))
    await query.refetch()
  }, [config, query, t])

  const onOffClick = useCallback(() => {
    void handleOff()
  }, [handleOff])

  if (probeQuery.data === 'unsupported_environment') {
    return (
      <SurfacePage
        title={t(config.titleKey)}
        description={t(config.subtitleKey)}
        state="runtime-unavailable"
        stateTitle={t('grok.dashboard.localOnly.title')}
        stateDescription={t('grok.dashboard.localOnly.description')}
      />
    )
  }

  if (query.isPending) {
    return <SurfacePage title={t(config.titleKey)} description={t(config.subtitleKey)} state="loading" />
  }

  const loggedIn = query.data?.loggedIn === true
  const canOff = query.data?.canAuthOff === true
  const statusKey = loggedIn ? `${config.i18nPrefix}.signedIn` : `${config.i18nPrefix}.signedOut`

  return (
    <SurfacePage
      title={t(config.titleKey)}
      description={t(config.subtitleKey)}
      actions={
        <button type="button" className="rounded-lg border border-border-default px-3 py-2 text-sm" onClick={handleRefresh}>
          {t('common.refresh')}
        </button>
      }
    >
      <section className="grid gap-3 rounded-2xl border border-border-default bg-bg-surface p-4" data-testid="platform-auth-session">
        {config.sessionFileLabelKey ? (
          <p className="text-xs font-semibold text-text-muted">{t(config.sessionFileLabelKey)}</p>
        ) : null}
        <p className="text-lg font-bold text-text-primary" data-testid="platform-auth-status">
          {t(statusKey)}
        </p>
        {query.data?.detail ? <p className="text-sm text-text-secondary">{query.data.detail}</p> : null}
        {canOff ? (
          <button type="button" className="w-fit rounded-lg border border-border-default px-3 py-2 text-sm" onClick={onOffClick} data-testid="platform-auth-off">
            {t('auth.off')}
          </button>
        ) : null}
      </section>
    </SurfacePage>
  )
}
