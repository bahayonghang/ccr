import { useCallback, useEffect, useMemo, useState } from 'react'
import { getSyncStatus, listSyncAssets, pullSyncAsset, pushSyncAsset, syncAllAssets, syncSingleAsset } from '@/api'
import type {
  SyncAssetGroup,
  SyncAssetInfo,
  SyncAssetOperation,
  SyncAssetOperationOptions,
  SyncOperationOutput,
  SyncStatusView,
} from '@/types/syncSelection'
import { logger } from '@/utils/logger'
import { useSyncT } from './locale'
import { toErrorMessage } from './sync-mask'
import { buildErrorOutput, buildOperationOutput } from './sync-output'

const GROUP_ORDER = ['ccr', 'claude', 'codex']

export type SyncConnectionState = 'unconfigured' | 'unreachable' | 'connected' | 'unknown'

export function useSyncPage() {
  const t = useSyncT()
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')
  const [syncStatus, setSyncStatus] = useState<SyncStatusView | null>(null)
  const [assets, setAssets] = useState<SyncAssetInfo[]>([])
  const [operationOutput, setOperationOutput] = useState<SyncOperationOutput | null>(null)
  const [refreshingAssets, setRefreshingAssets] = useState(false)
  const [globalOperating, setGlobalOperating] = useState(false)
  const [busyAssetId, setBusyAssetId] = useState<string | null>(null)
  const [busyOperation, setBusyOperation] = useState<SyncAssetOperation | null>(null)
  const [forceRetry, setForceRetry] = useState<{ assetId: string; operation: SyncAssetOperation } | null>(null)
  const [forceRetryAll, setForceRetryAll] = useState(false)
  const [passphraseModalOpen, setPassphraseModalOpen] = useState(false)
  const [pending, setPending] = useState<{ asset?: SyncAssetInfo; operation?: SyncAssetOperation; force: boolean; all: boolean } | null>(null)
  const [accountDialogOpen, setAccountDialogOpen] = useState(false)
  const [accountDialogMode, setAccountDialogMode] = useState<'add' | 'edit'>('add')

  const openAccountDialog = useCallback((mode: 'add' | 'edit') => {
    setAccountDialogMode(mode)
    setAccountDialogOpen(true)
  }, [])

  const connectionState = useMemo<SyncConnectionState>(() => {
    if (!syncStatus) return 'unknown'
    if (!syncStatus.configured) return 'unconfigured'
    if (syncStatus.remote_accessible === true) return 'connected'
    if (syncStatus.remote_accessible === false) return 'unreachable'
    return 'unknown'
  }, [syncStatus])
  // 未配置/不可达时门控同步操作；未测试（unknown）不门控
  const syncGated = connectionState === 'unconfigured' || connectionState === 'unreachable'
  const syncGateReason = !syncGated
    ? null
    : connectionState === 'unconfigured'
      ? t('sync.gating.disabledUnconfigured')
      : t('sync.gating.disabledUnreachable')

  const assetGroups = useMemo<SyncAssetGroup[]>(
    () => GROUP_ORDER.map((key) => ({
      key,
      title: t(`sync.assetGroups.${key}.title`),
      description: t(`sync.assetGroups.${key}.description`),
      assets: assets.filter((asset) => asset.group === key),
    })).filter((group) => group.assets.length > 0),
    [assets, t],
  )

  const fetchSyncStatus = useCallback(async () => {
    try {
      setSyncStatus(await getSyncStatus())
    } catch (err) {
      logger.error('Failed to fetch sync status:', err)
    }
  }, [])

  const fetchAssets = useCallback(async () => {
    const next = await listSyncAssets()
    setAssets(Array.isArray(next) ? next : [])
  }, [])

  const refreshAll = useCallback(async () => {
    setRefreshingAssets(true)
    try {
      await Promise.all([fetchSyncStatus(), fetchAssets()])
    } catch (err) {
      setOperationOutput(buildErrorOutput({ message: toErrorMessage(err), fallback: t('sync.messages.statusFailed'), t }))
    } finally {
      setRefreshingAssets(false)
    }
  }, [fetchAssets, fetchSyncStatus, t])

  const maybeOfferForce = useCallback((assetId: string, operation: SyncAssetOperation, message: string) => {
    if (/already exists|overwrite|force/i.test(message)) setForceRetry({ assetId, operation })
  }, [])

  const runAsset = useCallback(async (asset: SyncAssetInfo, operation: SyncAssetOperation, options: SyncAssetOperationOptions) => {
    setBusyAssetId(asset.id)
    setBusyOperation(operation)
    setForceRetry(null)
    setForceRetryAll(false)
    try {
      const result = operation === 'push'
        ? await pushSyncAsset(asset.id, options)
        : operation === 'pull'
          ? await pullSyncAsset(asset.id, options)
          : await syncSingleAsset(asset.id, options)
      setOperationOutput(buildOperationOutput({ result, fallback: t('sync.messages.operationComplete'), t, assets, targetAsset: asset }))
      if (result?.success === false) {
        maybeOfferForce(asset.id, operation, `${result.message || ''}\n${(result.failed || []).map((failure) => failure.message).join('\n')}`)
      }
      await fetchAssets()
    } catch (err) {
      const message = toErrorMessage(err)
      setOperationOutput(buildErrorOutput({ message, fallback: t('sync.messages.operationFailed'), t, targetAsset: asset }))
      maybeOfferForce(asset.id, operation, message)
      await fetchAssets()
    } finally {
      setBusyAssetId(null)
      setBusyOperation(null)
    }
  }, [assets, fetchAssets, maybeOfferForce, t])

  const runAllAssets = useCallback(async (options: SyncAssetOperationOptions) => {
    setGlobalOperating(true)
    setForceRetry(null)
    setForceRetryAll(false)
    try {
      const result = await syncAllAssets(options)
      setOperationOutput(buildOperationOutput({ result, fallback: t('sync.messages.batchSyncComplete'), t, assets }))
      if (result?.success === false && /already exists|overwrite|force/i.test(`${result.message || ''}`)) setForceRetryAll(true)
      await fetchAssets()
    } catch (err) {
      const message = toErrorMessage(err)
      setOperationOutput(buildErrorOutput({ message, fallback: t('sync.messages.batchSyncFailed'), t }))
      if (/already exists|overwrite|force/i.test(message)) setForceRetryAll(true)
    } finally {
      setGlobalOperating(false)
    }
  }, [assets, fetchAssets, t])

  const requestRunAsset = useCallback((asset: SyncAssetInfo, operation: SyncAssetOperation, force: boolean) => {
    if (!asset.sensitive) {
      void runAsset(asset, operation, { force })
      return
    }
    setPending({ asset, operation, force, all: false })
    setPassphraseModalOpen(true)
  }, [runAsset])

  const requestRunAll = useCallback((force: boolean) => {
    setPending({ force, all: true })
    setPassphraseModalOpen(true)
  }, [])

  const submitSensitiveOperation = useCallback((payload: { passphrase: string; migratePlaintextV1: boolean }) => {
    const current = pending
    setPending(null)
    if (!current) return
    const options: SyncAssetOperationOptions = { force: current.force, passphrase: payload.passphrase, migratePlaintextV1: payload.migratePlaintextV1 }
    if (current.all) void runAllAssets(options)
    else if (current.asset && current.operation) void runAsset(current.asset, current.operation, options)
  }, [pending, runAllAssets, runAsset])

  useEffect(() => {
    let cancelled = false
    setLoading(true)
    Promise.all([fetchSyncStatus(), fetchAssets()])
      .catch((err: unknown) => {
        if (!cancelled) setError(toErrorMessage(err, t('sync.loadFailed')))
      })
      .finally(() => {
        if (!cancelled) setLoading(false)
      })
    return () => {
      cancelled = true
    }
  }, [fetchAssets, fetchSyncStatus, t])

  return {
    t,
    loading,
    error,
    syncStatus,
    assets,
    assetGroups,
    operationOutput,
    refreshingAssets,
    globalOperating,
    busyAssetId,
    busyOperation,
    forceRetry,
    forceRetryAll,
    passphraseModalOpen,
    pending,
    setPassphraseModalOpen,
    connectionState,
    syncGated,
    syncGateReason,
    accountDialogOpen,
    accountDialogMode,
    setAccountDialogOpen,
    openAccountDialog,
    refreshAll,
    requestRunAsset,
    requestRunAll,
    submitSensitiveOperation,
    clearOperationOutput: () => {
      setOperationOutput(null)
      setForceRetry(null)
      setForceRetryAll(false)
    },
  }
}
