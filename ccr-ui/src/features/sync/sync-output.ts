import type { TranslateFunction } from '@/utils/tf'
import type { SyncAssetInfo, SyncOperationOutput, SyncOperationResult } from '@/types/syncSelection'
import {
  extractRemotePathFromMessage,
  isAncestorNotFound,
  maskSecrets,
  normalizeRemoteParentPath,
} from './sync-mask'

const statusLabelOf = (status: SyncOperationOutput['status'], t: TranslateFunction): string => {
  if (status === 'success') return t('sync.output.statusSuccess')
  if (status === 'partial') return t('sync.output.statusPartial')
  return t('sync.output.statusFailed')
}

export function buildOperationOutput(input: {
  result: SyncOperationResult
  fallback: string
  t: TranslateFunction
  assets: SyncAssetInfo[]
  targetAsset?: SyncAssetInfo
}): SyncOperationOutput {
  const { result, fallback, t, assets, targetAsset } = input
  const resultFailures = result.failed
  const fallbackFailure = result.success === false && resultFailures.length === 0
    ? [{ folder: targetAsset?.id ?? t('sync.output.unknownAsset'), message: result.message || fallback }]
    : null
  const outputFailures = fallbackFailure ?? resultFailures
  const failedCount = outputFailures.length
  const successCount = result.successCount
  const status: SyncOperationOutput['status'] = failedCount > 0 ? ((successCount ?? 0) > 0 ? 'partial' : 'failed') : 'success'
  const title = targetAsset ? `${targetAsset.name} · ${statusLabelOf(status, t)}` : statusLabelOf(status, t)
  const failures = outputFailures.map((failure) => {
    const asset = assets.find((item) => item.id === failure.folder || item.name === failure.folder || item.canonicalName === failure.folder) ?? targetAsset
    const maskedMessage = maskSecrets(failure.message)
    const ancestorFailure = isAncestorNotFound(failure.message)
    const remotePath = asset?.remotePath || extractRemotePathFromMessage(failure.message)
    return {
      assetId: asset?.id,
      assetName: asset?.name ?? failure.folder,
      message: maskedMessage,
      reason: ancestorFailure ? t('sync.output.ancestorReason') : maskedMessage,
      localPath: asset?.resolvedLocalPath || asset?.localPath,
      remotePath,
      advice: ancestorFailure ? t('sync.output.ancestorAdvice', { path: normalizeRemoteParentPath(remotePath ?? failure.folder) }) : '',
    }
  })
  return {
    status,
    title,
    summary: maskSecrets(result.message || fallback),
    total: result.total,
    successCount,
    failedCount,
    durationMs: result.durationMs,
    failures,
    suggestions: [...new Set(failures.map((item) => item.advice).filter((item): item is string => Boolean(item)))],
    rawLog: maskSecrets(JSON.stringify({
      title,
      summary: maskSecrets(result.message || fallback),
      total: result.total,
      successCount,
      failedCount,
      failures: outputFailures.map((failure) => ({ folder: failure.folder, message: failure.message })),
      durationMs: result.durationMs,
    }, null, 2)),
  }
}

function errorFailure(input: {
  message: string
  t: TranslateFunction
  targetAsset?: SyncAssetInfo
}) {
  const maskedMessage = maskSecrets(input.message)
  const ancestorFailure = isAncestorNotFound(input.message)
  return {
    assetId: input.targetAsset?.id,
    assetName: input.targetAsset?.name ?? input.t('sync.output.unknownAsset'),
    message: maskedMessage,
    reason: ancestorFailure ? input.t('sync.output.ancestorReason') : maskedMessage,
    localPath: input.targetAsset?.resolvedLocalPath || input.targetAsset?.localPath,
    remotePath: input.targetAsset?.remotePath,
    advice: ancestorFailure ? input.t('sync.output.ancestorAdvice', { path: normalizeRemoteParentPath(input.targetAsset?.remotePath ?? '') }) : '',
  }
}

export function buildErrorOutput(input: {
  message: string
  fallback: string
  t: TranslateFunction
  targetAsset?: SyncAssetInfo
}): SyncOperationOutput {
  const { message, fallback, t, targetAsset } = input
  const maskedMessage = maskSecrets(message)
  const title = targetAsset ? `${targetAsset.name} · ${t('sync.output.statusFailed')}` : t('sync.output.statusFailed')
  const failure = errorFailure({ message, t, targetAsset })
  return {
    status: 'failed',
    title,
    summary: `${fallback}: ${maskedMessage}`,
    total: targetAsset ? 1 : undefined,
    successCount: 0,
    failedCount: 1,
    durationMs: undefined,
    failures: [failure],
    suggestions: failure.advice ? [failure.advice] : [],
    rawLog: maskSecrets(JSON.stringify({
      title,
      summary: `${fallback}: ${maskedMessage}`,
      error: message,
      asset: targetAsset
        ? { id: targetAsset.id, name: targetAsset.name, localPath: targetAsset.resolvedLocalPath || targetAsset.localPath, remotePath: targetAsset.remotePath }
        : null,
    }, null, 2)),
  }
}
