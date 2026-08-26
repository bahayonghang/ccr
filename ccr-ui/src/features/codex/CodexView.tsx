import { useCallback, useMemo } from 'react'
import { Link } from 'react-router'
import { EmptyState, PageHeader, PageShell, SIcon, StatTile, buttonClass } from '@/ui'
import { CodexSubnav } from './CodexSubnav'
import { ReadinessCard } from './home/CodexHomeCards'
import { ActionConsole, ManagePanel, UsageStrip } from './home/CodexHomePanels'
import { panelCardClass } from './ui-classes'
import { useCodexDashboard } from './useCodexDashboard'
import { useCodexLocale } from './useCodexLocale'

export function CodexView() {
  const { t } = useCodexLocale()
  const dashboard = useCodexDashboard(t)
  const {
    overview,
    usageSummary,
    loading,
    error,
    overviewLoading,
    usageLoading,
    usageError,
    versionLabel,
    currentAccountLabel,
    currentProfileLabel,
    usageTotalRequests,
    usageTotalTokens,
    readinessItems,
    nextActions,
    primaryAction,
    compactInventory,
    formatDateTime,
    refresh,
  } = dashboard

  const visibleNextActions = useMemo(() => nextActions.slice(0, 2), [nextActions])
  const handleRefresh = useCallback(() => {
    void refresh(true)
  }, [refresh])

  const primaryClass = buttonClass({
    variant: primaryAction.tone === 'danger' ? 'danger' : 'primary',
  })

  return (
    <PageShell
      className="bg-bg-elevated"
      header={
        <PageHeader
          title={t('codex.overview.title')}
          eyebrow={t('codex.dashboard.header.eyebrow')}
          description={t('codex.dashboard.header.subtitle')}
          actions={
            <div className="flex flex-wrap gap-2">
              <button type="button" className={buttonClass({ variant: 'ghost' })} disabled={loading} onClick={handleRefresh}>
                <SIcon name="RefreshCw" size="w-4 h-4" className={loading ? 'animate-spin' : undefined} />
                {t('codex.dashboard.header.refresh')}
              </button>
              <Link to={primaryAction.to} className={primaryClass}>
                <span className="inline-flex items-center gap-2">
                  <SIcon name={primaryAction.icon} size="w-4 h-4" />
                  {primaryAction.title}
                </span>
              </Link>
              <Link to="/codex/auth" className={buttonClass({ variant: 'secondary' })}>
                <SIcon name="KeyRound" size="w-4 h-4" />
                {t('codex.dashboard.header.authConfig')}
              </Link>
              <Link to="/codex/profiles" className={buttonClass({ variant: 'secondary' })}>
                <SIcon name="Folders" size="w-4 h-4" />
                {t('codex.dashboard.header.profileConfig')}
              </Link>
            </div>
          }
        />
      }
      subnav={<CodexSubnav />}
    >
      <div className="mb-4 grid grid-cols-[repeat(auto-fit,minmax(10rem,1fr))] gap-4 rounded-xl border border-border-subtle bg-bg-surface p-4">
        <StatTile label={t('codex.dashboard.header.version')} value={versionLabel} />
        <StatTile label={t('codex.dashboard.header.profile')} value={currentProfileLabel} />
        <StatTile label={t('codex.dashboard.header.account')} value={currentAccountLabel} />
      </div>

      <section className={`${panelCardClass} mb-4`}>
        <div className="mb-4 flex flex-col gap-2 lg:flex-row lg:items-end lg:justify-between">
          <div>
            <p className="text-xs font-semibold uppercase tracking-[0.18em] text-[color:var(--stage-text-quiet)]">
              {t('codex.dashboard.readiness.eyebrow')}
            </p>
            <h2 className="mt-1 text-lg font-semibold tracking-tight text-[color:var(--stage-text-primary)]">
              {t('codex.dashboard.readiness.title')}
            </h2>
          </div>
          <p className="max-w-xl text-sm leading-6 text-[color:var(--stage-text-secondary)]">{t('codex.dashboard.readiness.subtitle')}</p>
        </div>
        {readinessItems.length > 0 ? (
          <div className="grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-4">
            {readinessItems.map((item) => (
              <ReadinessCard key={item.key} item={item} />
            ))}
          </div>
        ) : overviewLoading ? (
          <div className="grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-4">
            <div className="h-52 animate-pulse rounded-3xl bg-[var(--stage-surface-soft)]" />
            <div className="h-52 animate-pulse rounded-3xl bg-[var(--stage-surface-soft)]" />
            <div className="h-52 animate-pulse rounded-3xl bg-[var(--stage-surface-soft)]" />
            <div className="h-52 animate-pulse rounded-3xl bg-[var(--stage-surface-soft)]" />
          </div>
        ) : (
          <EmptyState
            icon="ShieldCheck"
            title={t('codex.dashboard.empty.readinessTitle')}
            description={t('codex.dashboard.empty.readinessDescription')}
            actionText={t('codex.dashboard.header.refresh')}
            actionIcon="RefreshCw"
            onAction={handleRefresh}
          />
        )}
      </section>

      <section className="grid grid-cols-1 gap-4 xl:grid-cols-5">
        <div className="xl:col-span-3">
          <ActionConsole
            t={t}
            error={error}
            overviewMissing={!overview}
            visibleNextActions={visibleNextActions}
            overviewLoading={overviewLoading}
            onRefresh={handleRefresh}
          />
          <UsageStrip
            t={t}
            usageSummary={usageSummary}
            overviewModel={overview?.config.model}
            usageLoading={usageLoading}
            usageTotalRequests={usageTotalRequests}
            usageTotalTokens={usageTotalTokens}
            formatDateTime={formatDateTime}
          />
          {usageError && !usageSummary ? (
            <div className="mt-4 rounded-3xl border border-accent-warning/20 bg-accent-warning/10 p-4 text-sm text-accent-warning">
              <p className="font-semibold">{t('codex.dashboard.error.usageTitle')}</p>
              <p className="mt-1 break-words">{usageError}</p>
            </div>
          ) : null}
        </div>
        <ManagePanel t={t} compactInventory={compactInventory} overviewLoading={overviewLoading} onRefresh={handleRefresh} />
      </section>
    </PageShell>
  )
}
