<template>
  <div class="stats-view min-h-full space-y-6 p-4 sm:p-6">
    <div class="glass-effect rounded-3xl border border-white/20 p-5 shadow-sm sm:p-6">
      <div class="flex flex-col gap-4 xl:flex-row xl:items-start xl:justify-between">
        <div class="space-y-2">
          <div class="flex items-center gap-3">
            <div class="flex h-11 w-11 items-center justify-center rounded-2xl border border-accent-primary/20 bg-gradient-to-br from-violet-500/20 to-fuchsia-500/20 text-accent-primary">
              <svg
                class="h-6 w-6"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
                />
              </svg>
            </div>
            <div>
              <h1 class="text-2xl font-bold text-text-primary">
                {{ $t('stats.title') }}
              </h1>
              <p class="mt-1 text-sm text-text-secondary">
                {{ $t('stats.subtitle') }}
              </p>
            </div>
          </div>
        </div>

        <div class="flex flex-col gap-3 sm:flex-row sm:flex-wrap xl:justify-end">
          <select
            v-model="selectedRange"
            class="min-h-[44px] rounded-xl border border-border-default bg-bg-surface px-4 py-2.5 text-sm font-medium text-text-primary shadow-sm transition-[border-color,box-shadow] hover:border-accent-primary/40 focus:outline-none focus:ring-2 focus:ring-accent-primary/20"
            :aria-label="$t('stats.actions.refresh')"
            @change="loadData"
          >
            <option value="today">
              {{ $t('stats.timeRange.today') }}
            </option>
            <option value="week">
              {{ $t('stats.timeRange.thisWeek') }}
            </option>
            <option value="month">
              {{ $t('stats.timeRange.thisMonth') }}
            </option>
          </select>

          <button
            type="button"
            class="inline-flex min-h-[44px] items-center justify-center gap-2 rounded-xl border border-accent-primary/20 bg-accent-primary/10 px-4 py-2.5 text-sm font-medium text-accent-primary transition-colors hover:bg-accent-primary/15 focus:outline-none focus:ring-2 focus:ring-accent-primary/20"
            @click="showProvidersModal = true"
          >
            <svg
              class="h-4 w-4"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M4 6h16M4 10h16M4 14h16M4 18h16"
              />
            </svg>
            <span>{{ $t('stats.actions.providersStats') }}</span>
          </button>

          <button
            :disabled="loading"
            type="button"
            class="inline-flex min-h-[44px] items-center justify-center gap-2 rounded-xl bg-gradient-to-r from-violet-500 to-purple-600 px-4 py-2.5 text-sm font-semibold text-white shadow-lg shadow-violet-500/25 transition-[color,background-color,border-color,transform] hover:-translate-y-0.5 hover:shadow-violet-500/35 focus:outline-none focus:ring-2 focus:ring-accent-primary/30 disabled:cursor-not-allowed disabled:opacity-50"
            @click="loadData"
          >
            <svg
              class="h-4 w-4"
              :class="{ 'animate-spin': loading }"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
              />
            </svg>
            <span>{{ $t('stats.actions.refresh') }}</span>
          </button>
        </div>
      </div>
    </div>

    <div
      v-if="loading"
      class="glass-effect flex items-center justify-center rounded-3xl border border-white/20 py-16"
      aria-live="polite"
    >
      <div class="flex flex-col items-center gap-4">
        <div class="h-12 w-12 animate-spin rounded-full border-4 border-accent-primary/15 border-t-accent-primary" />
        <p class="text-sm text-text-secondary">
          {{ $t('stats.states.loading', '加载中...') }}
        </p>
      </div>
    </div>

    <div
      v-if="error"
      class="rounded-2xl border border-red-500/30 bg-red-500/10 p-6 backdrop-blur-md"
      role="alert"
    >
      <div class="flex items-start gap-4">
        <div class="rounded-xl bg-red-500/15 p-2">
          <svg
            class="h-5 w-5 text-red-300"
            fill="currentColor"
            viewBox="0 0 20 20"
          >
            <path
              fill-rule="evenodd"
              d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
              clip-rule="evenodd"
            />
          </svg>
        </div>
        <div>
          <h2 class="text-sm font-semibold text-red-100">
            {{ $t('stats.states.loadFailed') }}
          </h2>
          <p class="mt-1 text-sm text-red-100/85">
            {{ error }}
          </p>
        </div>
      </div>
    </div>

    <div
      v-if="!loading && !error && stats"
      class="space-y-6"
    >
      <div class="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-4">
        <article class="glass-effect rounded-3xl border border-white/20 p-5 shadow-sm">
          <div class="flex items-start justify-between gap-4">
            <div>
              <p class="text-xs font-medium uppercase tracking-[0.24em] text-text-muted">
                {{ $t('stats.summaryCards.totalCost') }}
              </p>
              <p class="mt-2 text-2xl font-bold text-text-primary">
                ${{ formatCost(stats.total_cost) }}
              </p>
            </div>
            <div class="flex h-12 w-12 items-center justify-center rounded-2xl border border-accent-success/20 bg-accent-success/10 text-accent-success">
              <svg
                class="h-6 w-6"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              </svg>
            </div>
          </div>
        </article>

        <article class="glass-effect rounded-3xl border border-white/20 p-5 shadow-sm">
          <div class="flex items-start justify-between gap-4">
            <div>
              <p class="text-xs font-medium uppercase tracking-[0.24em] text-text-muted">
                {{ $t('stats.summaryCards.apiCalls') }}
              </p>
              <p class="mt-2 text-2xl font-bold text-text-primary">
                {{ stats.record_count }}
              </p>
            </div>
            <div class="flex h-12 w-12 items-center justify-center rounded-2xl border border-accent-primary/20 bg-accent-primary/10 text-accent-primary">
              <svg
                class="h-6 w-6"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
                />
              </svg>
            </div>
          </div>
        </article>

        <article class="glass-effect rounded-3xl border border-white/20 p-5 shadow-sm">
          <div class="flex items-start justify-between gap-4">
            <div>
              <p class="text-xs font-medium uppercase tracking-[0.24em] text-text-muted">
                {{ $t('stats.summaryCards.inputToken') }}
              </p>
              <p class="mt-2 text-2xl font-bold text-text-primary">
                {{ formatNumber(stats.token_stats?.total_input_tokens ?? 0) }}
              </p>
            </div>
            <div class="flex h-12 w-12 items-center justify-center rounded-2xl border border-accent-primary/20 bg-accent-primary/10 text-accent-primary">
              <svg
                class="h-6 w-6"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M9 19l3 3m0 0l3-3m-3 3V10"
                />
              </svg>
            </div>
          </div>
        </article>

        <article class="glass-effect rounded-3xl border border-white/20 p-5 shadow-sm">
          <div class="flex items-start justify-between gap-4">
            <div>
              <p class="text-xs font-medium uppercase tracking-[0.24em] text-text-muted">
                {{ $t('stats.summaryCards.outputToken') }}
              </p>
              <p class="mt-2 text-2xl font-bold text-text-primary">
                {{ formatNumber(stats.token_stats?.total_output_tokens ?? 0) }}
              </p>
            </div>
            <div class="flex h-12 w-12 items-center justify-center rounded-2xl border border-fuchsia-400/20 bg-fuchsia-500/10 text-fuchsia-300">
              <svg
                class="h-6 w-6"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"
                />
              </svg>
            </div>
          </div>
        </article>
      </div>

      <section class="glass-effect rounded-3xl border border-white/20 p-6 shadow-sm">
        <h2 class="mb-4 flex items-center gap-2 text-lg font-bold text-text-primary">
          <svg
            class="h-5 w-5 text-accent-primary"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M13 10V3L4 14h7v7l9-11h-7z"
            />
          </svg>
          {{ $t('stats.tokenDetails.title') }}
        </h2>
        <div class="grid grid-cols-1 gap-4 md:grid-cols-3">
          <div class="rounded-2xl border border-white/10 bg-bg-surface/60 p-4">
            <p class="text-sm text-text-secondary">
              {{ $t('stats.tokenDetails.cacheToken') }}
            </p>
            <p class="mt-1 text-xl font-bold text-text-primary">
              {{ formatNumber(stats.token_stats?.total_cache_tokens ?? 0) }}
            </p>
          </div>
          <div class="rounded-2xl border border-white/10 bg-bg-surface/60 p-4">
            <p class="text-sm text-text-secondary">
              {{ $t('stats.tokenDetails.cacheEfficiency') }}
            </p>
            <p class="mt-1 text-xl font-bold text-text-primary">
              {{ formatPercent(stats.token_stats?.cache_efficiency ?? 0) }}%
            </p>
          </div>
          <div class="rounded-2xl border border-white/10 bg-bg-surface/60 p-4">
            <p class="text-sm text-text-secondary">
              {{ $t('stats.tokenDetails.totalToken') }}
            </p>
            <p class="mt-1 text-xl font-bold text-text-primary">
              {{ formatNumber(getTotalTokens()) }}
            </p>
          </div>
        </div>
      </section>

      <div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
        <section class="glass-effect rounded-3xl border border-white/20 p-6 shadow-sm">
          <h2 class="mb-4 flex items-center gap-2 text-lg font-bold text-text-primary">
            <svg
              class="h-5 w-5 text-accent-primary"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
              />
            </svg>
            {{ $t('stats.sections.byModel') }}
          </h2>
          <div class="scrollbar-thin space-y-2 max-h-64 overflow-y-auto pr-1">
            <div
              v-for="[model, cost] in sortedModels"
              :key="model"
              class="flex items-center justify-between gap-3 rounded-2xl border border-white/10 bg-bg-surface/50 p-3 transition-colors hover:bg-bg-elevated/70"
            >
              <span class="text-sm font-medium text-text-secondary">{{ shortenModelName(model) }}</span>
              <span class="text-sm font-bold text-text-primary">${{ formatCost(cost) }}</span>
            </div>
            <div
              v-if="Object.keys(stats.by_model || {}).length === 0"
              class="py-8 text-center text-sm text-text-muted"
            >
              {{ $t('stats.states.noData') }}
            </div>
          </div>
        </section>

        <section class="glass-effect rounded-3xl border border-white/20 p-6 shadow-sm">
          <h2 class="mb-4 flex items-center gap-2 text-lg font-bold text-text-primary">
            <svg
              class="h-5 w-5 text-accent-success"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"
              />
            </svg>
            {{ $t('stats.sections.byProject') }}
          </h2>
          <div class="scrollbar-thin space-y-2 max-h-64 overflow-y-auto pr-1">
            <div
              v-for="[project, cost] in sortedProjects.slice(0, 10)"
              :key="project"
              class="flex items-center justify-between gap-3 rounded-2xl border border-white/10 bg-bg-surface/50 p-3 transition-colors hover:bg-bg-elevated/70"
            >
              <span class="mr-4 flex-1 truncate text-sm font-medium text-text-secondary">{{ shortenPath(project) }}</span>
              <span class="text-sm font-bold text-text-primary">${{ formatCost(cost) }}</span>
            </div>
            <div
              v-if="Object.keys(stats.by_project || {}).length === 0"
              class="py-8 text-center text-sm text-text-muted"
            >
              {{ $t('stats.states.noData') }}
            </div>
          </div>
        </section>
      </div>

      <section
        v-if="stats.trend && stats.trend.length > 0"
        class="glass-effect rounded-3xl border border-white/20 p-6 shadow-sm"
      >
        <h2 class="mb-4 flex items-center gap-2 text-lg font-bold text-text-primary">
          <svg
            class="h-5 w-5 text-fuchsia-300"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M13 7h8m0 0v8m0-8l-8 8-4-4-6 6"
            />
          </svg>
          {{ $t('stats.sections.costTrend') }}
        </h2>
        <div class="grid grid-cols-2 gap-3 sm:grid-cols-4 xl:grid-cols-7">
          <div
            v-for="daily in stats.trend.slice().reverse().slice(0, 7).reverse()"
            :key="daily.date"
            class="rounded-2xl border border-white/10 bg-bg-surface/60 p-3 text-center"
          >
            <p class="text-xs font-medium text-text-muted">
              {{ daily.date }}
            </p>
            <p class="mt-1 text-lg font-bold text-text-primary">
              ${{ formatCost(daily.cost) }}
            </p>
            <p class="text-xs text-text-secondary">
              {{ daily.count }} {{ $t('stats.units.times') }}
            </p>
          </div>
        </div>
      </section>
    </div>

    <div
      v-if="!loading && !error && stats && stats.record_count === 0"
      class="glass-effect rounded-3xl border border-white/20 p-10 text-center shadow-sm sm:p-16"
    >
      <div class="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full border border-white/10 bg-bg-surface/70">
        <svg
          class="h-8 w-8 text-text-muted"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M9 17v-2m3 2v-4m3 4v-6m2 10H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
          />
        </svg>
      </div>
      <h3 class="text-lg font-semibold text-text-primary">
        {{ $t('stats.states.noStatsData') }}
      </h3>
      <p class="mt-2 text-sm text-text-secondary">
        {{ $t('stats.states.noStatsHint') }}
      </p>
    </div>

    <BaseModal
      v-model="showProvidersModal"
      :title="$t('stats.sections.providerUsage')"
      :description="$t('stats.sections.providerUsageSubtitle')"
      size="xl"
      surface="solid"
      content-class="stats-provider-modal"
    >
      <div class="space-y-4 py-2">
        <p class="text-sm text-text-secondary">
          {{ $t('stats.sections.providerUsageSubtitle') }}
        </p>
        <div class="scrollbar-thin space-y-4 max-h-[60vh] overflow-y-auto pr-1">
          <div
            v-for="[provider, count] in sortedProviders"
            :key="provider"
            class="space-y-2 rounded-2xl border border-white/10 bg-bg-surface/50 p-4"
          >
            <div class="flex items-center justify-between gap-3 text-sm">
              <span class="truncate font-medium text-text-primary">{{ provider || 'unknown' }}</span>
              <span class="font-bold text-accent-primary">{{ count }} {{ $t('stats.units.times') }}</span>
            </div>
            <div class="h-2 w-full overflow-hidden rounded-full bg-bg-surface">
              <div
                class="h-full rounded-full bg-gradient-to-r from-violet-500 to-fuchsia-500 transition-[width]"
                :style="{ width: `${getProviderBarWidth(count)}%` }"
              />
            </div>
          </div>
          <div
            v-if="sortedProviders.length === 0"
            class="py-8 text-center text-sm text-text-muted"
          >
            {{ $t('stats.states.noData') }}
          </div>
        </div>
      </div>
    </BaseModal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import BaseModal from '@/components/common/BaseModal.vue'
import { getCostOverview, getProviderUsage } from '@/api'
import type { CostStats } from '@/types'
import { logger } from '@/utils/logger'

const { t } = useI18n()

const stats = ref<CostStats | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)
const selectedRange = ref('today')
const showProvidersModal = ref(false)
const providerUsage = ref<Record<string, number>>({})

const loadData = async () => {
  loading.value = true
  error.value = null

  try {
    const [statsData, providerData] = await Promise.all([
      getCostOverview<CostStats>(selectedRange.value),
      getProviderUsage<Record<string, number>>(),
    ])
    stats.value = statsData ?? null
    providerUsage.value = providerData ?? {}
  } catch (e: unknown) {
    const errorMessage = e instanceof Error ? e.message : t('stats.states.loadFailedMessage')
    error.value = errorMessage
    logger.error('Failed to load stats:', e)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  loadData()
})

const sortedModels = computed(() => {
  if (!stats.value) return []
  return Object.entries(stats.value.by_model || {}).sort((a, b) => b[1] - a[1])
})

const sortedProviders = computed(() => {
  return Object.entries(providerUsage.value || {}).sort((a, b) => b[1] - a[1])
})

const maxProviderCount = computed(() => {
  const values = Object.values(providerUsage.value || {})
  return values.length ? Math.max(...values) : 0
})

const sortedProjects = computed(() => {
  if (!stats.value) return []
  return Object.entries(stats.value.by_project || {}).sort((a, b) => b[1] - a[1])
})

const formatCost = (cost: number): string => {
  return cost.toFixed(4)
}

const formatNumber = (num: number): string => {
  if (num >= 1000000) return (num / 1000000).toFixed(1) + 'M'
  if (num >= 1000) return (num / 1000).toFixed(1) + 'K'
  return num.toString()
}

const formatPercent = (num: number): string => {
  return num.toFixed(2)
}

const getTotalTokens = (): number => {
  if (!stats.value?.token_stats) return 0
  return (
    stats.value.token_stats.total_input_tokens +
    stats.value.token_stats.total_output_tokens +
    stats.value.token_stats.total_cache_tokens
  )
}

const shortenModelName = (model: string): string => {
  return model.replace('claude-', '').replace('-20241022', '').replace('-20240229', '')
}

const shortenPath = (path: string): string => {
  const parts = path.split('/')
  return parts[parts.length - 1] || path
}

const getProviderBarWidth = (count: number): number => {
  const max = maxProviderCount.value || 1
  return Math.min(100, (count / max) * 100)
}
</script>

<style scoped>
.stats-view {
  min-height: calc(100vh - 64px);
}

:deep(.stats-provider-modal) {
  max-width: min(52rem, calc(100vw - 2rem));
}

.scrollbar-thin::-webkit-scrollbar {
  width: 4px;
}

.scrollbar-thin::-webkit-scrollbar-track {
  background: transparent;
}

.scrollbar-thin::-webkit-scrollbar-thumb {
  background-color: rgb(168 85 247 / 45%);
  border-radius: 9999px;
}

.scrollbar-thin::-webkit-scrollbar-thumb:hover {
  background-color: rgb(168 85 247 / 70%);
}
</style>
