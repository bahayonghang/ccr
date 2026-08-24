import { useCallback, useEffect, useRef, useState } from 'react'
import { Link } from 'react-router'
import { surfaceNotify } from '@/configs/surfaceNotify'
import { EmptyState, PageHeader, PageShell, SIcon, StatTile } from '@/ui'
import { copyText } from '@/utils/clipboard'
import { GrokSubnav } from './GrokSubnav'
import { ActionRow, CommandRow, ManageRow, ReadinessCard } from './home/GrokHomeCards'
import { GROK_HOME_COMMANDS } from './home/grokHomeModel'
import { t } from './locale'
import { dangerBtnClass, ghostBtnClass, primaryBtnClass } from './ui-classes'
import { useGrokHome } from './useGrokHome'

export function GrokView() {
  const dashboard = useGrokHome(t)
  const {
    overview,
    loading,
    initialLoading,
    loadError,
    refreshError,
    localOnly,
    localOnlyEnvType,
    versionLabel,
    currentProfileLabel,
    authModeLabel,
    activationWarning,
    readinessItems,
    nextActions,
    primaryAction,
    managementItems,
    refresh,
  } = dashboard
  const [copiedCommand, setCopiedCommand] = useState<string | null>(null)
  const copyTimerRef = useRef<number | null>(null)

  const handleRefresh = useCallback(() => {
    void refresh(true)
  }, [refresh])

  useEffect(() => {
    void refresh(false)
  }, [refresh])

  useEffect(() => {
    if (refreshError) {
      surfaceNotify.error(`${t('grok.dashboard.error.refreshFailed')} ${refreshError}`)
    }
  }, [refreshError])

  useEffect(() => {
    return () => {
      if (copyTimerRef.current === null) return
      window.clearTimeout(copyTimerRef.current)
      copyTimerRef.current = null
    }
  }, [])

  const copyCommand = useCallback(async (command: string) => {
    if (!(await copyText(command))) {
      surfaceNotify.error(t('grok.dashboard.commands.copyFailed'))
      return
    }
    setCopiedCommand(command)
    if (copyTimerRef.current !== null) window.clearTimeout(copyTimerRef.current)
    copyTimerRef.current = window.setTimeout(() => {
      copyTimerRef.current = null
      setCopiedCommand((current) => (current === command ? null : current))
    }, 1600)
  }, [])

  const primaryClass = primaryAction.tone === 'danger' ? dangerBtnClass : primaryBtnClass
  const primaryInner = (
    <>
      <SIcon name={primaryAction.icon} size="w-4 h-4" />
      {primaryAction.title}
    </>
  )

  if (localOnly) {
    return (
      <PageShell
        className="bg-bg-elevated"
        header={
          <PageHeader
            title={t('grok.dashboard.localOnly.title')}
            eyebrow={t('grok.dashboard.header.eyebrow')}
            description={t('grok.dashboard.localOnly.description')}
            status={
              <span className="text-sm text-text-secondary">
                {t('grok.dashboard.localOnly.environment', {
                  env: localOnlyEnvType || t('grok.states.unknown'),
                })}
              </span>
            }
          />
        }
        subnav={<GrokSubnav />}
      />
    )
  }

  return (
    <PageShell
      className="bg-bg-elevated"
      header={
        <PageHeader
          title={t('grok.overview.title')}
          eyebrow={t('grok.dashboard.header.eyebrow')}
          description={t('grok.overview.subtitle')}
          actions={
            <div className="flex flex-wrap gap-2">
              <button type="button" className={ghostBtnClass} disabled={loading} onClick={handleRefresh}>
                <SIcon name="RefreshCw" size="w-4 h-4" className={loading ? 'animate-spin' : undefined} />
                {t('grok.dashboard.header.refresh')}
              </button>
              {overview && primaryAction.external ? (
                <a href={primaryAction.to} target="_blank" rel="noreferrer" className={primaryClass}>
                  {primaryInner}
                </a>
              ) : overview ? (
                <Link to={primaryAction.to} className={primaryClass}>
                  {primaryInner}
                </Link>
              ) : null}
            </div>
          }
        />
      }
      subnav={<GrokSubnav />}
    >
      {overview ? (
        <div className="mb-4 grid grid-cols-[repeat(auto-fit,minmax(10rem,1fr))] gap-4 rounded-xl border border-border-subtle bg-bg-surface p-4">
          <StatTile label={t('grok.dashboard.header.version')} value={versionLabel} />
          <StatTile label={t('grok.dashboard.header.profile')} value={currentProfileLabel} />
          <StatTile
            label={t('grok.dashboard.header.auth')}
            value={authModeLabel}
            hint={activationWarning?.label}
          />
        </div>
      ) : null}

      {initialLoading ? (
        <div className="py-6" aria-hidden="true">
          <div className="h-16 animate-pulse rounded-lg bg-[var(--stage-surface-soft)]" />
          <div className="mt-3 grid grid-cols-3 gap-3">
            <div className="h-48 animate-pulse rounded-lg bg-[var(--stage-surface-soft)]" />
            <div className="h-48 animate-pulse rounded-lg bg-[var(--stage-surface-soft)]" />
            <div className="h-48 animate-pulse rounded-lg bg-[var(--stage-surface-soft)]" />
          </div>
        </div>
      ) : loadError && !overview ? (
        <EmptyState
          icon="AlertCircle"
          title={t('grok.dashboard.empty.title')}
          description={loadError}
          actionText={t('grok.dashboard.header.refresh')}
          actionIcon="RefreshCw"
          onAction={handleRefresh}
        />
      ) : overview ? (
        <>
          <section className="border-b border-[color:var(--stage-border-soft)] py-6">
            <div className="mb-4 flex items-end justify-between gap-6">
              <div>
                <p className="m-0 text-sm font-semibold text-[color:var(--color-platform-grok)]">
                  {t('grok.dashboard.readiness.eyebrow')}
                </p>
                <h2 className="mt-1 text-lg font-semibold text-[color:var(--stage-text-primary)]">
                  {t('grok.dashboard.readiness.title')}
                </h2>
              </div>
              <p className="max-w-md text-sm text-[color:var(--stage-text-secondary)]">
                {t('grok.dashboard.readiness.subtitle')}
              </p>
            </div>
            <div className="grid gap-3 md:grid-cols-3">
              {readinessItems.map((item) => (
                <ReadinessCard key={item.key} item={item} />
              ))}
            </div>
          </section>

          <section className="grid gap-8 border-b border-[color:var(--stage-border-soft)] py-6 xl:grid-cols-[minmax(0,3fr)_minmax(18rem,2fr)]">
            <div>
              <p className="m-0 text-sm font-semibold text-[color:var(--color-platform-grok)]">
                {t('grok.dashboard.actions.eyebrow')}
              </p>
              <h2 className="mt-1 text-lg font-semibold text-[color:var(--stage-text-primary)]">
                {t('grok.dashboard.actions.title')}
              </h2>
              <p className="mb-3 text-sm text-[color:var(--stage-text-secondary)]">
                {t('grok.dashboard.actions.subtitle')}
              </p>
              <div className="border-t border-[color:var(--stage-border-soft)]">
                {nextActions.map((action, index) => (
                  <ActionRow key={action.key} action={action} index={index} />
                ))}
              </div>
            </div>
            <div>
              <p className="m-0 text-sm font-semibold text-[color:var(--color-platform-grok)]">
                {t('grok.dashboard.management.eyebrow')}
              </p>
              <h2 className="mt-1 text-lg font-semibold text-[color:var(--stage-text-primary)]">
                {t('grok.dashboard.management.title')}
              </h2>
              <p className="mb-3 text-sm text-[color:var(--stage-text-secondary)]">
                {t('grok.dashboard.management.subtitle')}
              </p>
              <div className="border-t border-[color:var(--stage-border-soft)]">
                {managementItems.map((item) => (
                  <ManageRow key={item.key} item={item} />
                ))}
              </div>
            </div>
          </section>

          <section className="py-6">
            <p className="m-0 text-sm font-semibold text-[color:var(--color-platform-grok)]">
              {t('grok.dashboard.commands.eyebrow')}
            </p>
            <h2 className="mb-3 mt-1 text-lg font-semibold text-[color:var(--stage-text-primary)]">
              {t('grok.dashboard.commands.title')}
            </h2>
            <div className="grid gap-2 md:grid-cols-2">
              {GROK_HOME_COMMANDS.map((command) => (
                <CommandRow
                  key={command}
                  command={command}
                  copied={copiedCommand === command}
                  copyLabel={t(
                    copiedCommand === command
                      ? 'grok.dashboard.commands.copied'
                      : 'grok.dashboard.commands.copy',
                  )}
                  onCopy={copyCommand}
                />
              ))}
            </div>
          </section>
        </>
      ) : null}
    </PageShell>
  )
}
