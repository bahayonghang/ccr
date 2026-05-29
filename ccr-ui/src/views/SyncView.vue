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
            @click="runAllAssets(false)"
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
            @click="runAllAssets(true)"
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
                      @click="runAsset(asset, 'push', false)"
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
                      @click="runAsset(asset, 'pull', false)"
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
                      @click="runAsset(asset, 'sync', false)"
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
import { logger } from '@/utils/logger'
import type {
  SyncAssetGroup,
  SyncAssetInfo,
  SyncAssetKind,
  SyncAssetOperation,
  SyncOperationResult,
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

const normalizeAsset = (asset: SyncAssetInfo): SyncAssetInfo => {
  const raw = asset as SyncAssetInfo
  return {
    ...raw,
    localPath: raw.localPath ?? raw.local_path ?? '',
    resolvedLocalPath: raw.resolvedLocalPath ?? raw.resolved_local_path ?? raw.localPath ?? raw.local_path ?? '',
    remotePath: raw.remotePath ?? raw.remote_path ?? '',
    localExists: raw.localExists ?? raw.local_exists ?? false,
    remoteExists: raw.remoteExists ?? raw.remote_exists ?? null,
    canonicalName: raw.canonicalName ?? raw.canonical_name ?? null,
  }
}

const formatOperationResult = (result: SyncOperationResult, fallback: string): string => {
  const output = result?.data?.output || result?.output
  if (output) return maskSecrets(output)

  const lines = [result?.message || fallback]
  if (typeof result?.total === 'number') {
    const successCount = result.successCount ?? result.success_count ?? 0
    lines.push(`${successCount}/${result.total} succeeded`)
  }
  for (const failure of result?.failed || []) {
    lines.push(`- ${failure.folder}: ${failure.message}`)
  }
  return maskSecrets(lines.join('\n'))
}

const maskSecrets = (value: string): string => {
  return value
    .replace(/(api[_-]?key|token|password|secret|bearer)(\s*[=:]\s*)([^\s,;}]+)/gi, '$1$2••••••')
    .replace(/(sk-[A-Za-z0-9_-]{8,})/g, 'sk-••••••')
}

const loading = ref(true)
const error = ref('')
const syncStatus = ref<SyncStatusView | null>(null)
const assets = ref<SyncAssetInfo[]>([])
const operationOutput = ref('')
const refreshingAssets = ref(false)
const globalOperating = ref(false)
const busyAssetId = ref<string | null>(null)
const busyOperation = ref<SyncAssetOperation | null>(null)
const forceRetry = ref<{ assetId: string; operation: SyncAssetOperation } | null>(null)
const forceRetryAll = ref(false)

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
    syncStatus.value = await getSyncStatus<SyncStatusView>()
  } catch (err: unknown) {
    logger.error('Failed to fetch sync status:', err)
  }
}

const fetchAssets = async () => {
  const response = await listSyncAssets<SyncAssetInfo[]>()
  assets.value = response.map(normalizeAsset)
}

const refreshAll = async () => {
  refreshingAssets.value = true
  try {
    await Promise.all([fetchSyncStatus(), fetchAssets()])
  } catch (err: unknown) {
    operationOutput.value = `${t('sync.messages.statusFailed')}: ${toErrorMessage(err)}`
  } finally {
    refreshingAssets.value = false
  }
}

const clearOperationOutput = () => {
  operationOutput.value = ''
  forceRetry.value = null
  forceRetryAll.value = false
}

const runAsset = async (asset: SyncAssetInfo, operation: SyncAssetOperation, force: boolean) => {
  busyAssetId.value = asset.id
  busyOperation.value = operation
  forceRetry.value = null
  forceRetryAll.value = false
  try {
    const result = operation === 'push'
      ? await pushSyncAsset<SyncOperationResult>(asset.id, force)
      : operation === 'pull'
        ? await pullSyncAsset<SyncOperationResult>(asset.id, force)
        : await syncSingleAsset<SyncOperationResult>(asset.id, force)

    operationOutput.value = `[${asset.name}] ${formatOperationResult(result, t('sync.messages.operationComplete'))}`
    if (result?.success === false) {
      maybeOfferForce(asset.id, operation, operationOutput.value)
    }
    await fetchAssets()
  } catch (err: unknown) {
    const message = toErrorMessage(err)
    operationOutput.value = `[${asset.name}] ${t('sync.messages.operationFailed')}: ${maskSecrets(message)}`
    maybeOfferForce(asset.id, operation, message)
    await fetchAssets()
  } finally {
    busyAssetId.value = null
    busyOperation.value = null
  }
}

const runAllAssets = async (force: boolean) => {
  globalOperating.value = true
  forceRetry.value = null
  forceRetryAll.value = false
  try {
    const result = await syncAllAssets<SyncOperationResult>(force)
    operationOutput.value = formatOperationResult(result, t('sync.messages.batchSyncComplete'))
    if (result?.success === false) {
      maybeOfferForceAll(operationOutput.value)
    }
    await fetchAssets()
  } catch (err: unknown) {
    const message = toErrorMessage(err)
    operationOutput.value = `${t('sync.messages.batchSyncFailed')}: ${maskSecrets(message)}`
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
  await runAsset(asset, retry.operation, true)
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
  border-color: rgb(var(--color-text-primary-rgb) / 8%);
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
  border-color: rgb(var(--color-text-primary-rgb) / 12%);
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
