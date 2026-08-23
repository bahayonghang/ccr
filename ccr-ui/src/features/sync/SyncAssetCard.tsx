import { memo, useCallback } from 'react'
import type { SyncAssetInfo } from '@/types/syncSelection'
import type { TranslateFunction } from '@/utils/tf'
import { SIcon } from '@/ui'

function AssetTitleRow({ asset, t }: { asset: SyncAssetInfo; t: TranslateFunction }) {
  const kindLabel = asset.kind === 'directory' ? t('sync.assets.kindDirectory') : t('sync.assets.kindFile')
  return (
    <div className="sync-asset-card__title-row">
      <h4>{asset.name}</h4>
      {asset.sensitive ? <span className="sync-sensitive-chip">{t('sync.assets.sensitive')}</span> : null}
      {asset.encryptionState === 'v2_required' ? <span className="sync-encryption-chip">{t('sync.assets.encryptionV2')}</span> : null}
      <span className="sync-kind-chip">{kindLabel}</span>
    </div>
  )
}

function remoteStatus(asset: SyncAssetInfo, t: TranslateFunction) {
  if (asset.remoteExists === true) return { className: 'sync-status-chip sync-status-chip--ok', icon: 'CheckCircle', text: t('sync.assetStatus.remoteReady') }
  if (asset.remoteExists === false) return { className: 'sync-status-chip sync-status-chip--fail', icon: 'AlertCircle', text: t('sync.assetStatus.remoteMissing') }
  return { className: 'sync-status-chip sync-status-chip--neutral', icon: 'Cloud', text: t('sync.assetStatus.remoteUnknown') }
}

function AssetStatusRow({ asset, t }: { asset: SyncAssetInfo; t: TranslateFunction }) {
  const remote = remoteStatus(asset, t)
  return (
    <div className="sync-status-row">
      <span className={asset.localExists ? 'sync-status-chip sync-status-chip--ok' : 'sync-status-chip sync-status-chip--fail'}>
        <SIcon name={asset.localExists ? 'CheckCircle' : 'AlertCircle'} size="w-3.5 h-3.5" />
        {asset.localExists ? t('sync.assetStatus.localReady') : t('sync.assetStatus.localMissing')}
      </span>
      <span className={remote.className}>
        <SIcon name={remote.icon} size="w-3.5 h-3.5" />
        {remote.text}
      </span>
      {asset.canonicalName ? <span className="sync-status-chip sync-status-chip--neutral">{t('sync.assets.canonical', { name: asset.canonicalName })}</span> : null}
    </div>
  )
}

interface SyncAssetCardProps {
  asset: SyncAssetInfo
  busy: boolean
  busyLabel: string
  showForce: boolean
  t: TranslateFunction
  onPush: (asset: SyncAssetInfo) => void
  onPull: (asset: SyncAssetInfo) => void
  onSync: (asset: SyncAssetInfo) => void
  onForce: (asset: SyncAssetInfo) => void
}

export const SyncAssetCard = memo(function SyncAssetCard({
  asset,
  busy,
  busyLabel,
  showForce,
  t,
  onPush,
  onPull,
  onSync,
  onForce,
}: SyncAssetCardProps) {
  const handlePush = useCallback(() => {
    onPush(asset)
  }, [asset, onPush])
  const handlePull = useCallback(() => {
    onPull(asset)
  }, [asset, onPull])
  const handleSync = useCallback(() => {
    onSync(asset)
  }, [asset, onSync])
  const handleForce = useCallback(() => {
    onForce(asset)
  }, [asset, onForce])
  const localPath = asset.resolvedLocalPath || asset.localPath

  return (
    <div className={`sync-asset-card${asset.localExists ? '' : ' sync-asset-card--missing'}`}>
      <div className="sync-asset-card__body">
        <div className="sync-asset-card__icon">
          <SIcon name={asset.kind === 'directory' ? 'Folder' : 'FileText'} size="w-5 h-5" />
        </div>
        <div className="sync-asset-card__content">
          <AssetTitleRow asset={asset} t={t} />
          <p>{asset.description}</p>
          <dl className="sync-path-grid">
            <div>
              <dt>{t('sync.assets.localPath')}</dt>
              <dd title={`${asset.localPath} -> ${localPath}`}>{localPath}</dd>
            </div>
            <div>
              <dt>{t('sync.assets.remotePath')}</dt>
              <dd title={asset.remotePath}>{asset.remotePath}</dd>
            </div>
          </dl>
          <AssetStatusRow asset={asset} t={t} />
        </div>
      </div>
      <div className="sync-asset-card__actions">
        <button type="button" className="sync-action-button" disabled={busy || !asset.localExists} onClick={handlePush}>
          <SIcon name="Upload" size="w-4 h-4" />
          {t('sync.assetActions.push')}
        </button>
        <button type="button" className="sync-action-button" disabled={busy} onClick={handlePull}>
          <SIcon name="Download" size="w-4 h-4" />
          {t('sync.assetActions.pull')}
        </button>
        <button type="button" className="sync-action-button" disabled={busy} onClick={handleSync}>
          <SIcon name="RefreshCw" size="w-4 h-4" className={busy ? 'animate-spin' : ''} />
          {busyLabel || t('sync.assetActions.sync')}
        </button>
        {showForce ? (
          <button type="button" className="sync-action-button sync-action-button--force" disabled={busy} onClick={handleForce}>
            <SIcon name="Shield" size="w-4 h-4" />
            {t('sync.assetActions.forceRetry')}
          </button>
        ) : null}
      </div>
    </div>
  )
})
