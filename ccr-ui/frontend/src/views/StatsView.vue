<template>
  <div class="stats-view p-6 space-y-6">
    <!-- 页面标题 -->
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-3xl font-bold text-gray-900 dark:text-white">
          {{ $t('stats.title') }}
        </h1>
        <p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
          {{ $t('stats.subtitle') }}
        </p>
      </div>
      <div class="flex items-center space-x-4">
        <!-- 时间范围选择 -->
        <select
          v-model="selectedRange"
          class="px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
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

        <!-- 提供商统计弹窗按钮 -->
        <button
          class="px-4 py-2 border border-blue-200 dark:border-blue-500 text-blue-700 dark:text-blue-200 rounded-lg flex items-center space-x-2 hover:bg-blue-50 dark:hover:bg-blue-900/30"
          @click="showProvidersModal = true"
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
              d="M3 3h18M3 9h18M3 15h18M3 21h18"
            />
          </svg>
          <span>{{ $t('stats.actions.providersStats') }}</span>
        </button>

        <!-- 刷新按钮 -->
        <button
          :disabled="loading"
          class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg flex items-center space-x-2 disabled:opacity-50"
          @click="loadData"
        >
          <svg
            class="w-5 h-5"
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

    <!-- 加载状态 -->
    <div
      v-if="loading"
      class="flex items-center justify-center py-12"
    >
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600" />
    </div>

    <!-- 错误提示 -->
    <div
      v-if="error"
      class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4"
    >
      <div class="flex">
        <svg
          class="h-5 w-5 text-red-400"
          fill="currentColor"
          viewBox="0 0 20 20"
        >
          <path
            fill-rule="evenodd"
            d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
            clip-rule="evenodd"
          />
        </svg>
        <div class="ml-3">
          <h3 class="text-sm font-medium text-red-800 dark:text-red-200">
            {{ $t('stats.states.loadFailed') }}
          </h3>
          <p class="mt-2 text-sm text-red-700 dark:text-red-300">
            {{ error }}
          </p>
        </div>
      </div>
    </div>

    <!-- 统计内容 -->
    <div
      v-if="!loading && !error && stats"
      class="space-y-6"
    >
      <!-- 概览卡片 -->
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <!-- 总成本 -->
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm font-medium text-gray-600 dark:text-gray-400">
                {{ $t('stats.summaryCards.totalCost') }}
              </p>
              <p class="mt-2 text-3xl font-bold text-gray-900 dark:text-white">
                ${{ formatCost(stats.total_cost) }}
              </p>
            </div>
            <div class="p-3 bg-green-100 dark:bg-green-900/20 rounded-full">
              <svg
                class="w-8 h-8 text-green-600 dark:text-green-400"
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

        <!-- 记录数 -->
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm font-medium text-gray-600 dark:text-gray-400">
                {{ $t('stats.summaryCards.apiCalls') }}
              </p>
              <p class="mt-2 text-3xl font-bold text-gray-900 dark:text-white">
                {{ stats.record_count }}
              </p>
            </div>
            <div class="p-3 bg-blue-100 dark:bg-blue-900/20 rounded-full">
              <svg
                class="w-8 h-8 text-blue-600 dark:text-blue-400"
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

        <!-- 输入 Token -->
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm font-medium text-gray-600 dark:text-gray-400">
                {{ $t('stats.summaryCards.inputToken') }}
              </p>
              <p class="mt-2 text-3xl font-bold text-gray-900 dark:text-white">
                {{ formatNumber(stats.token_stats.total_input_tokens) }}
              </p>
            </div>
            <div class="p-3 bg-purple-100 dark:bg-purple-900/20 rounded-full">
              <svg
                class="w-8 h-8 text-purple-600 dark:text-purple-400"
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

        <!-- 输出 Token -->
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm font-medium text-gray-600 dark:text-gray-400">
                {{ $t('stats.summaryCards.outputToken') }}
              </p>
              <p class="mt-2 text-3xl font-bold text-gray-900 dark:text-white">
                {{ formatNumber(stats.token_stats.total_output_tokens) }}
              </p>
            </div>
            <div class="p-3 bg-orange-100 dark:bg-orange-900/20 rounded-full">
              <svg
                class="w-8 h-8 text-orange-600 dark:text-orange-400"
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

      <!-- Token 详细统计 -->
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <h2 class="text-xl font-bold text-gray-900 dark:text-white mb-4">
          {{ $t('stats.tokenDetails.title') }}
        </h2>
        <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div>
            <p class="text-sm text-gray-600 dark:text-gray-400">
              {{ $t('stats.tokenDetails.cacheToken') }}
            </p>
            <p class="text-2xl font-bold text-gray-900 dark:text-white mt-1">
              {{ formatNumber(stats.token_stats.total_cache_tokens) }}
            </p>
          </div>
          <div>
            <p class="text-sm text-gray-600 dark:text-gray-400">
              {{ $t('stats.tokenDetails.cacheEfficiency') }}
            </p>
            <p class="text-2xl font-bold text-gray-900 dark:text-white mt-1">
              {{ formatPercent(stats.token_stats.cache_efficiency) }}%
            </p>
          </div>
          <div>
            <p class="text-sm text-gray-600 dark:text-gray-400">
              {{ $t('stats.tokenDetails.totalToken') }}
            </p>
            <p class="text-2xl font-bold text-gray-900 dark:text-white mt-1">
              {{ formatNumber(getTotalTokens()) }}
            </p>
          </div>
        </div>
      </div>

      <!-- 按模型分组 -->
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <h2 class="text-xl font-bold text-gray-900 dark:text-white mb-4">
          {{ $t('stats.sections.byModel') }}
        </h2>
        <div class="space-y-3">
          <div
            v-for="[model, cost] in sortedModels"
            :key="model"
            class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700/50 rounded"
          >
            <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
              {{ shortenModelName(model) }}
            </span>
            <span class="text-sm font-bold text-gray-900 dark:text-white">
              ${{ formatCost(cost) }}
            </span>
          </div>
          <div
            v-if="Object.keys(stats.by_model).length === 0"
            class="text-center text-gray-500 dark:text-gray-400 py-4"
          >
            {{ $t('stats.states.noData') }}
          </div>
        </div>
      </div>

      <!-- 按项目分组 -->
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <h2 class="text-xl font-bold text-gray-900 dark:text-white mb-4">
          {{ $t('stats.sections.byProject') }}
        </h2>
        <div class="space-y-3">
          <div
            v-for="[project, cost] in sortedProjects.slice(0, 10)"
            :key="project"
            class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700/50 rounded"
          >
            <span class="text-sm font-medium text-gray-700 dark:text-gray-300 truncate flex-1 mr-4">
              {{ shortenPath(project) }}
            </span>
            <span class="text-sm font-bold text-gray-900 dark:text-white">
              ${{ formatCost(cost) }}
            </span>
          </div>
          <div
            v-if="Object.keys(stats.by_project).length === 0"
            class="text-center text-gray-500 dark:text-gray-400 py-4"
          >
            {{ $t('stats.states.noData') }}
          </div>
        </div>
      </div>

      <!-- 趋势图表（简单版）-->
      <div
        v-if="stats.trend && stats.trend.length > 0"
        class="bg-white dark:bg-gray-800 rounded-lg shadow p-6"
      >
        <h2 class="text-xl font-bold text-gray-900 dark:text-white mb-4">
          {{ $t('stats.sections.costTrend') }}
        </h2>
        <div class="space-y-2">
          <div
            v-for="daily in stats.trend.slice().reverse().slice(0, 7).reverse()"
            :key="daily.date"
            class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700/50 rounded"
          >
            <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
              {{ daily.date }}
            </span>
            <div class="flex items-center space-x-4">
              <span class="text-sm text-gray-600 dark:text-gray-400">
                {{ daily.count }} {{ $t('stats.units.times') }}
              </span>
              <span class="text-sm font-bold text-gray-900 dark:text-white">
                ${{ formatCost(daily.cost) }}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <div
      v-if="!loading && !error && stats && stats.record_count === 0"
      class="bg-white dark:bg-gray-800 rounded-lg shadow p-12 text-center"
    >
      <svg
        class="mx-auto h-12 w-12 text-gray-400"
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
      <h3 class="mt-2 text-sm font-medium text-gray-900 dark:text-white">
        {{ $t('stats.states.noStatsData') }}
      </h3>
      <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
        {{ $t('stats.states.noStatsHint') }}
      </p>
    </div>
  </div>

  <!-- 提供商统计弹窗 -->
  <div
    v-if="showProvidersModal"
    class="providers-modal"
    @click.self="showProvidersModal = false"
  >
    <div class="providers-modal-card dark:bg-gray-800 dark:text-white">
      <div class="flex items-center justify-between mb-4">
        <div>
          <h3 class="text-xl font-bold">
            {{ $t('stats.sections.providerUsage') }}
          </h3>
          <p class="text-sm text-gray-600 dark:text-gray-400">
            {{ $t('stats.sections.providerUsageSubtitle') }}
          </p>
        </div>
        <button
          class="text-gray-500 hover:text-gray-800 dark:hover:text-white"
          @click="showProvidersModal = false"
        >
          ✕
        </button>
      </div>
      <div class="space-y-3 max-h-[60vh] overflow-y-auto">
        <div
          v-for="[provider, count] in sortedProviders"
          :key="provider"
          class="space-y-2"
        >
          <div class="flex items-center justify-between text-sm text-gray-700 dark:text-gray-300">
            <span class="font-medium truncate">{{ provider || 'unknown' }}</span>
            <span class="font-bold text-gray-900 dark:text-white">{{ count }} {{ $t('stats.units.times') }}</span>
          </div>
          <div class="w-full bg-gray-100 dark:bg-gray-700/70 rounded-full h-3">
            <div
              class="h-3 rounded-full bg-blue-500 dark:bg-blue-400 transition-all"
              :style="{ width: `${getProviderBarWidth(count)}%` }"
            />
          </div>
        </div>
        <div
          v-if="sortedProviders.length === 0"
          class="text-center text-gray-500 dark:text-gray-400 py-4"
        >
          {{ $t('stats.states.noData') }}
        </div>
      </div>
    </div>
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
    stats.value = await getCostOverview(selectedRange.value)
    providerUsage.value = await getProviderUsage()
  } catch (e: any) {
    error.value = e.message || t('stats.states.loadFailedMessage')
    console.error('Failed to load stats:', e)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  loadData()
})

// 计算属性
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

// 工具函数
const formatCost = (cost: number): string => {
  return cost.toFixed(4)
}

const formatNumber = (num: number): string => {
  if (num >= 1000000) {
    return (num / 1000000).toFixed(1) + 'M'
  } else if (num >= 1000) {
    return (num / 1000).toFixed(1) + 'K'
  }
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

.providers-modal {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 50;
}

.providers-modal-card {
  width: 100%;
  max-width: 640px;
  background: var(--color-surface, #fff);
  color: var(--color-text, #111827);
  border-radius: 0.75rem;
  padding: 1.5rem;
  box-shadow: 0 15px 40px rgba(0, 0, 0, 0.25);
}
</style>
