import { useCallback } from 'react'
import { Link } from 'react-router'
import type { SyncAssetInfo } from '@/types/syncSelection'
import { AsyncStatePanel, PageHeader, PageShell, SIcon, buttonClass } from '@/ui'
import { SyncAssetCard } from './SyncAssetCard'
import { SyncInfoSidebar } from './SyncInfoSidebar'
import { SyncOperationOutputPanel } from './SyncOperationOutputPanel'
import { SyncPassphraseModal } from './SyncPassphraseModal'
import { useSyncPage } from './useSyncPage'
import './styles/sync-view.css'

export function SyncView() {
  const page = useSyncPage()
  const handleRefresh = useCallback(() => {
    void page.refreshAll()
  }, [page])
  const handleSyncAll = useCallback(() => {
    page.requestRunAll(false)
  }, [page])
  const handleForceAll = useCallback(() => {
    page.requestRunAll(true)
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

  return (
    <PageShell
      className="sync-page"
      header={
        <PageHeader
          title={page.t('sync.title')}
          description={page.t('sync.subtitle')}
          status={<span className="sync-badge">{page.t('sync.assets.badge')}</span>}
          actions={
            <>
              <button type="button" className={buttonClass({ variant: 'ghost', className: 'sync-hero-button' })} disabled={page.loading || page.refreshingAssets} onClick={handleRefresh}>
                <SIcon name="RefreshCw" size="w-4 h-4" className={page.refreshingAssets ? 'animate-spin' : ''} />
                <span>{page.t('sync.assets.refresh')}</span>
              </button>
              <button type="button" className={buttonClass({ variant: 'primary', className: 'sync-hero-button' })} disabled={page.globalOperating || page.assets.length === 0} onClick={handleSyncAll}>
                <SIcon name="Sparkles" size="w-4 h-4" />
                <span>{page.globalOperating ? page.t('sync.assets.syncingAll') : page.t('sync.assets.syncAll')}</span>
              </button>
              {page.forceRetryAll ? (
                <button type="button" className={buttonClass({ variant: 'warning', className: 'sync-hero-button' })} disabled={page.globalOperating || page.assets.length === 0} onClick={handleForceAll}>
                  <SIcon name="Shield" size="w-4 h-4" />
                  <span>{page.t('sync.assets.forceRetryAll')}</span>
                </button>
              ) : null}
              <Link to="/" className="sync-back-link">
                <SIcon name="Home" size="w-4 h-4" />
                <span>{page.t('sync.backHome')}</span>
              </Link>
            </>
          }
        />
      }
    >
      <div className="sync-scope-strip">
        {[
          { key: 'ccr', label: page.t('sync.assets.scopeCcrLabel'), value: page.t('sync.assets.scopeCcrValue') },
          { key: 'claude', label: page.t('sync.assets.scopeClaudeLabel'), value: page.t('sync.assets.scopeClaudeValue') },
          { key: 'codex', label: page.t('sync.assets.scopeCodexLabel'), value: page.t('sync.assets.scopeCodexValue') },
        ].map((item) => (
          <div key={item.key} className="sync-scope-strip__item">
            <span className="sync-scope-strip__label">{item.label}</span>
            <strong>{item.value}</strong>
          </div>
        ))}
      </div>

      {page.loading ? <AsyncStatePanel state="loading" title={page.t('common.loading')} /> : null}
      {!page.loading && page.error ? <AsyncStatePanel state="error" title={page.t('sync.loadFailed')} description={page.error} /> : null}
      {!page.loading && !page.error ? (
        <div className="sync-console-grid">
          <section className="sync-console-main">
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
            <div className="sync-asset-groups">
              {page.assetGroups.map((group) => (
                <article key={group.key} className="sync-asset-group">
                  <header className="sync-asset-group__header">
                    <div>
                      <p className="sync-eyebrow">{groupLabel(group.key)}</p>
                      <h3>{group.title}</h3>
                      <p>{group.description}</p>
                    </div>
                    <span className="sync-count-chip">{page.t('sync.assets.itemCount', { count: group.assets.length })}</span>
                  </header>
                  <div className="sync-asset-list">
                    {group.assets.map((asset) => (
                      <SyncAssetCard
                        key={asset.id}
                        asset={asset}
                        busy={isBusy(asset.id)}
                        busyLabel={busyLabelOf(asset.id)}
                        showForce={page.forceRetry?.assetId === asset.id}
                        t={page.t}
                        onPush={handlePush}
                        onPull={handlePull}
                        onSync={handleSync}
                        onForce={handleForce}
                      />
                    ))}
                  </div>
                </article>
              ))}
            </div>
          </section>
          <aside className="sync-console-side">
            <SyncInfoSidebar syncStatus={page.syncStatus} onStatusRefresh={handleRefresh} />
            <section className="sync-safety-card">
              <p className="sync-eyebrow">{page.t('sync.assets.safetyTitle')}</p>
              <ul>
                <li>{page.t('sync.assets.safetyAllowlist')}</li>
                <li>{page.t('sync.assets.safetyBackup')}</li>
                <li>{page.t('sync.assets.safetyMask')}</li>
              </ul>
            </section>
            <SyncOperationOutputPanel output={page.operationOutput} onClear={page.clearOperationOutput} />
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
