import { useCallback, useEffect, useRef } from 'react'
import type { SyncAssetInfo, SyncOperationOutput } from '@/types/syncSelection'
import type { TranslateFunction } from '@/utils/tf'
import { readPrefersReducedMotion } from '@/utils/reducedMotion'
import { AsyncStatePanel, PageHeader, PageShell, SIcon, buttonClass } from '@/ui'
import { SyncAssetCard } from './SyncAssetCard'
import { SyncInfoSidebar } from './SyncInfoSidebar'
import { SyncOperationOutputPanel } from './SyncOperationOutputPanel'
import { SyncPassphraseModal } from './SyncPassphraseModal'
import { useSyncPage } from './useSyncPage'
import type { SyncConnectionState } from './useSyncPage'
import './styles/sync-view.css'

const SERVICE_KEYS = ['nutstore', 'nextcloud', 'owncloud', 'any'] as const

type SyncPage = ReturnType<typeof useSyncPage>

function resolveConnectionChip(state: SyncConnectionState, hasStatus: boolean, t: TranslateFunction) {
  if (!hasStatus) return null
  if (state === 'connected') return { className: 'sync-connection-chip sync-connection-chip--ok', icon: 'CheckCircle', text: t('sync.webdav.connected') }
  if (state === 'unreachable') return { className: 'sync-connection-chip sync-connection-chip--danger', icon: 'AlertCircle', text: t('sync.webdav.unreachable') }
  if (state === 'unconfigured') return { className: 'sync-connection-chip sync-connection-chip--neutral', icon: 'Cloud', text: t('sync.webdav.notConfigured') }
  return { className: 'sync-connection-chip sync-connection-chip--neutral', icon: 'Cloud', text: t('sync.webdav.untested') }
}

function ScopeStrip({ t }: { t: TranslateFunction }) {
  const items = [
    { key: 'ccr', label: t('sync.assets.scopeCcrLabel'), value: t('sync.assets.scopeCcrValue') },
    { key: 'claude', label: t('sync.assets.scopeClaudeLabel'), value: t('sync.assets.scopeClaudeValue') },
    { key: 'codex', label: t('sync.assets.scopeCodexLabel'), value: t('sync.assets.scopeCodexValue') },
  ]
  return (
    <div className="sync-scope-strip">
      {items.map((item) => (
        <div key={item.key} className="sync-scope-strip__item">
          <span className="sync-scope-strip__label">{item.label}</span>
          <strong className="sync-scope-strip__value">{item.value}</strong>
        </div>
      ))}
    </div>
  )
}

interface GatingNoticeProps {
  connectionState: SyncConnectionState
  refreshing: boolean
  t: TranslateFunction
  onConfigure: () => void
  onRetest: () => void
}

function GatingNotice({ connectionState, refreshing, t, onConfigure, onRetest }: GatingNoticeProps) {
  if (connectionState === 'unconfigured') {
    return (
      <section className="sync-gating-card">
        <SIcon name="Cloud" size="w-5 h-5" className="sync-gating-card__icon" />
        <div className="sync-gating-card__body">
          <h3 className="sync-gating-card__title">{t('sync.gating.unconfiguredTitle')}</h3>
          <p className="sync-gating-card__desc">{t('sync.gating.unconfiguredDescription')}</p>
          <div className="sync-gating-card__actions">
            <button type="button" className={buttonClass({ variant: 'primary', size: 'sm' })} onClick={onConfigure}>
              {t('sync.gating.unconfiguredCta')}
            </button>
          </div>
        </div>
      </section>
    )
  }
  if (connectionState === 'unreachable') {
    return (
      <section className="sync-gating-card sync-gating-card--warning" role="alert">
        <SIcon name="AlertTriangle" size="w-5 h-5" className="sync-gating-card__icon" />
        <div className="sync-gating-card__body">
          <h3 className="sync-gating-card__title">{t('sync.gating.unreachableTitle')}</h3>
          <p className="sync-gating-card__desc">{t('sync.gating.unreachableDescription')}</p>
          <div className="sync-gating-card__actions">
            <button type="button" className={buttonClass({ variant: 'ghost', size: 'sm' })} disabled={refreshing} onClick={onRetest}>
              {t('sync.gating.retest')}
            </button>
          </div>
        </div>
      </section>
    )
  }
  return null
}

function AboutSyncDetails({ t }: { t: TranslateFunction }) {
  return (
    <details className="sync-about">
      <summary className="sync-about__summary">
        <SIcon name="ChevronRight" size="w-4 h-4" className="sync-about__chevron" />
        <span>{t('sync.about.title')}</span>
      </summary>
      <div className="sync-about__body">
        <section>
          <h3 className="sync-about__heading">{t('sync.assets.safetyTitle')}</h3>
          <ul className="sync-about__list">
            <li>{t('sync.assets.safetyAllowlist')}</li>
            <li>{t('sync.assets.safetyBackup')}</li>
            <li>{t('sync.assets.safetyMask')}</li>
          </ul>
        </section>
        <section>
          <h3 className="sync-about__heading">{t('sync.features.title')}</h3>
          <p className="sync-about__text">{t('sync.features.sensitiveMaskingDesc')}</p>
        </section>
        <section>
          <h3 className="sync-about__heading">{t('sync.supportedServices.title')}</h3>
          <ul className="sync-about__list sync-about__list--icons">
            {SERVICE_KEYS.map((key) => (
              <li key={key} className="sync-about__service">
                <SIcon name="CheckCircle" size="w-4 h-4" className="sync-about__service-icon" />
                <span>{t(`sync.supportedServices.${key}`)}</span>
              </li>
            ))}
          </ul>
        </section>
      </div>
    </details>
  )
}

interface SyncPageHeaderProps {
  page: SyncPage
  onRefresh: () => void
  onSyncAll: () => void
  onForceAll: () => void
}

function SyncPageHeader({ page, onRefresh, onSyncAll, onForceAll }: SyncPageHeaderProps) {
  const t = page.t
  const chip = resolveConnectionChip(page.connectionState, Boolean(page.syncStatus), t)
  const syncAllDisabled = page.globalOperating || page.assets.length === 0 || page.syncGated
  const gateTitle = page.syncGateReason ?? undefined
  return (
    <PageHeader
      title={t('sync.title')}
      description={t('sync.subtitle')}
      status={
        <>
          {chip ? (
            <span className={chip.className}>
              <SIcon name={chip.icon} size="w-3.5 h-3.5" />
              {chip.text}
            </span>
          ) : null}
          <span className="sync-badge">{t('sync.assets.badge')}</span>
        </>
      }
      actions={
        <>
          <button type="button" className={buttonClass({ variant: 'ghost', className: 'sync-hero-button' })} disabled={page.loading || page.refreshingAssets} onClick={onRefresh}>
            <SIcon name="RefreshCw" size="w-4 h-4" className={page.refreshingAssets ? 'animate-spin' : ''} />
            <span>{t('sync.assets.refresh')}</span>
          </button>
          <button type="button" className={buttonClass({ variant: 'primary', className: 'sync-hero-button' })} disabled={syncAllDisabled} title={gateTitle} onClick={onSyncAll}>
            <SIcon name="Sparkles" size="w-4 h-4" />
            <span>{page.globalOperating ? t('sync.assets.syncingAll') : t('sync.assets.syncAll')}</span>
          </button>
          {page.forceRetryAll ? (
            <button type="button" className={buttonClass({ variant: 'warning', className: 'sync-hero-button' })} disabled={syncAllDisabled} title={gateTitle} onClick={onForceAll}>
              <SIcon name="Shield" size="w-4 h-4" />
              <span>{t('sync.assets.forceRetryAll')}</span>
            </button>
          ) : null}
        </>
      }
    />
  )
}

interface AssetGroupsProps {
  page: SyncPage
  isBusy: (assetId: string) => boolean
  busyLabelOf: (assetId: string) => string
  groupLabel: (key: string) => string
  onPush: (asset: SyncAssetInfo) => void
  onPull: (asset: SyncAssetInfo) => void
  onSync: (asset: SyncAssetInfo) => void
  onForce: (asset: SyncAssetInfo) => void
}

function AssetGroups({ page, isBusy, busyLabelOf, groupLabel, onPush, onPull, onSync, onForce }: AssetGroupsProps) {
  const t = page.t
  return (
    <div className="sync-asset-groups">
      {page.assetGroups.map((group) => (
        <article key={group.key} className="sync-asset-group">
          <header className="sync-asset-group__header">
            <div>
              <p className="sync-eyebrow">{groupLabel(group.key)}</p>
              <h3>{group.title}</h3>
              <p>{group.description}</p>
            </div>
            <span className="sync-count-chip">{t('sync.assets.itemCount', { count: group.assets.length })}</span>
          </header>
          <div className="sync-asset-list">
            {group.assets.map((asset) => (
              <SyncAssetCard
                key={asset.id}
                asset={asset}
                busy={isBusy(asset.id)}
                busyLabel={busyLabelOf(asset.id)}
                showForce={page.forceRetry?.assetId === asset.id}
                t={t}
                disabledReason={page.syncGateReason ?? undefined}
                onPush={onPush}
                onPull={onPull}
                onSync={onSync}
                onForce={onForce}
              />
            ))}
          </div>
        </article>
      ))}
    </div>
  )
}

export function SyncView() {
  const page = useSyncPage()
  const outputAnchorRef = useRef<HTMLDivElement>(null)
  const lastOutputRef = useRef<SyncOperationOutput | null>(null)

  const handleRefresh = useCallback(() => {
    void page.refreshAll()
  }, [page])
  const handleSyncAll = useCallback(() => {
    page.requestRunAll(false)
  }, [page])
  const handleForceAll = useCallback(() => {
    page.requestRunAll(true)
  }, [page])
  const handleConfigure = useCallback(() => {
    page.openAccountDialog('add')
  }, [page])
  const handlePush = useCallback((asset: SyncAssetInfo) => {
    page.requestRunAsset(asset, 'push', false)
  }, [page])
  const handlePull = useCallback((asset: SyncAssetInfo) => {
    page.requestRunAsset(asset, 'pull', false)
  }, [page])
  const handleSync = useCallback((asset: SyncAssetInfo) => {
    page.requestRunAsset(asset, 'sync', false)
  }, [page])
  const handleForce = useCallback((asset: SyncAssetInfo) => {
    const retry = page.forceRetry
    if (!retry || retry.assetId !== asset.id) return
    page.requestRunAsset(asset, retry.operation, true)
  }, [page])
  const busyLabelOf = useCallback((assetId: string) => {
    if (page.busyAssetId !== assetId || !page.busyOperation) return ''
    return page.t(`sync.assetActions.${page.busyOperation}ing`)
  }, [page])
  const isBusy = useCallback((assetId: string) => page.globalOperating || page.busyAssetId === assetId, [page])
  const groupLabel = useCallback((key: string) => page.t(`sync.assetGroups.${key}.label`), [page])

  useEffect(() => {
    const output = page.operationOutput
    if (!output || output === lastOutputRef.current) return
    lastOutputRef.current = output
    outputAnchorRef.current?.scrollIntoView({
      behavior: readPrefersReducedMotion() ? 'auto' : 'smooth',
      block: 'nearest',
    })
  }, [page.operationOutput])

  return (
    <PageShell
      className="sync-page"
      header={<SyncPageHeader page={page} onRefresh={handleRefresh} onSyncAll={handleSyncAll} onForceAll={handleForceAll} />}
    >
      <ScopeStrip t={page.t} />

      {page.loading ? <AsyncStatePanel state="loading" title={page.t('common.loading')} /> : null}
      {!page.loading && page.error ? <AsyncStatePanel state="error" title={page.t('sync.loadFailed')} description={page.error} /> : null}
      {!page.loading && !page.error ? (
        <div className="sync-console-grid">
          <section className="sync-console-main">
            <GatingNotice connectionState={page.connectionState} refreshing={page.refreshingAssets} t={page.t} onConfigure={handleConfigure} onRetest={handleRefresh} />
            <div className="sync-console-intro">
              <div>
                <p className="sync-eyebrow">{page.t('sync.assets.eyebrow')}</p>
                <h2>{page.t('sync.assets.title')}</h2>
                <p>{page.t('sync.assets.description')}</p>
              </div>
              <div className="sync-console-intro__meta">
                <span>{page.t('sync.assets.total', { count: page.assets.length })}</span>
                <span>{page.t('sync.assets.sensitiveHint')}</span>
              </div>
            </div>
            <AssetGroups
              page={page}
              isBusy={isBusy}
              busyLabelOf={busyLabelOf}
              groupLabel={groupLabel}
              onPush={handlePush}
              onPull={handlePull}
              onSync={handleSync}
              onForce={handleForce}
            />
          </section>
          <aside className="sync-console-side">
            <SyncInfoSidebar
              syncStatus={page.syncStatus}
              onStatusRefresh={handleRefresh}
              accountDialogOpen={page.accountDialogOpen}
              accountDialogMode={page.accountDialogMode}
              onAccountDialogOpenChange={page.setAccountDialogOpen}
              onOpenAccountDialog={page.openAccountDialog}
            />
            <div ref={outputAnchorRef}>
              <SyncOperationOutputPanel output={page.operationOutput} onClear={page.clearOperationOutput} />
            </div>
            <AboutSyncDetails t={page.t} />
          </aside>
        </div>
      ) : null}

      <SyncPassphraseModal
        open={page.passphraseModalOpen}
        assetName={page.pending?.asset?.name}
        onOpenChange={page.setPassphraseModalOpen}
        onSubmit={page.submitSensitiveOperation}
      />
    </PageShell>
  )
}
