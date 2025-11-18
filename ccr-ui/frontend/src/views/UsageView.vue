<template>
  <div class="usage-view min-h-screen bg-gray-50 dark:bg-gray-900 p-4 md:p-6 lg:p-8">
    <!-- 🎨 Animated Background Orbs (Subtle) -->
    <div class="fixed inset-0 overflow-hidden pointer-events-none -z-10">
      <div
        class="absolute top-20 right-20 w-96 h-96 rounded-full opacity-10 dark:opacity-20 blur-3xl animate-float"
        style="background: radial-gradient(circle, rgba(59, 130, 246, 0.4) 0%, transparent 70%)"
      />
      <div
        class="absolute bottom-20 left-20 w-96 h-96 rounded-full opacity-10 dark:opacity-20 blur-3xl animate-float-delayed"
        style="background: radial-gradient(circle, rgba(16, 185, 129, 0.4) 0%, transparent 70%)"
      />
      <div
        class="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 w-96 h-96 rounded-full opacity-8 dark:opacity-15 blur-3xl animate-float-slow"
        style="background: radial-gradient(circle, rgba(139, 92, 246, 0.3) 0%, transparent 70%)"
      />
    </div>

    <!-- Header Section -->
    <div class="max-w-7xl mx-auto mb-6">
      <!-- Title & Platform Selector -->
      <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-4 mb-6">
        <div class="glass-card backdrop-blur-xl bg-white/90 dark:bg-gray-800/70 border border-gray-200/50 dark:border-gray-700/50 rounded-2xl p-6 flex-1 shadow-xl">
          <div class="flex items-center gap-4">
            <div class="p-3 rounded-xl bg-gradient-to-br from-blue-500/20 to-purple-600/20 dark:from-blue-400/10 dark:to-purple-500/10">
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
            <div class="flex-1">
              <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-1">
                使用统计分析
              </h1>
              <p class="text-sm text-gray-600 dark:text-gray-400">
                实时追踪跨 AI 平台的 Token 消耗情况
              </p>
            </div>
          </div>
        </div>

        <!-- Platform & Refresh -->
        <div class="flex items-center gap-3">
          <select
            v-model="selectedPlatform"
            class="group px-5 py-3.5 border-2 border-indigo-300 dark:border-indigo-600 rounded-xl bg-gradient-to-br from-white to-indigo-50/50 dark:from-gray-800 dark:to-indigo-900/20 text-gray-900 dark:text-white focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 transition-all duration-300 font-bold shadow-lg hover:shadow-xl hover:shadow-indigo-500/30 dark:hover:shadow-indigo-400/20 cursor-pointer hover:border-indigo-400 dark:hover:border-indigo-500 hover:scale-105 text-base"
          >
            <option value="claude">
              🤖 Claude
            </option>
            <option value="codex">
              💻 Codex
            </option>
            <option value="gemini">
              ✨ Gemini
            </option>
          </select>

          <button
            :disabled="loading"
            class="group relative px-6 py-3.5 bg-gradient-to-r from-emerald-500 via-teal-500 to-cyan-500 hover:from-emerald-600 hover:via-teal-600 hover:to-cyan-600 disabled:from-gray-400 disabled:to-gray-500 text-white rounded-xl flex items-center gap-2.5 transition-all duration-300 shadow-lg hover:shadow-2xl hover:shadow-emerald-500/50 dark:hover:shadow-cyan-500/50 hover:scale-110 active:scale-95 border-2 border-white/40 dark:border-white/20 font-bold disabled:opacity-50 disabled:cursor-not-allowed"
            :style="loading ? '' : 'box-shadow: 0 0 20px rgba(16, 185, 129, 0.4), 0 0 40px rgba(6, 182, 212, 0.2);'"
            @click="loadData"
          >
            <!-- Animated glow background -->
            <div class="absolute inset-0 rounded-xl bg-gradient-to-r from-emerald-400 to-cyan-400 opacity-0 group-hover:opacity-30 blur-xl transition-opacity duration-300" />

            <svg
              class="w-6 h-6 relative z-10 filter drop-shadow-[0_2px_4px_rgba(0,0,0,0.3)]"
              :class="{ 'animate-spin': loading }"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
              stroke-width="3"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
              />
            </svg>
            <span class="hidden sm:inline relative z-10 filter drop-shadow-[0_2px_4px_rgba(0,0,0,0.3)] text-base">{{ loading ? '加载中...' : '刷新数据' }}</span>
          </button>
        </div>
      </div>

      <!-- Filters Bar -->
      <div class="glass-card backdrop-blur-xl bg-white/90 dark:bg-gray-800/70 border border-gray-200/50 dark:border-gray-700/50 rounded-2xl p-4 shadow-xl">
        <div class="flex flex-wrap items-center gap-3">
          <!-- Time Range -->
          <div class="flex items-center gap-2 flex-1 min-w-[200px]">
            <svg
              class="w-5 h-5 text-blue-600 dark:text-blue-400"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
              stroke-width="2"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
            <select
              v-model="selectedRange"
              :disabled="!!(customDateRange.startDate && customDateRange.endDate)"
              class="flex-1 px-3 py-2 border-2 border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 transition-all disabled:opacity-50 text-sm font-semibold shadow-md hover:shadow-lg cursor-pointer hover:border-blue-400 dark:hover:border-blue-500"
            >
              <option value="5h">
                最近 5 小时
              </option>
              <option value="today">
                今天
              </option>
              <option value="7d">
                最近 7 天
              </option>
              <option value="week">
                本周
              </option>
              <option value="month">
                本月
              </option>
              <option value="all">
                全部时间
              </option>
            </select>
          </div>

          <!-- Custom Date Range -->
          <DateRangePicker v-model="customDateRange" />

          <!-- Model Filter -->
          <div class="flex items-center gap-2 flex-1 min-w-[180px]">
            <svg
              class="w-5 h-5 text-purple-600 dark:text-purple-400"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
              stroke-width="2"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z"
              />
            </svg>
            <select
              v-model="selectedModel"
              class="flex-1 px-3 py-2 border-2 border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:ring-2 focus:ring-purple-500 transition-all text-sm font-semibold shadow-md hover:shadow-lg cursor-pointer hover:border-purple-400 dark:hover:border-purple-500"
            >
              <option value="all">
                全部模型
              </option>
              <option
                v-for="model in availableModels"
                :key="model"
                :value="model"
              >
                {{ shortenModelName(model) }}
              </option>
            </select>
          </div>

          <!-- Auto-refresh -->
          <div class="flex items-center gap-3 px-4 py-2 rounded-lg bg-gradient-to-r from-gray-100 to-gray-50 dark:from-gray-700 dark:to-gray-600 border-2 border-gray-300 dark:border-gray-600 shadow-md hover:shadow-lg transition-all">
            <input
              id="auto-refresh"
              v-model="autoRefreshEnabled"
              type="checkbox"
              class="w-5 h-5 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 focus:ring-2 cursor-pointer"
            >
            <label
              for="auto-refresh"
              class="text-sm font-bold text-gray-800 dark:text-gray-200 cursor-pointer whitespace-nowrap"
            >
              自动刷新
            </label>
            <select
              v-if="autoRefreshEnabled"
              v-model="autoRefreshInterval"
              class="px-2 py-1 text-xs border border-gray-300/50 dark:border-gray-600/50 rounded bg-white/70 dark:bg-gray-800/70 text-gray-900 dark:text-white"
            >
              <option :value="30">
                30s
              </option>
              <option :value="60">
                60s
              </option>
              <option :value="120">
                2m
              </option>
              <option :value="300">
                5m
              </option>
            </select>
          </div>
        </div>
      </div>
    </div>

    <!-- Loading State -->
    <div
      v-if="loading"
      class="max-w-7xl mx-auto flex flex-col items-center justify-center py-24 space-y-4"
    >
      <div class="relative">
        <div class="animate-spin rounded-full h-20 w-20 border-b-4 border-t-4 border-blue-600" />
        <div class="absolute inset-0 rounded-full bg-blue-500/20 animate-ping" />
      </div>
      <p class="text-gray-700 dark:text-gray-300 text-lg font-medium">
        正在加载使用数据...
      </p>
    </div>

    <!-- Error State -->
    <div
      v-else-if="error"
      class="max-w-7xl mx-auto glass-card backdrop-blur-xl bg-red-50/80 dark:bg-red-900/20 border-2 border-red-200/50 dark:border-red-800/50 rounded-2xl p-8 shadow-xl"
    >
      <div class="flex items-start gap-4">
        <svg
          class="w-8 h-8 text-red-600 dark:text-red-400 flex-shrink-0 mt-1"
          fill="currentColor"
          viewBox="0 0 20 20"
        >
          <path
            fill-rule="evenodd"
            d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
            clip-rule="evenodd"
          />
        </svg>
        <div class="flex-1">
          <h3 class="text-xl font-bold text-red-800 dark:text-red-200 mb-2">
            加载使用数据失败
          </h3>
          <p class="text-red-700 dark:text-red-300 mb-4">
            {{ error }}
          </p>
          <button
            class="px-6 py-2.5 bg-red-600 hover:bg-red-700 text-white rounded-lg transition-all shadow-lg hover:shadow-xl font-medium"
            @click="loadData"
          >
            重试
          </button>
        </div>
      </div>
    </div>

    <!-- Main Content -->
    <div
      v-else-if="records && records.length > 0"
      class="max-w-7xl mx-auto space-y-6"
    >
      <!-- Summary Cards Grid -->
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <!-- Input Tokens Card -->
        <div class="glass-card backdrop-blur-xl bg-gradient-to-br from-white/95 to-white/85 dark:from-gray-800/80 dark:to-gray-800/60 rounded-2xl p-5 border border-gray-200/50 dark:border-gray-700/50 shadow-xl hover:shadow-2xl transition-all duration-300 hover:scale-105 group">
          <div class="flex items-center justify-between">
            <div class="flex-1">
              <p class="text-xs font-semibold text-gray-600 dark:text-gray-400 mb-1 uppercase tracking-wide">
                输入 Tokens
              </p>
              <p class="text-3xl font-black text-gray-900 dark:text-white">
                {{ formatNumber(totalInputTokens) }}
              </p>
            </div>
            <div class="p-3 bg-gradient-to-br from-blue-500/20 to-blue-600/20 dark:from-blue-400/20 dark:to-blue-500/20 rounded-xl group-hover:scale-110 transition-transform">
              <svg
                class="w-7 h-7 text-blue-600 dark:text-blue-400"
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

        <!-- Output Tokens Card -->
        <div class="glass-card backdrop-blur-xl bg-gradient-to-br from-white/95 to-white/85 dark:from-gray-800/80 dark:to-gray-800/60 rounded-2xl p-5 border border-gray-200/50 dark:border-gray-700/50 shadow-xl hover:shadow-2xl transition-all duration-300 hover:scale-105 group">
          <div class="flex items-center justify-between">
            <div class="flex-1">
              <p class="text-xs font-semibold text-gray-600 dark:text-gray-400 mb-1 uppercase tracking-wide">
                输出 Tokens
              </p>
              <p class="text-3xl font-black text-gray-900 dark:text-white">
                {{ formatNumber(totalOutputTokens) }}
              </p>
            </div>
            <div class="p-3 bg-gradient-to-br from-green-500/20 to-green-600/20 dark:from-green-400/20 dark:to-green-500/20 rounded-xl group-hover:scale-110 transition-transform">
              <svg
                class="w-7 h-7 text-green-600 dark:text-green-400"
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

        <!-- Cache Read Tokens Card -->
        <div class="glass-card backdrop-blur-xl bg-gradient-to-br from-white/95 to-white/85 dark:from-gray-800/80 dark:to-gray-800/60 rounded-2xl p-5 border border-gray-200/50 dark:border-gray-700/50 shadow-xl hover:shadow-2xl transition-all duration-300 hover:scale-105 group">
          <div class="flex items-center justify-between">
            <div class="flex-1">
              <p class="text-xs font-semibold text-gray-600 dark:text-gray-400 mb-1 uppercase tracking-wide">
                缓存读取
              </p>
              <p class="text-3xl font-black text-gray-900 dark:text-white">
                {{ formatNumber(totalCacheTokens) }}
              </p>
            </div>
            <div class="p-3 bg-gradient-to-br from-amber-500/20 to-amber-600/20 dark:from-amber-400/20 dark:to-amber-500/20 rounded-xl group-hover:scale-110 transition-transform">
              <svg
                class="w-7 h-7 text-amber-600 dark:text-amber-400"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M5 8h14M5 8a2 2 0 110-4h14a2 2 0 110 4M5 8v10a2 2 0 002 2h10a2 2 0 002-2V8m-9 4h4"
                />
              </svg>
            </div>
          </div>
        </div>

        <!-- Cache Efficiency Card -->
        <div class="glass-card backdrop-blur-xl bg-gradient-to-br from-white/95 to-white/85 dark:from-gray-800/80 dark:to-gray-800/60 rounded-2xl p-5 border border-gray-200/50 dark:border-gray-700/50 shadow-xl hover:shadow-2xl transition-all duration-300 hover:scale-105 group">
          <div class="flex items-center justify-between">
            <div class="flex-1">
              <p class="text-xs font-semibold text-gray-600 dark:text-gray-400 mb-1 uppercase tracking-wide">
                缓存效率
              </p>
              <p class="text-3xl font-black text-gray-900 dark:text-white">
                {{ cacheEfficiency.toFixed(1) }}%
              </p>
            </div>
            <div class="p-3 bg-gradient-to-br from-purple-500/20 to-purple-600/20 dark:from-purple-400/20 dark:to-purple-500/20 rounded-xl group-hover:scale-110 transition-transform">
              <svg
                class="w-7 h-7 text-purple-600 dark:text-purple-400"
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
            </div>
          </div>
        </div>
      </div>

      <!-- Charts Grid -->
      <div class="grid grid-cols-1 gap-6">
        <!-- Token Usage Chart -->
        <TokenUsageChart
          :records="filteredRecords"
          :time-range="selectedRange"
          :selected-model="selectedModel"
        />

        <!-- Activity Heatmap -->
        <ActivityHeatmap :records="filteredRecords" />
      </div>

      <!-- Truncated Warning -->
      <div
        v-if="truncated"
        class="glass-card backdrop-blur-xl bg-gradient-to-r from-yellow-50/80 to-amber-50/80 dark:from-yellow-900/20 dark:to-amber-900/20 border-2 border-yellow-200/50 dark:border-yellow-800/50 rounded-2xl p-5 shadow-xl"
      >
        <div class="flex items-center gap-3">
          <svg
            class="w-6 h-6 text-yellow-600 dark:text-yellow-400 flex-shrink-0"
            fill="currentColor"
            viewBox="0 0 20 20"
          >
            <path
              fill-rule="evenodd"
              d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z"
              clip-rule="evenodd"
            />
          </svg>
          <p class="text-sm font-semibold text-yellow-800 dark:text-yellow-200">
            Showing first 10,000 records. Total {{ totalRecords.toLocaleString() }} records found.
          </p>
        </div>
      </div>
    </div>

    <!-- Empty State -->
    <div
      v-else
      class="max-w-7xl mx-auto glass-card backdrop-blur-xl bg-white/90 dark:bg-gray-800/70 border border-gray-200/50 dark:border-gray-700/50 rounded-2xl p-16 text-center shadow-xl"
    >
      <svg
        class="mx-auto h-20 w-20 text-gray-400 dark:text-gray-500 mb-6"
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
      <h3 class="text-2xl font-bold text-gray-900 dark:text-white mb-3">
        No Usage Data Available
      </h3>
      <p class="text-gray-600 dark:text-gray-400 text-lg">
        Start using {{ selectedPlatform.charAt(0).toUpperCase() + selectedPlatform.slice(1) }} to see usage statistics here
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { getUsageRecords } from '@/api/client'
import type { UsageRecord, TimeRange } from '@/types'
import TokenUsageChart from '@/components/TokenUsageChart.vue'
import ActivityHeatmap from '@/components/ActivityHeatmap.vue'
import DateRangePicker from '@/components/DateRangePicker.vue'

const records = ref<UsageRecord[]>([])
const loading = ref(false)
const error = ref<string | null>(null)
const selectedRange = ref<TimeRange>('today')
const selectedModel = ref('all')
const truncated = ref(false)
const totalRecords = ref(0)
const selectedPlatform = ref<'claude' | 'codex' | 'gemini'>('claude')
const autoRefreshEnabled = ref(false)
const autoRefreshInterval = ref(60) // seconds
const customDateRange = ref<{ startDate: string | null; endDate: string | null }>({
  startDate: null,
  endDate: null
})

let refreshTimer: number | null = null

const loadData = async () => {
  loading.value = true
  error.value = null

  try {
    const response = await getUsageRecords(selectedPlatform.value, 10000)
    records.value = response.records
    truncated.value = response.truncated
    totalRecords.value = response.total_records
  } catch (e: any) {
    error.value = e.message || 'Failed to load usage data'
    console.error('Failed to load usage data:', e)
  } finally {
    loading.value = false
  }
}

// Auto-refresh management
const startAutoRefresh = () => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
  }
  if (autoRefreshEnabled.value) {
    refreshTimer = window.setInterval(() => {
      loadData()
    }, autoRefreshInterval.value * 1000)
  }
}

const stopAutoRefresh = () => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
    refreshTimer = null
  }
}

watch(autoRefreshEnabled, () => {
  if (autoRefreshEnabled.value) {
    startAutoRefresh()
  } else {
    stopAutoRefresh()
  }
})

watch(autoRefreshInterval, () => {
  if (autoRefreshEnabled.value) {
    startAutoRefresh()
  }
})

// Reload when platform changes
watch(selectedPlatform, () => {
  loadData()
})

onMounted(() => {
  loadData()
})

onUnmounted(() => {
  stopAutoRefresh()
})

// Filtered records based on time range, model, and custom date range
const filteredRecords = computed(() => {
  let filtered = records.value

  // Filter by model
  if (selectedModel.value !== 'all') {
    filtered = filtered.filter(r => r.model === selectedModel.value)
  }

  // Filter by custom date range if set
  if (customDateRange.value.startDate && customDateRange.value.endDate) {
    const start = new Date(customDateRange.value.startDate)
    const end = new Date(customDateRange.value.endDate)
    end.setHours(23, 59, 59, 999)
    filtered = filtered.filter(r => {
      const recordDate = new Date(r.timestamp)
      return recordDate >= start && recordDate <= end
    })
  } else {
    // Filter by preset time range
    const now = new Date()
    let startTime: Date

    switch (selectedRange.value) {
      case '5h':
        startTime = new Date(now.getTime() - 5 * 60 * 60 * 1000)
        break
      case 'today':
        startTime = new Date(now.getFullYear(), now.getMonth(), now.getDate())
        break
      case '7d':
        startTime = new Date(now.getTime() - 6 * 24 * 60 * 60 * 1000)
        break
      case 'week': {
        const dayOfWeek = now.getDay()
        startTime = new Date(now.getTime() - dayOfWeek * 24 * 60 * 60 * 1000)
        startTime.setHours(0, 0, 0, 0)
        break
      }
      case 'month':
        startTime = new Date(now.getFullYear(), now.getMonth(), 1)
        break
      case 'all':
        startTime = new Date(0)
        break
      default:
        startTime = new Date(now.getTime() - 5 * 60 * 60 * 1000)
    }

    filtered = filtered.filter(r => new Date(r.timestamp) >= startTime)
  }

  return filtered
})

// Available models for filter
const availableModels = computed(() => {
  const models = new Set<string>()
  records.value.forEach(r => {
    if (r.model) models.add(r.model)
  })
  return Array.from(models).sort()
})

// Total tokens calculations
const totalInputTokens = computed(() => {
  return filteredRecords.value.reduce((sum, r) => {
    return sum + (r.usage?.input_tokens || 0)
  }, 0)
})

const totalOutputTokens = computed(() => {
  return filteredRecords.value.reduce((sum, r) => {
    return sum + (r.usage?.output_tokens || 0)
  }, 0)
})

const totalCacheTokens = computed(() => {
  return filteredRecords.value.reduce((sum, r) => {
    return sum + (r.usage?.cache_read_input_tokens || 0)
  }, 0)
})

const cacheEfficiency = computed(() => {
  if (totalInputTokens.value === 0) return 0
  return (totalCacheTokens.value / totalInputTokens.value) * 100
})

// Utility functions
const formatNumber = (num: number): string => {
  if (num >= 1000000) {
    return (num / 1000000).toFixed(1) + 'M'
  } else if (num >= 1000) {
    return (num / 1000).toFixed(1) + 'K'
  }
  return num.toString()
}

const shortenModelName = (model: string): string => {
  return model.replace('claude-', '').replace(/-(202\d{5})/, '')
}
</script>

<style scoped>
.usage-view {
  position: relative;
  min-height: 100vh;
}

.glass-card {
  box-shadow: 0 8px 32px 0 rgba(31, 38, 135, 0.15);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.glass-card:hover {
  box-shadow: 0 12px 48px 0 rgba(31, 38, 135, 0.25);
}

select, button {
  transition: all 0.2s ease;
}

select:hover, button:hover:not(:disabled) {
  transform: translateY(-1px);
}

/* Animated background orbs */
@keyframes float {
  0%, 100% {
    transform: translate(0, 0);
  }
  50% {
    transform: translate(30px, -30px);
  }
}

@keyframes float-delayed {
  0%, 100% {
    transform: translate(0, 0);
  }
  50% {
    transform: translate(-30px, 30px);
  }
}

@keyframes float-slow {
  0%, 100% {
    transform: translate(-50%, -50%);
  }
  50% {
    transform: translate(calc(-50% + 40px), calc(-50% - 40px));
  }
}

.animate-float {
  animation: float 8s ease-in-out infinite;
}

.animate-float-delayed {
  animation: float-delayed 10s ease-in-out infinite;
  animation-delay: 2s;
}

.animate-float-slow {
  animation: float-slow 12s ease-in-out infinite;
  animation-delay: 4s;
}
</style>
