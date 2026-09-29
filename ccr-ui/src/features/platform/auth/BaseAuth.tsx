import { useCallback, useRef, useState } from 'react'
import { useQuery, useQueryClient, type UseQueryResult } from '@tanstack/react-query'
import type { AuthSession, AuthSessionConfig, AuthSessionState } from '@/configs/auth'
import { SurfacePage } from '@/features/platform/SurfacePage'
import { useResolvedT } from '@/i18n'
import type { TranslateFunction } from '@/utils/tf'

interface BaseAuthProps {
  config: AuthSessionConfig
  t?: TranslateFunction
}

export function BaseAuth({ config, t: tProp }: BaseAuthProps) {
  const t = useResolvedT(tProp)
  const client = useQueryClient()
  const offPending = useRef(false)
  const [offBusy, setOffBusy] = useState(false)
  const [offError, setOffError] = useState<string | null>(null)
  const probeQuery = useQuery({
    queryKey: ['platform-auth-probe', config.cacheKey],
    queryFn: config.probe ?? (async () => 'ok' as const),
    retry: false,
  })
  const enabled = probeQuery.data === 'ok' && !probeQuery.isError
  const query = useQuery({
    queryKey: ['platform-auth', config.cacheKey],
    queryFn: config.load,
    enabled,
    retry: false,
  })

  const handleRefresh = useCallback(async () => {
    setOffError(null)
    if (probeQuery.isError || probeQuery.data !== 'ok') {
      await client.invalidateQueries({ queryKey: ['platform-auth', config.cacheKey], exact: true, refetchType: 'none' })
      await probeQuery.refetch()
      return
    }
    await query.refetch()
  }, [client, config.cacheKey, probeQuery, query])

  const handleOff = useCallback(async () => {
    if (offPending.current) return
    offPending.current = true
    setOffBusy(true)
    setOffError(null)
    try {
      const ok = await config.notify.confirm({
        title: t('auth.confirmOffTitle'),
        message: t(config.confirmOffKey),
        confirmText: t('auth.off'),
        cancelText: t('common.cancel'),
        type: 'danger',
      })
      if (!ok) return
      const result = await config.authOff()
      if (result.unsupported) {
        client.setQueryData(['platform-auth', config.cacheKey], { status: 'unsupported_environment' })
        return
      }
      config.notify.success(result.changed ? t('auth.offSuccess') : t('auth.offUnchanged'))
      await query.refetch()
    } catch {
      const message = t('auth.offFailed')
      setOffError(message)
      config.notify.error(message)
    } finally {
      offPending.current = false
      setOffBusy(false)
    }
  }, [client, config, query, t])

  const onOffClick = useCallback(() => {
    void handleOff()
  }, [handleOff])

  const { session, error, unsupported, loading } = authView(probeQuery, query)
  if (unsupported) {
    return (
      <SurfacePage
        title={t(config.titleKey)}
        description={t(config.subtitleKey)}
        state="runtime-unavailable"
        stateTitle={t('grok.dashboard.localOnly.title')}
        stateDescription={t('grok.dashboard.localOnly.description')}
        actions={<button type="button" onClick={handleRefresh}>{t('common.retry')}</button>}
      />
    )
  }

  if (error && !session) {
    return <SurfacePage title={t(config.titleKey)} description={t(config.subtitleKey)} state="error" retryLabel={t('common.retry')} stateDescription={error instanceof Error ? error.message : String(error)} onRetry={handleRefresh} />
  }

  if (loading) {
    return <SurfacePage title={t(config.titleKey)} description={t(config.subtitleKey)} state="loading" />
  }

  return (
    <SurfacePage
      title={t(config.titleKey)}
      description={t(config.subtitleKey)}
      actions={
        <button type="button" className="rounded-lg border border-border-default px-3 py-2 text-sm" disabled={offBusy || probeQuery.isFetching || query.isFetching} onClick={handleRefresh}>
          {t('common.refresh')}
        </button>
      }
    >
      <AuthSessionPanel config={config} session={session} error={error} offError={offError} disabled={offBusy || Boolean(error) || query.isFetching} onRefresh={handleRefresh} onOff={onOffClick} t={t} />
    </SurfacePage>
  )
}

function authView(probe: UseQueryResult<import('@/configs/probeLocal').EnvironmentProbe>, query: UseQueryResult<AuthSessionState>) {
  const error = probe.error ?? query.error
  return {
    session: query.data && 'loggedIn' in query.data ? query.data : undefined,
    error,
    unsupported: !error && (probe.data === 'unsupported_environment' || Boolean(query.data && 'status' in query.data)),
    loading: probe.isPending || query.isPending,
  }
}

function AuthSessionPanel({ config, session, error, offError, disabled, onRefresh, onOff, t }: {
  config: AuthSessionConfig
  session: AuthSession | undefined
  error: unknown
  offError: string | null
  disabled: boolean
  onRefresh: () => Promise<void>
  onOff: () => void
  t: TranslateFunction
}) {
  const statusKey = session?.loggedIn ? config.i18nPrefix + '.signedIn' : config.i18nPrefix + '.signedOut'
  return (
      <section className="grid gap-3 rounded-2xl border border-border-default bg-bg-surface p-4" data-testid="platform-auth-session">
        {error ? <div role="alert"><p>{t('auth.stale')}</p><p>{error instanceof Error ? error.message : String(error)}</p><button type="button" onClick={onRefresh}>{t('common.retry')}</button></div> : null}
        {offError ? <p role="alert">{offError}</p> : null}
        {config.sessionFileLabelKey ? (
          <p className="text-xs font-semibold text-text-muted">{t(config.sessionFileLabelKey)}</p>
        ) : null}
        <p className="text-lg font-bold text-text-primary" data-testid="platform-auth-status">
          {t(statusKey)}
        </p>
        {session?.detail ? <p className="text-sm text-text-secondary">{session.detail}</p> : null}
        {session?.canAuthOff ? (
          <button type="button" className="w-fit rounded-lg border border-border-default px-3 py-2 text-sm" disabled={disabled} onClick={onOff} data-testid="platform-auth-off">
            {t('auth.off')}
          </button>
        ) : null}
      </section>
  )
}
