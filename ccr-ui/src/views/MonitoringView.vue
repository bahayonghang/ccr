<template>
  <main class="min-h-full bg-bg-base">
    <section class="flex w-full max-w-none flex-col gap-4">
      <header class="rounded-[24px] border border-border-default/55 bg-bg-elevated/80 p-4 shadow-sm shadow-black/5">
        <div class="flex flex-col gap-5 lg:flex-row lg:items-end lg:justify-between">
          <div class="max-w-3xl">
            <p class="text-xs font-semibold uppercase tracking-[0.28em] text-text-muted">
              {{ t('monitoring.eyebrow') }}
            </p>
            <h1 class="mt-2 text-2xl font-semibold tracking-tight text-text-primary sm:text-3xl">
              {{ t('monitoring.title') }}
            </h1>
            <p class="mt-2 max-w-2xl text-sm leading-6 text-text-secondary">
              {{ t('monitoring.subtitle') }}
            </p>
          </div>

          <div class="flex flex-wrap items-center gap-2">
            <div
              data-testid="monitoring-connection-status"
              class="inline-flex items-center gap-2 rounded-full border px-3 py-1.5 text-xs font-medium"
              :class="isConnected
                ? 'border-accent-success/30 bg-accent-success/8 text-accent-success'
                : 'border-accent-danger/30 bg-accent-danger/8 text-accent-danger'"
            >
              <span
                class="h-2 w-2 rounded-full"
                :class="isConnected ? 'bg-accent-success' : 'bg-accent-danger'"
              />
              {{ isConnected ? t('monitoring.connected') : t('monitoring.disconnected') }}
            </div>

            <button
              type="button"
              class="inline-flex items-center gap-2 rounded-xl border border-border-default/55 bg-bg-surface/80 px-3 py-2 text-xs font-medium text-text-secondary transition-colors hover:border-accent-secondary/30 hover:text-text-primary disabled:cursor-not-allowed disabled:opacity-60"
              :disabled="usageLoading"
              @click="refreshMonitoring"
            >
              <SIcon
                name="RefreshCw"
                size="w-3.5 h-3.5"
                :class="usageLoading ? 'animate-spin' : ''"
              />
              {{ t('monitoring.refresh') }}
            </button>

            <button
              type="button"
              class="inline-flex items-center gap-2 rounded-xl border border-border-default/55 bg-bg-surface/80 px-3 py-2 text-xs font-medium text-text-secondary transition-colors hover:border-accent-danger/25 hover:text-accent-danger"
              @click="clearLocalLogs"
            >
              <SIcon
                name="Trash2"
                size="w-3.5 h-3.5"
              />
              {{ t('monitoring.clearView') }}
            </button>
          </div>
        </div>
      </header>

      <div class="grid min-w-0 gap-4 xl:grid-cols-[minmax(320px,360px)_minmax(0,1fr)] 2xl:grid-cols-[minmax(340px,380px)_minmax(0,1fr)]">
        <aside class="min-w-0 space-y-4">
          <section class="rounded-[24px] border border-border-default/55 bg-bg-elevated/80 p-4 shadow-sm shadow-black/5">
            <div class="flex items-start justify-between gap-4">
              <div>
                <p class="text-xs font-semibold uppercase tracking-[0.2em] text-text-muted">
                  {{ t('monitoring.usageEyebrow') }}
                </p>
                <h2 class="mt-1 text-lg font-semibold text-text-primary">
                  {{ t('monitoring.usageTitle') }}
                </h2>
              </div>
              <span
                class="rounded-full border px-2.5 py-1 text-[11px] font-medium"
                :class="usageStatusClass"
              >
                {{ usageStatusLabel }}
              </span>
            </div>

            <div
              v-if="usageUnavailable"
              data-testid="monitoring-usage-unavailable"
              class="mt-4 rounded-2xl border border-accent-warning/25 bg-accent-warning/8 px-4 py-3 text-sm text-text-secondary"
            >
              <div class="flex items-start gap-3">
                <SIcon
                  name="AlertTriangle"
                  size="w-4 h-4"
                  class="mt-0.5 shrink-0 text-accent-warning"
                />
                <div>
                  <p class="font-medium text-text-primary">
                    {{ t('monitoring.usageUnavailable') }}
                  </p>
                  <p class="mt-1 text-xs leading-5 text-text-muted">
                    {{ usageUnavailableDetail }}
                  </p>
                </div>
              </div>
            </div>

            <div class="mt-3 grid grid-cols-2 gap-2">
              <div
                v-for="card in usageMetricCards"
                :key="card.id"
                class="rounded-2xl border border-border-default/45 bg-bg-surface/68 p-3"
                :data-testid="`monitoring-usage-card-${card.id}`"
              >
                <div class="flex items-center justify-between gap-3">
                  <p class="text-xs font-medium text-text-muted">
                    {{ card.label }}
                  </p>
                  <SIcon
                    :name="card.icon"
                    size="w-4 h-4"
                    class="text-text-muted"
                  />
                </div>
                <p class="mt-2 text-xl font-semibold tabular-nums tracking-tight text-text-primary">
                  {{ card.value }}
                </p>
                <p class="mt-1 min-h-5 text-xs leading-5 text-text-muted">
                  {{ card.detail }}
                </p>
              </div>
            </div>
          </section>

          <section class="rounded-[24px] border border-border-default/55 bg-bg-elevated/80 p-4 shadow-sm shadow-black/5">
            <div class="flex items-center justify-between gap-3">
              <div>
                <p class="text-xs font-semibold uppercase tracking-[0.2em] text-text-muted">
                  {{ t('monitoring.healthEyebrow') }}
                </p>
                <h2 class="mt-1 text-lg font-semibold text-text-primary">
                  {{ t('monitoring.healthTitle') }}
                </h2>
              </div>
              <span
                class="rounded-full border px-2.5 py-1 text-[11px] font-medium"
                :class="healthStatusClass"
              >
                {{ healthStatusLabel }}
              </span>
            </div>

            <div class="mt-3 grid grid-cols-2 gap-2">
              <button
                v-for="level in monitoringLevels"
                :key="level"
                type="button"
                class="rounded-2xl border px-3 py-3 text-left transition-colors"
                :class="filterLevel === level
                  ? 'border-accent-secondary/35 bg-accent-secondary/10'
                  : 'border-border-default/45 bg-bg-surface/60 hover:border-border-default'"
                @click="filterLevel = level"
              >
                <div class="flex items-center justify-between gap-2">
                  <span class="inline-flex items-center gap-2 text-xs font-medium uppercase tracking-[0.14em] text-text-secondary">
                    <span
                      class="h-2 w-2 rounded-full"
                      :class="getLevelDotClass(level)"
                    />
                    {{ getLevelLabel(level) }}
                  </span>
                  <span class="font-mono text-sm font-semibold text-text-primary">
                    {{ levelCounts[level] }}
                  </span>
                </div>
              </button>
            </div>

            <div class="mt-3 rounded-2xl border border-border-default/45 bg-bg-surface/60 p-3">
              <p class="text-xs font-semibold uppercase tracking-[0.18em] text-text-muted">
                {{ t('monitoring.recentUsageImport') }}
              </p>
              <div
                v-if="latestUsageEvent"
                class="mt-3 space-y-1"
              >
                <div class="flex items-center gap-2 text-xs text-text-muted">
                  <span
                    class="rounded-full px-2 py-0.5 font-semibold uppercase"
                    :class="getLevelClass(latestUsageEvent.level)"
                  >
                    {{ latestUsageEvent.level }}
                  </span>
                  <span>{{ formatTime(latestUsageEvent.timestamp) }}</span>
                  <span>{{ latestUsageEvent.source }}</span>
                </div>
                <p class="line-clamp-2 text-sm leading-5 text-text-secondary">
                  {{ latestUsageEvent.message }}
                </p>
              </div>
              <p
                v-else
                class="mt-3 text-sm text-text-muted"
              >
                {{ t('monitoring.noUsageImportEvent') }}
              </p>
            </div>

            <div class="mt-3 rounded-2xl border border-border-default/45 bg-bg-surface/60 p-3">
              <p class="text-xs font-semibold uppercase tracking-[0.18em] text-text-muted">
                {{ t('monitoring.recentIssues') }}
              </p>
              <div
                v-if="recentIssueEvents.length > 0"
                class="mt-3 space-y-2"
              >
                <div
                  v-for="event in recentIssueEvents"
                  :key="event.id"
                  class="rounded-xl border border-border-default/35 bg-bg-elevated/50 px-3 py-2"
                >
                  <div class="flex items-center gap-2 text-[11px] text-text-muted">
                    <span
                      class="rounded-full px-2 py-0.5 font-semibold uppercase"
                      :class="getLevelClass(event.level)"
                    >
                      {{ event.level }}
                    </span>
                    <span>{{ formatTime(event.timestamp) }}</span>
                    <span class="truncate">{{ event.channel }}</span>
                  </div>
                  <p class="mt-1 line-clamp-2 text-xs leading-5 text-text-secondary">
                    {{ event.message }}
                  </p>
                </div>
              </div>
              <p
                v-else
                class="mt-3 text-sm text-text-muted"
              >
                {{ t('monitoring.noRecentIssues') }}
              </p>
            </div>
          </section>
        </aside>

        <section class="min-w-0 overflow-hidden rounded-[24px] border border-border-default/55 bg-bg-elevated/80 shadow-sm shadow-black/5">
          <div class="flex flex-col gap-4 border-b border-border-default/45 p-4 lg:flex-row lg:items-center lg:justify-between">
            <div>
              <p class="text-xs font-semibold uppercase tracking-[0.2em] text-text-muted">
                {{ t('monitoring.logsEyebrow') }}
              </p>
              <h2 class="mt-1 flex items-center gap-2 text-lg font-semibold text-text-primary">
                <SIcon
                  name="Terminal"
                  size="w-4 h-4"
                  class="text-text-muted"
                />
                {{ t('monitoring.realTimeLogs') }}
              </h2>
            </div>

            <div class="flex flex-wrap items-center gap-2">
              <span
                data-testid="monitoring-filtered-count"
                class="rounded-full border border-border-default/45 bg-bg-surface/70 px-3 py-1.5 text-xs font-medium text-text-secondary"
              >
                {{ filteredCountLabel }}
              </span>
              <select
                v-model="filterLevel"
                data-testid="monitoring-level-filter"
                class="rounded-xl border border-border-default/55 bg-bg-surface/80 px-3 py-2 text-xs font-medium text-text-secondary outline-none transition-colors focus:border-accent-secondary/45"
              >
                <option value="all">
                  {{ t('monitoring.allLevels') }}
                </option>
                <option
                  v-for="level in monitoringLevels"
                  :key="level"
                  :value="level"
                >
                  {{ getLevelLabel(level) }}
                </option>
              </select>
            </div>
          </div>

          <div class="overflow-hidden p-3">
            <div class="overflow-x-auto">
              <div class="w-full min-w-[640px] rounded-2xl border border-border-default/45 bg-bg-surface/55 font-mono text-xs">
                <div class="grid grid-cols-[72px_62px_94px_94px_minmax(0,1fr)] gap-2 border-b border-border-default/45 px-3 py-2 text-[11px] font-semibold uppercase tracking-[0.14em] text-text-muted">
                  <span>{{ t('monitoring.columnTime') }}</span>
                  <span>{{ t('monitoring.columnLevel') }}</span>
                  <span>{{ t('monitoring.columnChannel') }}</span>
                  <span>{{ t('monitoring.columnSource') }}</span>
                  <span>{{ t('monitoring.columnMessage') }}</span>
                </div>

                <div
                  ref="logContainer"
                  class="max-h-[620px] min-h-[440px] overflow-y-auto"
                >
                  <div
                    v-for="log in filteredLogs"
                    :key="log.id"
                    data-testid="monitoring-log-row"
                    class="grid grid-cols-[72px_62px_94px_94px_minmax(0,1fr)] gap-2 border-b border-border-default/25 px-3 py-2 last:border-b-0 hover:bg-bg-elevated/60"
                  >
                    <span class="tabular-nums text-text-muted">{{ formatTime(log.timestamp) }}</span>
                    <span>
                      <span
                        class="rounded-full px-2 py-0.5 text-[10px] font-bold uppercase"
                        :class="getLevelClass(log.level)"
                      >
                        {{ log.level }}
                      </span>
                    </span>
                    <span
                      class="truncate text-text-muted"
                      :title="log.channel"
                    >{{ log.channel }}</span>
                    <span
                      class="truncate text-text-muted"
                      :title="log.source"
                    >{{ log.source }}</span>
                    <span
                      class="min-w-0 break-words leading-5 text-text-secondary line-clamp-2"
                      :title="log.message"
                    >{{ log.message }}</span>
                  </div>

                  <div
                    v-if="logs.length === 0"
                    class="flex min-h-[440px] flex-col items-center justify-center px-6 text-center text-text-muted"
                  >
                    <SIcon
                      name="Monitor"
                      size="w-10 h-10"
                      class="mb-3 opacity-35"
                    />
                    <p class="text-sm font-medium text-text-secondary">
                      {{ t('monitoring.noLogs') }}
                    </p>
                    <p class="mt-1 max-w-sm text-xs leading-5">
                      {{ t('monitoring.waitingForLogs') }}
                    </p>
                  </div>

                  <div
                    v-else-if="filteredLogs.length === 0"
                    class="flex min-h-[440px] flex-col items-center justify-center px-6 text-center text-text-muted"
                  >
                    <SIcon
                      name="Filter"
                      size="w-10 h-10"
                      class="mb-3 opacity-35"
                    />
                    <p class="text-sm font-medium text-text-secondary">
                      {{ t('monitoring.noFilteredLogs') }}
                    </p>
                    <p class="mt-1 max-w-sm text-xs leading-5">
                      {{ t('monitoring.adjustFilter') }}
                    </p>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </section>
      </div>
    </section>
  </main>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { getUsageCapabilitiesV2, getUsageSummaryV2 } from '@/api'
import {
  useMonitoringFeed,
  type MonitoringEntry,
  type MonitoringLevel,
} from '@/composables/useMonitoringFeed'
import { translateWithFallback } from '@/i18n/formatMessage'
import { isTauriRuntime } from '@/utils/tauriRuntime'
import type { UsageCapabilityReport, UsageFeatureCapability, UsageSummary } from '@/types/usage'

const { t, locale } = useI18n()
const { isConnected, logs, clearLogs, refresh } = useMonitoringFeed()

const monitoringLevels: MonitoringLevel[] = ['error', 'warn', 'info', 'debug']
const filterLevel = ref<'all' | MonitoringLevel>('all')
const logContainer = ref<HTMLElement | null>(null)

const usageSummary = ref<UsageSummary | null>(null)
const usageLoading = ref(false)
const usageStatus = ref<'idle' | 'ready' | 'unavailable'>('idle')
const usageUnavailableDetail = ref('')
const usageUpdatedAt = ref<string | null>(null)

const currentLocale = computed(() => locale.value === 'en-US' ? 'en-US' : 'zh-CN')

const filteredLogs = computed(() => {
  if (filterLevel.value === 'all') return logs.value
  return logs.value.filter((log: MonitoringEntry) => log.level === filterLevel.value)
})

const filteredCountLabel = computed(() => translateWithFallback(
  t,
  'monitoring.filteredCount',
  '{filtered} / {count} events',
  { filtered: filteredLogs.value.length, count: logs.value.length },
))

const levelCounts = computed<Record<MonitoringLevel, number>>(() => {
  const counts: Record<MonitoringLevel, number> = {
    error: 0,
    warn: 0,
    info: 0,
    debug: 0,
  }

  for (const log of logs.value) {
    counts[log.level] += 1
  }

  return counts
})

const recentIssueEvents = computed(() => {
  return logs.value
    .filter((log) => log.level === 'error' || log.level === 'warn')
    .slice(-4)
    .reverse()
})

const latestUsageEvent = computed(() => {
  return [...logs.value]
    .reverse()
    .find((log) => log.channel === 'usage' || log.eventType.includes('usage')) ?? null
})

const healthStatus = computed<'critical' | 'attention' | 'healthy' | 'quiet'>(() => {
  if (levelCounts.value.error > 0) return 'critical'
  if (levelCounts.value.warn > 0) return 'attention'
  if (logs.value.length > 0) return 'healthy'
  return 'quiet'
})

const healthStatusLabel = computed(() => {
  switch (healthStatus.value) {
    case 'critical':
      return t('monitoring.healthCritical')
    case 'attention':
      return t('monitoring.healthAttention')
    case 'healthy':
      return t('monitoring.healthHealthy')
    default:
      return t('monitoring.healthQuiet')
  }
})

const healthStatusClass = computed(() => {
  switch (healthStatus.value) {
    case 'critical':
      return 'border-accent-danger/30 bg-accent-danger/10 text-accent-danger'
    case 'attention':
      return 'border-accent-warning/30 bg-accent-warning/10 text-accent-warning'
    case 'healthy':
      return 'border-accent-success/30 bg-accent-success/10 text-accent-success'
    default:
      return 'border-border-default/45 bg-bg-surface/70 text-text-secondary'
  }
})

const usageUnavailable = computed(() => usageStatus.value === 'unavailable')

const usageStatusLabel = computed(() => {
  if (usageLoading.value) return t('monitoring.usageLoading')
  if (usageStatus.value === 'ready') return t('monitoring.usageReady')
  if (usageStatus.value === 'unavailable') return t('monitoring.usageUnavailable')
  return t('monitoring.usageIdle')
})

const usageStatusClass = computed(() => {
  if (usageLoading.value) return 'border-border-default/45 bg-bg-surface/70 text-text-secondary'
  if (usageStatus.value === 'ready') return 'border-accent-success/30 bg-accent-success/10 text-accent-success'
  if (usageStatus.value === 'unavailable') return 'border-accent-warning/30 bg-accent-warning/10 text-accent-warning'
  return 'border-border-default/45 bg-bg-surface/70 text-text-secondary'
})

const usageMetricValue = (value: number | null | undefined, formatter: (value: number) => string) => {
  if (usageLoading.value) return '…'
  if (usageStatus.value !== 'ready' || !usageSummary.value || value == null) return '—'
  return formatter(value)
}

const usageMetricDetail = (detail: string) => {
  if (usageLoading.value) return t('monitoring.usageLoadingDetail')
  if (usageStatus.value !== 'ready' || !usageSummary.value) return t('monitoring.usageMetricUnavailable')
  return detail
}

const usageMetricCards = computed(() => {
  const summary = usageSummary.value
  const inputOutputDetail = summary
    ? t('monitoring.inputOutputDetail', {
        input: formatCompactNumber(summary.total_input_tokens),
        output: formatCompactNumber(summary.total_output_tokens),
      })
    : ''

  return [
    {
      id: 'requests',
      label: t('monitoring.totalRequests'),
      value: usageMetricValue(summary?.total_requests, formatWholeNumber),
      detail: usageMetricDetail(t('monitoring.requestsDetail')),
      icon: 'Activity',
    },
    {
      id: 'tokens',
      label: t('monitoring.totalTokens'),
      value: usageMetricValue(summary?.total_tokens, formatCompactNumber),
      detail: usageMetricDetail(inputOutputDetail),
      icon: 'Layers',
    },
    {
      id: 'input-output',
      label: t('monitoring.inputOutputTokens'),
      value: usageMetricValue(
        summary ? summary.total_input_tokens + summary.total_output_tokens : null,
        formatCompactNumber,
      ),
      detail: usageMetricDetail(t('monitoring.cacheDetail', {
        cache: formatCompactNumber(summary?.total_cache_read_tokens ?? 0),
      })),
      icon: 'ArrowLeftRight',
    },
    {
      id: 'cost',
      label: t('monitoring.estimatedCost'),
      value: usageMetricValue(summary?.total_cost_usd, formatCostUsd),
      detail: usageMetricDetail(usageUpdatedAt.value
        ? t('monitoring.lastUpdated', { time: formatDateTime(usageUpdatedAt.value) })
        : t('monitoring.notUpdated')),
      icon: 'Wallet',
    },
  ]
})

const toErrorMessage = (error: unknown) => {
  if (error instanceof Error) return error.message
  return String(error)
}

const buildCapabilityUnavailableDetail = (capability: UsageFeatureCapability) => {
  if (capability.detail) return capability.detail
  if (capability.reason) {
    return t('monitoring.usageUnsupportedReason', { reason: capability.reason })
  }
  return t('monitoring.usageUnavailableDescription')
}

const overviewCapability = (report: UsageCapabilityReport | null | undefined) => {
  return report?.features.overview ?? null
}

const loadUsageSummary = async () => {
  usageLoading.value = true
  usageUnavailableDetail.value = ''

  if (!isTauriRuntime()) {
    usageSummary.value = null
    usageStatus.value = 'unavailable'
    usageUnavailableDetail.value = t('monitoring.usageUnavailableDescription')
    usageUpdatedAt.value = new Date().toISOString()
    usageLoading.value = false
    return
  }

  let capabilityError: unknown = null

  try {
    const capabilities = await getUsageCapabilitiesV2<UsageCapabilityReport>()
    const capability = overviewCapability(capabilities)
    if (capability && !capability.supported) {
      usageSummary.value = null
      usageStatus.value = 'unavailable'
      usageUnavailableDetail.value = buildCapabilityUnavailableDetail(capability)
      usageUpdatedAt.value = new Date().toISOString()
      usageLoading.value = false
      return
    }
  } catch (error) {
    capabilityError = error
  }

  try {
    const summary = await getUsageSummaryV2<UsageSummary>()
    if (!summary || typeof summary.total_requests !== 'number') {
      throw new Error('Invalid usage summary payload')
    }

    usageSummary.value = summary
    usageStatus.value = 'ready'
    usageUnavailableDetail.value = ''
    usageUpdatedAt.value = new Date().toISOString()
  } catch (error) {
    usageSummary.value = null
    usageStatus.value = 'unavailable'
    usageUnavailableDetail.value = capabilityError
      ? `${toErrorMessage(capabilityError)} · ${toErrorMessage(error)}`
      : toErrorMessage(error)
    usageUpdatedAt.value = new Date().toISOString()
  } finally {
    usageLoading.value = false
  }
}

const refreshMonitoring = async () => {
  await Promise.all([
    refresh(),
    loadUsageSummary(),
  ])
}

const clearLocalLogs = () => {
  clearLogs()
}

const isNearBottom = (element: HTMLElement) => {
  return element.scrollHeight - element.scrollTop - element.clientHeight < 24
}

const scrollToBottom = async () => {
  await nextTick()
  if (logContainer.value) {
    logContainer.value.scrollTop = logContainer.value.scrollHeight
  }
}

watch(
  () => logs.value.length,
  async (currentLength, previousLength) => {
    if (currentLength <= previousLength) {
      return
    }

    const container = logContainer.value
    if (!container || previousLength === 0 || isNearBottom(container)) {
      await scrollToBottom()
    }
  }
)

watch(filterLevel, async () => {
  await scrollToBottom()
})

onMounted(() => {
  void loadUsageSummary()
})

function formatWholeNumber(value: number) {
  return new Intl.NumberFormat(currentLocale.value).format(value)
}

function formatCompactNumber(value: number) {
  if (Math.abs(value) >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`
  if (Math.abs(value) >= 1_000) return `${(value / 1_000).toFixed(1)}K`
  return formatWholeNumber(value)
}

function formatCostUsd(value: number) {
  return `$${value.toFixed(value >= 100 ? 2 : 4)}`
}

function formatDateTime(timestamp: string) {
  const date = new Date(timestamp)
  if (Number.isNaN(date.getTime())) return timestamp

  return new Intl.DateTimeFormat(currentLocale.value, {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  }).format(date)
}

function formatTime(timestamp: string) {
  const date = new Date(timestamp)
  if (Number.isNaN(date.getTime())) return '--:--:--'

  return new Intl.DateTimeFormat(currentLocale.value, {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
  }).format(date)
}

function getLevelLabel(level: MonitoringLevel) {
  return t(`monitoring.levels.${level}`)
}

function getLevelClass(level: MonitoringLevel) {
  switch (level) {
    case 'error':
      return 'bg-accent-danger/15 text-accent-danger'
    case 'warn':
      return 'bg-accent-warning/15 text-accent-warning'
    case 'info':
      return 'bg-accent-primary/15 text-accent-primary'
    default:
      return 'bg-text-muted/15 text-text-muted'
  }
}

function getLevelDotClass(level: MonitoringLevel) {
  switch (level) {
    case 'error':
      return 'bg-accent-danger'
    case 'warn':
      return 'bg-accent-warning'
    case 'info':
      return 'bg-accent-primary'
    default:
      return 'bg-text-muted'
  }
}
</script>
