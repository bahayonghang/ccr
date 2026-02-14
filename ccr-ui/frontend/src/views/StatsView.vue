<template>
  <div class="stats-view p-6 space-y-6 min-h-screen bg-gradient-to-br from-gray-50 to-gray-100 dark:from-gray-900 dark:to-gray-950">
    <!-- Page Header -->
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
      <div>
        <h1 class="text-2xl font-bold text-gray-900 dark:text-white flex items-center gap-3">
          <div class="p-2 rounded-xl bg-gradient-to-br from-orange-500 to-red-500 text-white shadow-lg">
            <svg
              class="w-6 h-6"
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
          {{ $t('stats.title') }}
        </h1>
        <p class="mt-1 text-sm text-gray-600 dark:text-gray-400">
          {{ $t('stats.subtitle') }}
        </p>
      </div>
      <div class="flex items-center gap-3">
        <!-- Time Range Selector -->
        <select
          v-model="selectedRange"
          class="px-4 py-2.5 border border-gray-200 dark:border-gray-600 rounded-xl bg-white/80 dark:bg-gray-800/80 text-gray-900 dark:text-white text-sm font-medium shadow-sm hover:border-orange-300 dark:hover:border-orange-600 focus:ring-2 focus:ring-orange-500/20 focus:border-orange-500 transition-all cursor-pointer"
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

        <!-- Provider Stats Button -->
        <button
          class="px-4 py-2.5 border border-orange-200 dark:border-orange-700 text-orange-600 dark:text-orange-300 rounded-xl flex items-center gap-2 hover:bg-orange-50 dark:hover:bg-orange-900/30 font-medium text-sm transition-all shadow-sm"
          @click="showProvidersModal = true"
        >
          <svg
            class="w-4 h-4"
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

        <!-- Refresh Button -->
        <button
          :disabled="loading"
          class="px-4 py-2.5 bg-gradient-to-r from-orange-500 to-red-500 hover:from-orange-600 hover:to-red-600 text-white rounded-xl flex items-center gap-2 disabled:opacity-50 font-medium text-sm shadow-lg shadow-orange-500/25 transition-all"
          @click="loadData"
        >
          <svg
            class="w-4 h-4"
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

    <!-- Loading State -->
    <div
      v-if="loading"
      class="flex items-center justify-center py-16"
    >
      <div class="flex flex-col items-center gap-4">
        <div class="animate-spin rounded-full h-12 w-12 border-4 border-orange-200 border-t-orange-500" />
        <p class="text-sm text-gray-500 dark:text-gray-400">
          {{ $t('stats.states.loading', '加载中...') }}
        </p>
      </div>
    </div>

    <!-- Error State -->
    <div
      v-if="error"
      class="rounded-2xl bg-red-50/80 dark:bg-red-900/20 border border-red-200 dark:border-red-800 p-6 backdrop-blur-sm"
    >
      <div class="flex items-start gap-4">
        <div class="p-2 rounded-lg bg-red-100 dark:bg-red-800/30">
          <svg
            class="h-5 w-5 text-red-500"
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
          <h3 class="text-sm font-semibold text-red-800 dark:text-red-200">
            {{ $t('stats.states.loadFailed') }}
          </h3>
          <p class="mt-1 text-sm text-red-700 dark:text-red-300">
            {{ error }}
          </p>
        </div>
      </div>
    </div>

    <!-- Stats Content -->
    <div
      v-if="!loading && !error && stats"
      class="space-y-6"
    >
      <!-- Overview Cards Grid -->
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <!-- Total Cost -->
        <div class="p-5 rounded-2xl bg-white/80 dark:bg-gray-800/80 border border-gray-200/50 dark:border-gray-700/50 shadow-sm backdrop-blur-sm hover:shadow-md transition-shadow">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                {{ $t('stats.summaryCards.totalCost') }}
              </p>
              <p class="mt-2 text-2xl font-bold text-gray-900 dark:text-white">
                ${{ formatCost(stats.total_cost) }}
              </p>
            </div>
            <div class="p-3 rounded-xl bg-gradient-to-br from-green-100 to-emerald-100 dark:from-green-900/30 dark:to-emerald-900/30">
              <svg
                class="w-6 h-6 text-green-600 dark:text-green-400"
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
        </div>

        <!-- Record Count -->
        <div class="p-5 rounded-2xl bg-white/80 dark:bg-gray-800/80 border border-gray-200/50 dark:border-gray-700/50 shadow-sm backdrop-blur-sm hover:shadow-md transition-shadow">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                {{ $t('stats.summaryCards.apiCalls') }}
              </p>
              <p class="mt-2 text-2xl font-bold text-gray-900 dark:text-white">
                {{ stats.record_count }}
              </p>
            </div>
            <div class="p-3 rounded-xl bg-gradient-to-br from-blue-100 to-indigo-100 dark:from-blue-900/30 dark:to-indigo-900/30">
              <svg
                class="w-6 h-6 text-blue-600 dark:text-blue-400"
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
        </div>

        <!-- Input Tokens -->
        <div class="p-5 rounded-2xl bg-white/80 dark:bg-gray-800/80 border border-gray-200/50 dark:border-gray-700/50 shadow-sm backdrop-blur-sm hover:shadow-md transition-shadow">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                {{ $t('stats.summaryCards.inputToken') }}
              </p>
              <p class="mt-2 text-2xl font-bold text-gray-900 dark:text-white">
                {{ formatNumber(stats.token_stats.total_input_tokens) }}
              </p>
            </div>
            <div class="p-3 rounded-xl bg-gradient-to-br from-purple-100 to-violet-100 dark:from-purple-900/30 dark:to-violet-900/30">
              <svg
                class="w-6 h-6 text-purple-600 dark:text-purple-400"
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
        </div>

        <!-- Output Tokens -->
        <div class="p-5 rounded-2xl bg-white/80 dark:bg-gray-800/80 border border-gray-200/50 dark:border-gray-700/50 shadow-sm backdrop-blur-sm hover:shadow-md transition-shadow">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                {{ $t('stats.summaryCards.outputToken') }}
              </p>
              <p class="mt-2 text-2xl font-bold text-gray-900 dark:text-white">
                {{ formatNumber(stats.token_stats.total_output_tokens) }}
              </p>
            </div>
            <div class="p-3 rounded-xl bg-gradient-to-br from-orange-100 to-amber-100 dark:from-orange-900/30 dark:to-amber-900/30">
              <svg
                class="w-6 h-6 text-orange-600 dark:text-orange-400"
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
        </div>
      </div>

      <!-- Token Details Card -->
      <div class="p-6 rounded-2xl bg-white/80 dark:bg-gray-800/80 border border-gray-200/50 dark:border-gray-700/50 shadow-sm backdrop-blur-sm">
        <h2 class="text-lg font-bold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
          <svg
            class="w-5 h-5 text-orange-500"
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
        <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div class="p-4 rounded-xl bg-gray-50/50 dark:bg-gray-700/30">
            <p class="text-sm text-gray-600 dark:text-gray-400">
              {{ $t('stats.tokenDetails.cacheToken') }}
            </p>
            <p class="text-xl font-bold text-gray-900 dark:text-white mt-1">
              {{ formatNumber(stats.token_stats.total_cache_tokens) }}
            </p>
          </div>
          <div class="p-4 rounded-xl bg-gray-50/50 dark:bg-gray-700/30">
            <p class="text-sm text-gray-600 dark:text-gray-400">
              {{ $t('stats.tokenDetails.cacheEfficiency') }}
            </p>
            <p class="text-xl font-bold text-gray-900 dark:text-white mt-1">
              {{ formatPercent(stats.token_stats.cache_efficiency) }}%
            </p>
          </div>
          <div class="p-4 rounded-xl bg-gray-50/50 dark:bg-gray-700/30">
            <p class="text-sm text-gray-600 dark:text-gray-400">
              {{ $t('stats.tokenDetails.totalToken') }}
            </p>
            <p class="text-xl font-bold text-gray-900 dark:text-white mt-1">
              {{ formatNumber(getTotalTokens()) }}
            </p>
          </div>
        </div>
      </div>

      <!-- Two Column Layout: By Model & By Project -->
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <!-- By Model -->
        <div class="p-6 rounded-2xl bg-white/80 dark:bg-gray-800/80 border border-gray-200/50 dark:border-gray-700/50 shadow-sm backdrop-blur-sm">
          <h2 class="text-lg font-bold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
            <svg
              class="w-5 h-5 text-blue-500"
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
          <div class="space-y-2 max-h-64 overflow-y-auto scrollbar-thin">
            <div
              v-for="[model, cost] in sortedModels"
              :key="model"
              class="flex items-center justify-between p-3 rounded-xl bg-gray-50/50 dark:bg-gray-700/30 hover:bg-gray-100/50 dark:hover:bg-gray-600/30 transition-colors"
            >
              <span class="text-sm font-medium text-gray-700 dark:text-gray-300">{{ shortenModelName(model) }}</span>
              <span class="text-sm font-bold text-gray-900 dark:text-white">${{ formatCost(cost) }}</span>
            </div>
            <div
              v-if="Object.keys(stats.by_model).length === 0"
              class="text-center text-gray-500 dark:text-gray-400 py-8"
            >
              {{ $t('stats.states.noData') }}
            </div>
          </div>
        </div>

        <!-- By Project -->
        <div class="p-6 rounded-2xl bg-white/80 dark:bg-gray-800/80 border border-gray-200/50 dark:border-gray-700/50 shadow-sm backdrop-blur-sm">
          <h2 class="text-lg font-bold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
            <svg
              class="w-5 h-5 text-green-500"
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
          <div class="space-y-2 max-h-64 overflow-y-auto scrollbar-thin">
            <div
              v-for="[project, cost] in sortedProjects.slice(0, 10)"
              :key="project"
              class="flex items-center justify-between p-3 rounded-xl bg-gray-50/50 dark:bg-gray-700/30 hover:bg-gray-100/50 dark:hover:bg-gray-600/30 transition-colors"
            >
              <span class="text-sm font-medium text-gray-700 dark:text-gray-300 truncate flex-1 mr-4">{{ shortenPath(project) }}</span>
              <span class="text-sm font-bold text-gray-900 dark:text-white">${{ formatCost(cost) }}</span>
            </div>
            <div
              v-if="Object.keys(stats.by_project).length === 0"
              class="text-center text-gray-500 dark:text-gray-400 py-8"
            >
              {{ $t('stats.states.noData') }}
            </div>
          </div>
        </div>
      </div>

      <!-- Cost Trend -->
      <div
        v-if="stats.trend && stats.trend.length > 0"
        class="p-6 rounded-2xl bg-white/80 dark:bg-gray-800/80 border border-gray-200/50 dark:border-gray-700/50 shadow-sm backdrop-blur-sm"
      >
        <h2 class="text-lg font-bold text-gray-900 dark:text-white mb-4 flex items-center gap-2">
          <svg
            class="w-5 h-5 text-orange-500"
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
        <div class="grid grid-cols-2 sm:grid-cols-4 lg:grid-cols-7 gap-3">
          <div
            v-for="daily in stats.trend.slice().reverse().slice(0, 7).reverse()"
            :key="daily.date"
            class="p-3 rounded-xl bg-gradient-to-br from-gray-50/50 to-gray-100/50 dark:from-gray-700/30 dark:to-gray-600/30 border border-gray-200/30 dark:border-gray-600/30 text-center"
          >
            <p class="text-xs font-medium text-gray-500 dark:text-gray-400">
              {{ daily.date }}
            </p>
            <p class="text-lg font-bold text-gray-900 dark:text-white mt-1">
              ${{ formatCost(daily.cost) }}
            </p>
            <p class="text-xs text-gray-500 dark:text-gray-400">
              {{ daily.count }} {{ $t('stats.units.times') }}
            </p>
          </div>
        </div>
      </div>
    </div>

    <!-- Empty State -->
    <div
      v-if="!loading && !error && stats && stats.record_count === 0"
      class="p-16 rounded-2xl bg-white/80 dark:bg-gray-800/80 border border-gray-200/50 dark:border-gray-700/50 shadow-sm backdrop-blur-sm text-center"
    >
      <div class="p-4 rounded-full bg-gray-100 dark:bg-gray-700 w-16 h-16 mx-auto flex items-center justify-center mb-4">
        <svg
          class="h-8 w-8 text-gray-400"
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
      <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
        {{ $t('stats.states.noStatsData') }}
      </h3>
      <p class="mt-2 text-sm text-gray-500 dark:text-gray-400">
        {{ $t('stats.states.noStatsHint') }}
      </p>
    </div>

    <!-- Providers Modal -->
    <Teleport to="body">
      <div
        v-if="showProvidersModal"
        class="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50"
        @click.self="showProvidersModal = false"
      >
        <div class="w-full max-w-xl bg-white dark:bg-gray-800 rounded-2xl p-6 shadow-2xl m-4 border border-gray-200/50 dark:border-gray-700/50">
          <div class="flex items-center justify-between mb-6">
            <div>
              <h3 class="text-xl font-bold text-gray-900 dark:text-white">
                {{ $t('stats.sections.providerUsage') }}
              </h3>
              <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
                {{ $t('stats.sections.providerUsageSubtitle') }}
              </p>
            </div>
            <button
              class="p-2 rounded-lg text-gray-400 hover:text-gray-600 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
              @click="showProvidersModal = false"
            >
              <svg
                class="w-5 h-5"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M6 18L18 6M6 6l12 12"
                />
              </svg>
            </button>
          </div>
          <div class="space-y-4 max-h-[60vh] overflow-y-auto">
            <div
              v-for="[provider, count] in sortedProviders"
              :key="provider"
              class="space-y-2"
            >
              <div class="flex items-center justify-between text-sm">
                <span class="font-medium text-gray-700 dark:text-gray-300 truncate">{{ provider || 'unknown' }}</span>
                <span class="font-bold text-gray-900 dark:text-white">{{ count }} {{ $t('stats.units.times') }}</span>
              </div>
              <div class="w-full bg-gray-100 dark:bg-gray-700 rounded-full h-2">
                <div
                  class="h-2 rounded-full bg-gradient-to-r from-orange-400 to-red-500 transition-all"
                  :style="{ width: `${getProviderBarWidth(count)}%` }"
                />
              </div>
            </div>
            <div
              v-if="sortedProviders.length === 0"
              class="text-center text-gray-500 dark:text-gray-400 py-8"
            >
              {{ $t('stats.states.noData') }}
            </div>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { getCostOverview, getProviderUsage } from '@/api/client'
import type { CostStats } from '@/types'

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
      getCostOverview(selectedRange.value),
      getProviderUsage(),
    ])
    stats.value = statsData
    providerUsage.value = providerData
  } catch (e: unknown) {
    const errorMessage = e instanceof Error ? e.message : t('stats.states.loadFailedMessage')
    error.value = errorMessage
    console.error('Failed to load stats:', e)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  loadData()
})

// Computed
const sortedModels = computed(() => {
  if (!stats.value) return []
  return Object.entries(stats.value.by_model).sort((a, b) => b[1] - a[1])
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
  return Object.entries(stats.value.by_project).sort((a, b) => b[1] - a[1])
})

// Utility functions
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
  if (!stats.value) return 0
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

/* Custom scrollbar */
.scrollbar-thin::-webkit-scrollbar {
  width: 4px;
}

.scrollbar-thin::-webkit-scrollbar-track {
  background: transparent;
}

.scrollbar-thin::-webkit-scrollbar-thumb {
  background-color: #d1d5db;
  border-radius: 2px;
}

.dark .scrollbar-thin::-webkit-scrollbar-thumb {
  background-color: #4b5563;
}
</style>
