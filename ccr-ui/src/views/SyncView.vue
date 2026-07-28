<template>
  <div class="sync-page">
    <main class="sync-shell">
      <PageHeaderCard
        :title="$t('sync.title')"
        :description="$t('sync.subtitle')"
        :badge="$t('sync.assets.badge')"
        icon="Cloud"
        tone="secondary"
      >
        <template #actions>
          <button
            type="button"
            class="sync-hero-button sync-hero-button--ghost"
            :disabled="loading || refreshingAssets"
            @click="refreshAll"
          >
            <SIcon
              name="RefreshCw"
              size="w-4 h-4"
              :class="{ 'animate-spin': refreshingAssets }"
            />
            <span>{{ $t('sync.assets.refresh') }}</span>
          </button>
          <button
            type="button"
            class="sync-hero-button sync-hero-button--primary"
            :disabled="globalOperating || assets.length === 0"
            @click="requestRunAll(false)"
          >
            <SIcon
              name="Sparkles"
              size="w-4 h-4"
            />
            <span>{{ globalOperating ? $t('sync.assets.syncingAll') : $t('sync.assets.syncAll') }}</span>
          </button>
          <button
            v-if="forceRetryAll"
            type="button"
            class="sync-hero-button sync-hero-button--warning"
            :disabled="globalOperating || assets.length === 0"
            @click="requestRunAll(true)"
          >
            <SIcon
              name="Shield"
              size="w-4 h-4"
            />
            <span>{{ $t('sync.assets.forceRetryAll') }}</span>
          </button>
          <RouterLink
            to="/"
            class="sync-back-link"
          >
            <SIcon
              name="Home"
              size="w-4 h-4"
            />
            <span>{{ $t('sync.backHome') }}</span>
          </RouterLink>
        </template>

        <div class="sync-scope-strip">
          <div
            v-for="item in scopeHighlights"
            :key="item.key"
            class="sync-scope-strip__item"
          >
            <span class="sync-scope-strip__label">{{ item.label }}</span>
            <strong>{{ item.value }}</strong>
          </div>
        </div>
      </PageHeaderCard>

      <AsyncStatePanel
        v-if="loading"
        state="loading"
        :title="$t('common.loading')"
      />

      <AsyncStatePanel
        v-else-if="error"
        state="error"
        :title="$t('sync.loadFailed')"
        :description="error"
      />

      <div
        v-else
        class="sync-console-grid"
      >
        <section class="sync-console-main">
          <div class="sync-console-intro">
            <div>
              <p class="sync-eyebrow">
                {{ $t('sync.assets.eyebrow') }}
              </p>
              <h2>{{ $t('sync.assets.title') }}</h2>
              <p>{{ $t('sync.assets.description') }}</p>
            </div>
            <div class="sync-console-intro__meta">
              <span>{{ $t('sync.assets.total', { count: assets.length }) }}</span>
              <span>{{ $t('sync.assets.sensitiveHint') }}</span>
            </div>
          </div>

          <div class="sync-asset-groups">
            <article
              v-for="group in assetGroups"
              :key="group.key"
              class="sync-asset-group"
            >
              <header class="sync-asset-group__header">
                <div>
                  <p class="sync-eyebrow">
                    {{ groupLabel(group.key) }}
                  </p>
                  <h3>{{ group.title }}</h3>
                  <p>{{ group.description }}</p>
                </div>
                <span class="sync-count-chip">{{ $t('sync.assets.itemCount', { count: group.assets.length }) }}</span>
              </header>

              <div class="sync-asset-list">
                <div
                  v-for="asset in group.assets"
                  :key="asset.id"
                  class="sync-asset-card"
                  :class="{ 'sync-asset-card--missing': !asset.localExists }"
                >
                  <div class="sync-asset-card__body">
                    <div class="sync-asset-card__icon">
                      <SIcon
                        :name="assetIcon(asset)"
                        size="w-5 h-5"
                      />
                    </div>
                    <div class="sync-asset-card__content">
                      <div class="sync-asset-card__title-row">
                        <h4>{{ asset.name }}</h4>
                        <span
                          v-if="asset.sensitive"
                          class="sync-sensitive-chip"
                        >{{ $t('sync.assets.sensitive') }}</span>
                        <span
                          v-if="asset.encryptionState === 'v2_required'"
                          class="sync-encryption-chip"
                        >{{ $t('sync.assets.encryptionV2') }}</span>
                        <span class="sync-kind-chip">{{ kindLabel(asset.kind) }}</span>
                      </div>
                      <p>{{ asset.description }}</p>
                      <dl class="sync-path-grid">
                        <div>
                          <dt>{{ $t('sync.assets.localPath') }}</dt>
                          <dd :title="localPathTitle(asset)">
                            {{ normalizedLocalPath(asset) }}
                          </dd>
                        </div>
                        <div>
                          <dt>{{ $t('sync.assets.remotePath') }}</dt>
                          <dd :title="normalizedRemotePath(asset)">
                            {{ normalizedRemotePath(asset) }}
                          </dd>
                        </div>
                      </dl>
                      <div class="sync-status-row">
                        <span :class="statusClass(asset.localExists)">
                          <SIcon
                            :name="asset.localExists ? 'CheckCircle' : 'AlertCircle'"
                            size="w-3.5 h-3.5"
                          />
                          {{ asset.localExists ? $t('sync.assetStatus.localReady') : $t('sync.assetStatus.localMissing') }}
                        </span>
                        <span :class="remoteStatusClass(asset.remoteExists)">
                          <SIcon
                            :name="remoteStatusIcon(asset.remoteExists)"
                            size="w-3.5 h-3.5"
                          />
                          {{ remoteStatusText(asset.remoteExists) }}
                        </span>
                        <span
                          v-if="asset.canonicalName"
                          class="sync-status-chip sync-status-chip--neutral"
                        >{{ $t('sync.assets.canonical', { name: asset.canonicalName }) }}</span>
                      </div>
                    </div>
                  </div>

                  <div class="sync-asset-card__actions">
                    <button
                      type="button"
                      class="sync-action-button sync-action-button--push"
                      :disabled="isAssetBusy(asset.id) || !asset.localExists"
                      @click="requestRunAsset(asset, 'push', false)"
                    >
                      <SIcon
                        name="Upload"
                        size="w-4 h-4"
                      />
                      {{ $t('sync.assetActions.push') }}
                    </button>
                    <button
                      type="button"
                      class="sync-action-button sync-action-button--pull"
                      :disabled="isAssetBusy(asset.id)"
                      @click="requestRunAsset(asset, 'pull', false)"
                    >
                      <SIcon
                        name="Download"
                        size="w-4 h-4"
                      />
                      {{ $t('sync.assetActions.pull') }}
                    </button>
                    <button
                      type="button"
                      class="sync-action-button sync-action-button--sync"
                      :disabled="isAssetBusy(asset.id)"
                      @click="requestRunAsset(asset, 'sync', false)"
                    >
                      <SIcon
                        name="RefreshCw"
                        size="w-4 h-4"
                        :class="{ 'animate-spin': isAssetBusy(asset.id) }"
                      />
                      {{ busyLabel(asset.id) || $t('sync.assetActions.sync') }}
                    </button>
                    <button
                      v-if="needsForce(asset.id)"
                      type="button"
                      class="sync-action-button sync-action-button--force"
                      :disabled="isAssetBusy(asset.id)"
                      @click="retryForce(asset)"
                    >
                      <SIcon
                        name="Shield"
                        size="w-4 h-4"
                      />
                      {{ $t('sync.assetActions.forceRetry') }}
                    </button>
                  </div>
                </div>
              </div>
            </article>
          </div>
        </section>

        <aside class="sync-console-side">
          <SyncInfoSidebar
            :sync-status="syncStatus"
            @status-refresh="refreshAll"
          />

          <section class="sync-safety-card">
            <p class="sync-eyebrow">
              {{ $t('sync.assets.safetyTitle') }}
            </p>
            <ul>
              <li>{{ $t('sync.assets.safetyAllowlist') }}</li>
              <li>{{ $t('sync.assets.safetyBackup') }}</li>
              <li>{{ $t('sync.assets.safetyMask') }}</li>
            </ul>
          </section>

          <SyncOperationOutputPanel
            :clear-output="clearOperationOutput"
            :output="operationOutput"
          />
        </aside>
      </div>
    </main>

    <SyncPassphraseModal
      v-model="passphraseModalOpen"
      :asset-name="pendingSensitiveOperation?.asset?.name"
      @submit="submitSensitiveOperation"
    />
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, onMounted, computed } from 'vue'
import { RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'
import AsyncStatePanel from '@/components/ui/AsyncStatePanel.vue'
import PageHeaderCard from '@/components/PageHeaderCard.vue'
import {
  getSyncStatus,
  listSyncAssets,
  pushSyncAsset,
  pullSyncAsset,
  syncSingleAsset,
  syncAllAssets,
} from '@/api'
import SyncInfoSidebar from '@/components/sync/SyncInfoSidebar.vue'
import SyncOperationOutputPanel from '@/components/sync/SyncOperationOutputPanel.vue'
import SyncPassphraseModal from '@/components/sync/SyncPassphraseModal.vue'
import { logger } from '@/utils/logger'
import type {
  SyncAssetGroup,
  SyncAssetInfo,
  SyncAssetKind,
  SyncAssetOperation,
  SyncOperationOutput,
  SyncOperationResult,
  SyncAssetOperationOptions,
  SyncStatusView,
} from '@/types/syncSelection'

const { t } = useI18n()

const GROUP_ORDER = ['ccr', 'claude', 'codex']

const asRecord = (value: unknown): Record<string, unknown> => {
  return typeof value === 'object' && value !== null ? (value as Record<string, unknown>) : {}
}

const toErrorMessage = (error: unknown, fallback = 'unknown error'): string => {
  if (error instanceof Error) return error.message
  if (typeof error === 'string') return error
  const message = asRecord(asRecord(error).response).data
  if (typeof message === 'object' && message !== null && typeof (message as Record<string, unknown>).message === 'string') {
    return String((message as Record<string, unknown>).message)
  }
  return fallback
}

const maskSecrets = (value: string): string => {
  return value
    .replace(/((?:api[_-]?key|token|password|secret)["']?\s*[=:]\s*["']?)([^"',\s;}]+)(["']?)/gi, '$1••••••$3')
    .replace(/\b(Bearer\s+)([A-Za-z0-9._~+/-]+=*)/gi, '$1••••••')
    .replace(/(sk-[A-Za-z0-9_-]{8,})/g, 'sk-••••••')
}

const normalizeAsset = (asset: SyncAssetInfo): SyncAssetInfo => asset

const isAncestorNotFound = (message: string): boolean => {
  return /AncestorNotFound|ancestor\s+not\s+found|ancestor.*not.*found/i.test(message)
}

const normalizeRemoteParentPath = (remotePath: string): string => {
  const trimmed = remotePath.trim().replace(/\/+$/u, '')
  if (!trimmed) return '/ccr/'
  const segments = trimmed.split('/').filter(Boolean)
  if (segments.length <= 1) return '/'
  return `/${segments[0]}/`
}

const extractRemotePathFromMessage = (message: string): string | undefined => {
  const match = message.match(/(?:remote\s+path|for)\s+(\/[^\s,;}]+)/i)
    ?? message.match(/(\/ccr\/[^\s,;}]+)/i)
  return match?.[1]?.replace(/[.)'"]+$/u, '')
}

const getOperationStatusLabel = (status: SyncOperationOutput['status']): string => {
  if (status === 'success') return t('sync.output.statusSuccess')
  if (status === 'partial') return t('sync.output.statusPartial')
  return t('sync.output.statusFailed')
}

const buildOperationOutput = (
  result: SyncOperationResult,
  fallback: string,
  targetAsset?: SyncAssetInfo,
): SyncOperationOutput => {
  const resultFailures = result.failed
  const fallbackFailure = result.success === false && resultFailures.length === 0
    ? [{ folder: targetAsset?.id ?? t('sync.output.unknownAsset'), message: result.message || fallback }]
    : null
  const outputFailures = fallbackFailure ?? resultFailures
  const total = result.total
  const successCount = result.successCount
  const failedCount = outputFailures.length
  const status: SyncOperationOutput['status'] = failedCount > 0
    ? ((successCount ?? 0) > 0 ? 'partial' : 'failed')
    : 'success'
  const title = targetAsset ? `${targetAsset.name} · ${getOperationStatusLabel(status)}` : getOperationStatusLabel(status)
  const summary = maskSecrets(result.message || fallback)
  const failures = outputFailures.map((failure) => {
    const asset = assets.value.find(item => item.id === failure.folder || item.name === failure.folder || item.canonicalName === failure.folder)
      ?? targetAsset
    const maskedMessage = maskSecrets(failure.message)
    const ancestorFailure = isAncestorNotFound(failure.message)
    const remotePath = asset?.remotePath || extractRemotePathFromMessage(failure.message)
    const advice = ancestorFailure
      ? t('sync.output.ancestorAdvice', { path: normalizeRemoteParentPath(remotePath ?? failure.folder) })
      : ''

    return {
      assetId: asset?.id,
      assetName: asset?.name ?? failure.folder,
      message: maskedMessage,
      reason: ancestorFailure ? t('sync.output.ancestorReason') : maskedMessage,
      localPath: asset?.resolvedLocalPath || asset?.localPath,
      remotePath,
      advice,
    }
  })
  const suggestions = [...new Set(failures.map(item => item.advice).filter((item): item is string => Boolean(item)))]
  const rawLog = maskSecrets(JSON.stringify({
    title,
    summary,
    total,
    successCount,
    failedCount,
    failures: outputFailures.map(failure => ({
      folder: failure.folder,
      message: failure.message,
    })),
    durationMs: result.durationMs,
  }, null, 2))

  return {
    status,
    title,
    summary,
    total,
    successCount,
    failedCount,
    durationMs: result.durationMs,
    failures,
    suggestions,
    rawLog,
  }
}

const buildErrorOutput = (
  message: string,
  fallback: string,
  targetAsset?: SyncAssetInfo,
): SyncOperationOutput => {
  const maskedMessage = maskSecrets(message)
  const ancestorFailure = isAncestorNotFound(message)
  const title = targetAsset
    ? `${targetAsset.name} · ${t('sync.output.statusFailed')}`
    : t('sync.output.statusFailed')
  const failure = {
    assetId: targetAsset?.id,
    assetName: targetAsset?.name ?? t('sync.output.unknownAsset'),
    message: maskedMessage,
    reason: ancestorFailure ? t('sync.output.ancestorReason') : maskedMessage,
    localPath: targetAsset?.resolvedLocalPath || targetAsset?.localPath,
    remotePath: targetAsset?.remotePath,
    advice: ancestorFailure
      ? t('sync.output.ancestorAdvice', { path: normalizeRemoteParentPath(targetAsset?.remotePath ?? '') })
      : '',
  }

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
        ? {
            id: targetAsset.id,
            name: targetAsset.name,
            localPath: targetAsset.resolvedLocalPath || targetAsset.localPath,
            remotePath: targetAsset.remotePath,
          }
        : null,
    }, null, 2)),
  }
}

const loading = ref(true)
const error = ref('')
const syncStatus = ref<SyncStatusView | null>(null)
const assets = ref<SyncAssetInfo[]>([])
const operationOutput = ref<SyncOperationOutput | null>(null)
const refreshingAssets = ref(false)
const globalOperating = ref(false)
const busyAssetId = ref<string | null>(null)
const busyOperation = ref<SyncAssetOperation | null>(null)
const forceRetry = ref<{ assetId: string; operation: SyncAssetOperation } | null>(null)
const forceRetryAll = ref(false)
const passphraseModalOpen = ref(false)
const pendingSensitiveOperation = ref<{
  asset?: SyncAssetInfo
  operation?: SyncAssetOperation
  force: boolean
  all: boolean
} | null>(null)

const scopeHighlights = computed(() => [
  { key: 'ccr', label: t('sync.assets.scopeCcrLabel'), value: t('sync.assets.scopeCcrValue') },
  { key: 'claude', label: t('sync.assets.scopeClaudeLabel'), value: t('sync.assets.scopeClaudeValue') },
  { key: 'codex', label: t('sync.assets.scopeCodexLabel'), value: t('sync.assets.scopeCodexValue') },
])

const assetGroups = computed<SyncAssetGroup[]>(() => {
  return GROUP_ORDER.map((key) => ({
    key,
    title: t(`sync.assetGroups.${key}.title`),
    description: t(`sync.assetGroups.${key}.description`),
    assets: assets.value.filter(asset => asset.group === key),
  })).filter(group => group.assets.length > 0)
})

const fetchSyncStatus = async () => {
  try {
    syncStatus.value = await getSyncStatus()
  } catch (err: unknown) {
    logger.error('Failed to fetch sync status:', err)
  }
}

const fetchAssets = async () => {
  const response = await listSyncAssets()
  assets.value = response.map(normalizeAsset)
}

const refreshAll = async () => {
  refreshingAssets.value = true
  try {
    await Promise.all([fetchSyncStatus(), fetchAssets()])
  } catch (err: unknown) {
    operationOutput.value = buildErrorOutput(toErrorMessage(err), t('sync.messages.statusFailed'))
  } finally {
    refreshingAssets.value = false
  }
}

const clearOperationOutput = () => {
  operationOutput.value = null
  forceRetry.value = null
  forceRetryAll.value = false
}

const requestRunAsset = (
  asset: SyncAssetInfo,
  operation: SyncAssetOperation,
  force: boolean
) => {
  if (!asset.sensitive) {
    void runAsset(asset, operation, { force })
    return
  }
  pendingSensitiveOperation.value = { asset, operation, force, all: false }
  passphraseModalOpen.value = true
}

const requestRunAll = (force: boolean) => {
  pendingSensitiveOperation.value = { force, all: true }
  passphraseModalOpen.value = true
}

const submitSensitiveOperation = (payload: { passphrase: string; migratePlaintextV1: boolean }) => {
  const pending = pendingSensitiveOperation.value
  pendingSensitiveOperation.value = null
  if (!pending) return
  const options: SyncAssetOperationOptions = {
    force: pending.force,
    passphrase: payload.passphrase,
    migratePlaintextV1: payload.migratePlaintextV1,
  }
  if (pending.all) {
    void runAllAssets(options)
  } else if (pending.asset && pending.operation) {
    void runAsset(pending.asset, pending.operation, options)
  }
}

const runAsset = async (
  asset: SyncAssetInfo,
  operation: SyncAssetOperation,
  options: SyncAssetOperationOptions
) => {
  busyAssetId.value = asset.id
  busyOperation.value = operation
  forceRetry.value = null
  forceRetryAll.value = false
  try {
    const result = operation === 'push'
      ? await pushSyncAsset(asset.id, options)
      : operation === 'pull'
        ? await pullSyncAsset(asset.id, options)
        : await syncSingleAsset(asset.id, options)

    operationOutput.value = buildOperationOutput(result, t('sync.messages.operationComplete'), asset)
    if (result?.success === false) {
      maybeOfferForce(asset.id, operation, `${result.message || ''}\n${(result.failed || []).map(failure => failure.message).join('\n')}`)
    }
    await fetchAssets()
  } catch (err: unknown) {
    const message = toErrorMessage(err)
    operationOutput.value = buildErrorOutput(message, t('sync.messages.operationFailed'), asset)
    maybeOfferForce(asset.id, operation, message)
    await fetchAssets()
  } finally {
    busyAssetId.value = null
    busyOperation.value = null
  }
}

const runAllAssets = async (options: SyncAssetOperationOptions) => {
  globalOperating.value = true
  forceRetry.value = null
  forceRetryAll.value = false
  try {
    const result = await syncAllAssets(options)
    operationOutput.value = buildOperationOutput(result, t('sync.messages.batchSyncComplete'))
    if (result?.success === false) {
      maybeOfferForceAll(`${result.message || ''}\n${(result.failed || []).map(failure => failure.message).join('\n')}`)
    }
    await fetchAssets()
  } catch (err: unknown) {
    const message = toErrorMessage(err)
    operationOutput.value = buildErrorOutput(message, t('sync.messages.batchSyncFailed'))
    maybeOfferForceAll(message)
  } finally {
    globalOperating.value = false
  }
}

const maybeOfferForce = (assetId: string, operation: SyncAssetOperation, message: string) => {
  if (/already exists|overwrite|force/i.test(message)) {
    forceRetry.value = { assetId, operation }
  }
}

const maybeOfferForceAll = (message: string) => {
  if (/already exists|overwrite|force/i.test(message)) {
    forceRetryAll.value = true
  }
}

const retryForce = async (asset: SyncAssetInfo) => {
  const retry = forceRetry.value
  if (!retry || retry.assetId !== asset.id) return
  requestRunAsset(asset, retry.operation, true)
}

const needsForce = (assetId: string) => forceRetry.value?.assetId === assetId

const isAssetBusy = (assetId: string) => globalOperating.value || busyAssetId.value === assetId

const busyLabel = (assetId: string) => {
  if (busyAssetId.value !== assetId || !busyOperation.value) return ''
  return t(`sync.assetActions.${busyOperation.value}ing`)
}

const normalizedLocalPath = (asset: SyncAssetInfo) => asset.resolvedLocalPath || asset.localPath
const normalizedRemotePath = (asset: SyncAssetInfo) => asset.remotePath
const localPathTitle = (asset: SyncAssetInfo) => `${asset.localPath} -> ${normalizedLocalPath(asset)}`

const groupLabel = (key: string) => t(`sync.assetGroups.${key}.label`)
const kindLabel = (kind: SyncAssetKind) => kind === 'directory' ? t('sync.assets.kindDirectory') : t('sync.assets.kindFile')
const assetIcon = (asset: SyncAssetInfo) => asset.kind === 'directory' ? 'Folder' : 'FileText'
const statusClass = (ok: boolean) => ok ? 'sync-status-chip sync-status-chip--ok' : 'sync-status-chip sync-status-chip--fail'

const remoteStatusClass = (value: boolean | null | undefined) => {
  if (value === true) return 'sync-status-chip sync-status-chip--ok'
  if (value === false) return 'sync-status-chip sync-status-chip--fail'
  return 'sync-status-chip sync-status-chip--neutral'
}

const remoteStatusIcon = (value: boolean | null | undefined) => {
  if (value === true) return 'CheckCircle'
  if (value === false) return 'AlertCircle'
  return 'Cloud'
}

const remoteStatusText = (value: boolean | null | undefined) => {
  if (value === true) return t('sync.assetStatus.remoteReady')
  if (value === false) return t('sync.assetStatus.remoteMissing')
  return t('sync.assetStatus.remoteUnknown')
}

onMounted(async () => {
  loading.value = true
  try {
    await Promise.all([fetchSyncStatus(), fetchAssets()])
  } catch (err: unknown) {
    error.value = toErrorMessage(err, t('sync.loadFailed'))
  } finally {
    loading.value = false
  }
})
</script>

<style scoped>
.sync-page {
  @apply px-4 py-4 sm:px-6 sm:py-6;
}

.sync-shell {
  @apply mx-auto flex max-w-[1440px] flex-col gap-5;
}

.sync-back-link,
.sync-hero-button {
  @apply inline-flex items-center gap-2 rounded-full border px-4 py-2 text-sm font-semibold transition-colors duration-200 disabled:cursor-not-allowed disabled:opacity-55;
}

.sync-back-link,
.sync-hero-button--ghost {
  border-color: rgb(var(--color-border-default-rgb) / 42%);
  background: rgb(var(--color-bg-elevated-rgb) / 72%);
  color: var(--color-text-secondary);
}

.sync-back-link:hover,
.sync-hero-button--ghost:hover:not(:disabled) {
  border-color: rgb(var(--color-accent-primary-rgb) / 22%);
  color: var(--color-text-primary);
}

.sync-hero-button--primary {
  border-color: var(--color-border-subtle);
  background: var(--color-text-primary);
  color: var(--color-bg-base);
}

.sync-hero-button--warning {
  border-color: rgb(var(--color-warning-rgb) / 30%);
  background: rgb(var(--color-warning-rgb) / 13%);
  color: var(--accent-warning);
}

.sync-scope-strip {
  @apply grid grid-cols-1 gap-3 md:grid-cols-3;
}

.sync-scope-strip__item {
  @apply rounded-2xl px-4 py-3;

  border: 1px solid rgb(var(--color-border-default-rgb) / 36%);
  background: rgb(var(--color-bg-elevated-rgb) / 66%);
}

.sync-scope-strip__label,
.sync-eyebrow {
  @apply text-xs font-bold uppercase tracking-[0.16em];

  color: var(--color-text-muted);
}

.sync-scope-strip__item strong {
  @apply mt-1 block text-sm;

  color: var(--color-text-primary);
}

.sync-console-grid {
  @apply grid grid-cols-1 gap-6 xl:grid-cols-[minmax(0,1fr)_360px];
}

.sync-console-main,
.sync-safety-card {
  @apply rounded-3xl p-5;

  border: 1px solid rgb(var(--color-border-default-rgb) / 38%);
  background: linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 84%), rgb(var(--color-bg-surface-rgb) / 72%));
  box-shadow: var(--surface-card-shadow);
}

.sync-console-side {
  @apply flex flex-col gap-5;
}

.sync-console-intro {
  @apply mb-5 flex flex-col justify-between gap-4 border-b pb-5 md:flex-row;

  border-color: rgb(var(--color-border-default-rgb) / 36%);
}

.sync-console-intro h2 {
  @apply mt-1 text-2xl font-semibold tracking-[-0.03em];

  color: var(--color-text-primary);
}

.sync-console-intro p:not(.sync-eyebrow) {
  @apply mt-2 max-w-2xl text-sm leading-6;

  color: var(--color-text-secondary);
}

.sync-console-intro__meta {
  @apply flex flex-wrap items-start gap-2 md:justify-end;
}

.sync-console-intro__meta span,
.sync-count-chip,
.sync-kind-chip,
.sync-sensitive-chip,
.sync-encryption-chip,
.sync-status-chip {
  @apply inline-flex items-center gap-1.5 rounded-full px-3 py-1 text-xs font-semibold;
}

.sync-console-intro__meta span,
.sync-count-chip,
.sync-kind-chip,
.sync-status-chip--neutral {
  border: 1px solid rgb(var(--color-border-default-rgb) / 38%);
  background: rgb(var(--color-bg-elevated-rgb) / 58%);
  color: var(--color-text-secondary);
}

.sync-asset-groups {
  @apply flex flex-col gap-4;
}

.sync-asset-group {
  @apply rounded-2xl p-4;

  border: 1px solid rgb(var(--color-border-default-rgb) / 34%);
  background: rgb(var(--color-bg-base-rgb) / 44%);
}

.sync-asset-group__header {
  @apply mb-4 flex flex-col justify-between gap-3 md:flex-row md:items-start;
}

.sync-asset-group__header h3 {
  @apply mt-1 text-xl font-semibold tracking-[-0.02em];

  color: var(--color-text-primary);
}

.sync-asset-group__header p:not(.sync-eyebrow) {
  @apply mt-1 text-sm leading-6;

  color: var(--color-text-secondary);
}

.sync-asset-list {
  @apply flex flex-col gap-3;
}

.sync-asset-card {
  @apply rounded-2xl p-4;

  border: 1px solid rgb(var(--color-border-default-rgb) / 36%);
  background: rgb(var(--color-bg-elevated-rgb) / 72%);
}

.sync-asset-card--missing {
  border-color: rgb(var(--color-warning-rgb) / 34%);
}

.sync-asset-card__body {
  @apply flex gap-4;
}

.sync-asset-card__icon {
  @apply flex h-11 w-11 shrink-0 items-center justify-center rounded-2xl;

  border: 1px solid rgb(var(--color-border-default-rgb) / 36%);
  background: rgb(var(--color-bg-surface-rgb) / 76%);
  color: var(--color-text-secondary);
}

.sync-asset-card__content {
  @apply min-w-0 flex-1;
}

.sync-asset-card__title-row {
  @apply flex flex-wrap items-center gap-2;
}

.sync-asset-card h4 {
  @apply text-lg font-semibold tracking-[-0.02em];

  color: var(--color-text-primary);
}

.sync-asset-card p {
  @apply mt-1 text-sm leading-6;

  color: var(--color-text-secondary);
}

.sync-sensitive-chip {
  border: 1px solid rgb(var(--color-warning-rgb) / 30%);
  background: rgb(var(--color-warning-rgb) / 12%);
  color: var(--accent-warning);
}

.sync-encryption-chip {
  border: 1px solid rgb(var(--color-success-rgb) / 30%);
  background: rgb(var(--color-success-rgb) / 10%);
  color: var(--accent-success);
}

.sync-path-grid {
  @apply mt-3 grid grid-cols-1 gap-2 lg:grid-cols-2;
}

.sync-path-grid div {
  @apply min-w-0 rounded-xl px-3 py-2;

  border: 1px solid rgb(var(--color-border-default-rgb) / 28%);
  background: rgb(var(--color-bg-surface-rgb) / 62%);
}

.sync-path-grid dt {
  @apply text-[0.68rem] font-bold uppercase tracking-[0.13em];

  color: var(--color-text-muted);
}

.sync-path-grid dd {
  @apply mt-1 truncate font-mono text-xs;

  color: var(--color-text-secondary);
}

.sync-status-row {
  @apply mt-3 flex flex-wrap gap-2;
}

.sync-status-chip--ok {
  border: 1px solid rgb(var(--color-success-rgb) / 30%);
  background: rgb(var(--color-success-rgb) / 12%);
  color: var(--accent-success);
}

.sync-status-chip--fail {
  border: 1px solid rgb(var(--color-danger-rgb) / 28%);
  background: rgb(var(--color-danger-rgb) / 10%);
  color: var(--accent-danger);
}

.sync-asset-card__actions {
  @apply mt-4 flex flex-wrap gap-2 pl-0 md:pl-[3.75rem];
}

.sync-action-button {
  @apply inline-flex items-center gap-2 rounded-xl border px-3.5 py-2 text-sm font-semibold transition-all duration-200 disabled:cursor-not-allowed disabled:opacity-45;
}

.sync-action-button--push {
  border-color: rgb(var(--color-success-rgb) / 26%);
  background: rgb(var(--color-success-rgb) / 10%);
  color: var(--accent-success);
}

.sync-action-button--pull {
  border-color: rgb(var(--color-info-rgb) / 26%);
  background: rgb(var(--color-info-rgb) / 10%);
  color: var(--accent-info);
}

.sync-action-button--sync {
  border-color: var(--color-border-subtle);
  background: var(--color-text-primary);
  color: var(--color-bg-base);
}

.sync-action-button--force {
  border-color: rgb(var(--color-warning-rgb) / 30%);
  background: rgb(var(--color-warning-rgb) / 13%);
  color: var(--accent-warning);
}

.sync-action-button:hover:not(:disabled),
.sync-back-link:hover,
.sync-hero-button:hover:not(:disabled) {
  transform: translateY(-1px);
}

.sync-action-button:focus-visible,
.sync-back-link:focus-visible,
.sync-hero-button:focus-visible {
  outline: 2px solid rgb(var(--color-accent-primary-rgb) / 54%);
  outline-offset: 2px;
}

.sync-safety-card ul {
  @apply mt-3 space-y-2 text-sm leading-6;

  color: var(--color-text-secondary);
}

@media (width <= 640px) {
  .sync-page {
    @apply px-3 py-3;
  }

  .sync-console-main,
  .sync-safety-card {
    @apply rounded-2xl p-4;
  }

  .sync-asset-group {
    @apply p-3;
  }

  .sync-asset-card {
    @apply p-3;
  }

  .sync-asset-card__body {
    @apply flex-col gap-3;
  }

  .sync-asset-card__icon {
    @apply h-10 w-10 rounded-xl;
  }

  .sync-asset-card__actions {
    @apply grid grid-cols-2 pl-0;
  }

  .sync-action-button {
    @apply justify-center px-3;
  }

  .sync-action-button--force {
    @apply col-span-2;
  }

  .sync-path-grid dd {
    white-space: normal;
    word-break: break-all;
  }
}
</style>
