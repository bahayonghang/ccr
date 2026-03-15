<template>
  <Card
    variant="glass"
    class="relative overflow-hidden"
  >
    <!-- 背景装饰 -->
    <div class="absolute top-0 right-0 w-64 h-64 bg-accent-secondary/5 rounded-full blur-3xl -mr-16 -mt-16" />

    <div class="relative z-10 space-y-6">
      <!-- 标题区域 -->
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 class="text-xl font-bold text-text-primary flex items-center gap-2">
            <SIcon
              name="BarChart3"
              size="w-5 h-5"
              class="text-accent-secondary"
            />
            {{ $t('usageStats.title') }}
          </h2>
          <p class="text-sm text-text-muted mt-1">
            {{ $t('usageStats.updated') }} {{ formatLastUpdated(dailyStats?.last_updated) }}
            · {{ $t('usageStats.cacheEntries', { count: dailyStats?.summary?.total_sessions || 0 }) }}
          </p>
        </div>

        <div class="flex items-center gap-3">
          <!-- 日期范围选择器 -->
          <div class="flex items-center gap-1 bg-bg-surface/50 p-1 rounded-lg">
            <button
              v-for="range in dateRanges"
              :key="range.days"
              class="px-3 py-1 text-xs font-medium rounded-md transition-colors duration-200"
              :class="selectedDays === range.days
                ? 'bg-accent-secondary text-white shadow-sm'
                : 'text-text-secondary hover:text-text-primary hover:bg-bg-elevated/50'"
              @click="setDateRange(range.days)"
            >
              {{ range.label }}
            </button>
          </div>

          <!-- 视图切换按钮 -->
          <div class="flex items-center gap-1 bg-bg-surface/50 p-1 rounded-lg">
            <button
              v-for="mode in viewModes"
              :key="mode.value"
              class="px-4 py-1.5 text-sm font-medium rounded-md transition-colors duration-200"
              :class="viewMode === mode.value
                ? 'bg-accent-secondary text-white shadow-sm'
                : 'text-text-secondary hover:text-text-primary hover:bg-bg-elevated/50'"
              @click="viewMode = mode.value"
            >
              {{ $t(mode.labelKey) }}
            </button>
          </div>
        </div>
      </div>

      <!-- 柱状图 -->
      <div v-if="!loading && dailyStats">
        <UsageStatsChart
          :data="dailyStats?.daily_stats ?? []"
          :view-mode="viewMode"
        />
      </div>
      <div
        v-else-if="loading"
        class="h-72 flex items-center justify-center"
      >
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-accent-secondary" />
      </div>
      <div
        v-else
        class="h-72 flex items-center justify-center text-text-muted"
      >
        {{ $t('usageStats.noData') }}
      </div>

      <!-- 汇总卡片网格 -->
      <div class="grid grid-cols-2 lg:grid-cols-4 gap-3">
        <!-- Sessions 卡片 -->
        <div class="relative p-4 rounded-xl bg-bg-elevated/30 border border-border-default/30 hover:border-emerald-500/30 transition-[color,background-color,box-shadow] duration-300 overflow-hidden group">
          <div class="absolute top-0 left-0 w-1 h-full bg-emerald-500/60 rounded-r" />
          <div class="flex items-start gap-3 ml-1">
            <div class="p-2 rounded-lg bg-emerald-500/10 shrink-0 group-hover:bg-emerald-500/20 transition-colors">
              <SIcon
                name="Activity"
                size="w-4 h-4"
                class="text-emerald-400"
              />
            </div>
            <div class="flex-1 min-w-0">
              <p class="text-[11px] font-medium text-text-muted uppercase tracking-wider">
                {{ $t('usageStats.sessions') }}
              </p>
              <p class="text-2xl font-bold text-text-primary mt-0.5 tabular-nums">
                {{ dailyStats?.summary?.total_sessions || 0 }}
              </p>
              <p class="text-[11px] text-text-secondary mt-1">
                {{ $t('usageStats.inSelectedRange') }}
              </p>
            </div>
          </div>
        </div>

        <!-- Messages 卡片 -->
        <div class="relative p-4 rounded-xl bg-bg-elevated/30 border border-border-default/30 hover:border-blue-500/30 transition-[color,background-color,box-shadow] duration-300 overflow-hidden group">
          <div class="absolute top-0 left-0 w-1 h-full bg-blue-500/60 rounded-r" />
          <div class="flex items-start gap-3 ml-1">
            <div class="p-2 rounded-lg bg-blue-500/10 shrink-0 group-hover:bg-blue-500/20 transition-colors">
              <SIcon
                name="MessageSquare"
                size="w-4 h-4"
                class="text-blue-400"
              />
            </div>
            <div class="flex-1 min-w-0">
              <p class="text-[11px] font-medium text-text-muted uppercase tracking-wider">
                {{ $t('usageStats.messages') }}
              </p>
              <p class="text-2xl font-bold text-text-primary mt-0.5 tabular-nums">
                {{ formatNumber(dailyStats?.summary?.total_messages || 0) }}
              </p>
              <p class="text-[11px] text-text-secondary mt-1">
                {{ $t('usageStats.userAssistant') }}
              </p>
            </div>
          </div>
        </div>

        <!-- Active Time 卡片 -->
        <div class="relative p-4 rounded-xl bg-bg-elevated/30 border border-border-default/30 hover:border-amber-500/30 transition-[color,background-color,box-shadow] duration-300 overflow-hidden group">
          <div class="absolute top-0 left-0 w-1 h-full bg-amber-500/60 rounded-r" />
          <div class="flex items-start gap-3 ml-1">
            <div class="p-2 rounded-lg bg-amber-500/10 shrink-0 group-hover:bg-amber-500/20 transition-colors">
              <SIcon
                name="Timer"
                size="w-4 h-4"
                class="text-amber-400"
              />
            </div>
            <div class="flex-1 min-w-0">
              <p class="text-[11px] font-medium text-text-muted uppercase tracking-wider">
                {{ $t('usageStats.activeTime') }}
              </p>
              <p class="text-2xl font-bold text-text-primary mt-0.5 tabular-nums">
                {{ formatDuration(dailyStats?.summary?.total_duration_seconds || 0) }}
              </p>
              <p class="text-[11px] text-text-secondary mt-1">
                {{ $t('usageStats.totalTime') }}
              </p>
            </div>
          </div>
        </div>

        <!-- Platforms 卡片 -->
        <div class="relative p-4 rounded-xl bg-bg-elevated/30 border border-border-default/30 hover:border-purple-500/30 transition-[color,background-color,box-shadow] duration-300 overflow-hidden group">
          <div class="absolute top-0 left-0 w-1 h-full bg-purple-500/60 rounded-r" />
          <div class="flex items-start gap-3 ml-1">
            <div class="p-2 rounded-lg bg-purple-500/10 shrink-0 group-hover:bg-purple-500/20 transition-colors">
              <SIcon
                name="Layers"
                size="w-4 h-4"
                class="text-purple-400"
              />
            </div>
            <div class="flex-1 min-w-0">
              <p class="text-[11px] font-medium text-text-muted uppercase tracking-wider">
                {{ $t('usageStats.platforms') }}
              </p>
              <p class="text-2xl font-bold text-text-primary mt-0.5 tabular-nums">
                {{ Object.keys(dailyStats?.summary?.by_platform || {}).length }}
              </p>
              <p class="text-[11px] text-text-secondary mt-1">
                {{ $t('usageStats.trackedPlatforms') }}
              </p>
            </div>
          </div>
        </div>
      </div>

      <!-- 渐变分隔线 -->
      <div class="h-px bg-gradient-to-r from-transparent via-border-default/50 to-transparent" />

      <!-- 平台分项统计 -->
      <div class="grid grid-cols-2 lg:grid-cols-4 gap-3">
        <!-- All 卡片 -->
        <div class="p-4 rounded-xl bg-bg-elevated/40 border border-border-default/50 hover:bg-bg-elevated/60 transition-[color,background-color,box-shadow] duration-300">
          <div class="flex items-center gap-2 mb-3">
            <div class="w-2.5 h-2.5 rounded-full bg-text-secondary ring-2 ring-text-secondary/20" />
            <span class="font-semibold text-text-primary">All</span>
            <span class="ml-auto text-xs text-text-muted font-mono tabular-nums">
              {{ dailyStats?.summary?.total_sessions || 0 }}
            </span>
          </div>
          <div class="space-y-1.5 text-xs text-text-secondary">
            <div class="flex items-center gap-1.5">
              <SIcon
                name="Hash"
                size="w-3 h-3"
                class="opacity-40"
              />
              <span>{{ formatNumber(dailyStats?.summary?.total_messages || 0) }} messages</span>
            </div>
            <div class="flex items-center gap-1.5">
              <SIcon
                name="Clock"
                size="w-3 h-3"
                class="opacity-40"
              />
              <span>{{ formatAvgDuration(dailyStats?.summary?.total_duration_seconds || 0, dailyStats?.summary?.total_sessions || 1) }}</span>
            </div>
          </div>
        </div>

        <!-- Codex 卡片 -->
        <div class="p-4 rounded-xl bg-orange-500/5 border border-orange-500/20 hover:border-orange-500/40 hover:bg-orange-500/10 transition-[color,background-color,box-shadow] duration-300">
          <div class="flex items-center gap-2 mb-3">
            <div class="w-2.5 h-2.5 rounded-full bg-orange-500 ring-2 ring-orange-500/20" />
            <span class="font-semibold text-orange-400">Codex</span>
            <span class="ml-auto text-xs text-text-muted font-mono tabular-nums">
              {{ getPlatformSessions('codex') }}
            </span>
          </div>
          <div class="space-y-1.5 text-xs text-text-secondary">
            <div class="flex items-center gap-1.5">
              <SIcon
                name="Hash"
                size="w-3 h-3"
                class="opacity-40"
              />
              <span>{{ formatNumber(getPlatformMessages('codex')) }} messages</span>
            </div>
            <div class="flex items-center gap-1.5">
              <SIcon
                name="Clock"
                size="w-3 h-3"
                class="opacity-40"
              />
              <span>{{ formatAvgDuration(getPlatformDuration('codex'), getPlatformSessions('codex') || 1) }}</span>
            </div>
          </div>
        </div>

        <!-- Claude 卡片 -->
        <div class="p-4 rounded-xl bg-pink-500/5 border border-pink-500/20 hover:border-pink-500/40 hover:bg-pink-500/10 transition-[color,background-color,box-shadow] duration-300">
          <div class="flex items-center gap-2 mb-3">
            <div class="w-2.5 h-2.5 rounded-full bg-pink-400 ring-2 ring-pink-400/20" />
            <span class="font-semibold text-pink-400">Claude</span>
            <span class="ml-auto text-xs text-text-muted font-mono tabular-nums">
              {{ getPlatformSessions('claude') }}
            </span>
          </div>
          <div class="space-y-1.5 text-xs text-text-secondary">
            <div class="flex items-center gap-1.5">
              <SIcon
                name="Hash"
                size="w-3 h-3"
                class="opacity-40"
              />
              <span>{{ formatNumber(getPlatformMessages('claude')) }} messages</span>
            </div>
            <div class="flex items-center gap-1.5">
              <SIcon
                name="Clock"
                size="w-3 h-3"
                class="opacity-40"
              />
              <span>{{ formatAvgDuration(getPlatformDuration('claude'), getPlatformSessions('claude') || 1) }}</span>
            </div>
          </div>
        </div>

        <!-- Gemini 卡片 -->
        <div class="p-4 rounded-xl bg-blue-500/5 border border-blue-500/20 hover:border-blue-500/40 hover:bg-blue-500/10 transition-[color,background-color,box-shadow] duration-300">
          <div class="flex items-center gap-2 mb-3">
            <div class="w-2.5 h-2.5 rounded-full bg-blue-500 ring-2 ring-blue-500/20" />
            <span class="font-semibold text-blue-400">Gemini</span>
            <span class="ml-auto text-xs text-text-muted font-mono tabular-nums">
              {{ getPlatformSessions('gemini') }}
            </span>
          </div>
          <div class="space-y-1.5 text-xs text-text-secondary">
            <div class="flex items-center gap-1.5">
              <SIcon
                name="Hash"
                size="w-3 h-3"
                class="opacity-40"
              />
              <span>{{ formatNumber(getPlatformMessages('gemini')) }} messages</span>
            </div>
            <div class="flex items-center gap-1.5">
              <SIcon
                name="Clock"
                size="w-3 h-3"
                class="opacity-40"
              />
              <span>{{ formatAvgDuration(getPlatformDuration('gemini'), getPlatformSessions('gemini') || 1) }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { onMounted, ref } from 'vue'
import Card from '@/components/ui/Card.vue'
import UsageStatsChart from '@/components/UsageStatsChart.vue'
import { getDailyStats } from '@/api'
import type { DailyStatsResponse, StatsViewMode } from '@/types'
import { logger } from '@/utils/logger'

// 视图模式
const viewMode = ref<StatsViewMode>('sessions')
const viewModes = [
  { value: 'sessions' as StatsViewMode, labelKey: 'usageStats.viewModes.sessions' },
  { value: 'duration' as StatsViewMode, labelKey: 'usageStats.viewModes.duration' },
  { value: 'tokens' as StatsViewMode, labelKey: 'usageStats.viewModes.tokens' },
]

// 日期范围
const selectedDays = ref(30)
const dateRanges = [
  { days: 7, label: '7D' },
  { days: 30, label: '30D' },
  { days: 90, label: '90D' },
]

// 数据状态
const dailyStats = ref<DailyStatsResponse | null>(null)
const loading = ref(true)
const DAILY_STATS_CACHE_TTL = 60_000
const dailyStatsCache = new Map<number, { data: DailyStatsResponse; ts: number }>()

const loadDailyStats = async (days: number, force: boolean = false) => {
  const cached = dailyStatsCache.get(days)
  const now = Date.now()
  if (!force && cached && now - cached.ts < DAILY_STATS_CACHE_TTL) {
    dailyStats.value = cached.data
    loading.value = false
    return
  }

  loading.value = true
  try {
    const data = await getDailyStats<DailyStatsResponse>(days)
    if (data && data.daily_stats && data.summary) {
      dailyStats.value = data
      dailyStatsCache.set(days, { data, ts: now })
    }
  } catch (error) {
    logger.error('Failed to load daily stats:', error)
  } finally {
    loading.value = false
  }
}

// 设置日期范围
const setDateRange = async (days: number) => {
  if (selectedDays.value === days && dailyStats.value) return
  selectedDays.value = days
  await loadDailyStats(days)
}

// 加载数据
onMounted(async () => {
  await loadDailyStats(selectedDays.value)
})

// 获取平台统计
const getPlatformSessions = (platform: string): number => {
  return dailyStats.value?.summary?.by_platform?.[platform]?.sessions || 0
}

const getPlatformMessages = (platform: string): number => {
  return dailyStats.value?.summary?.by_platform?.[platform]?.messages || 0
}

const getPlatformDuration = (platform: string): number => {
  return dailyStats.value?.summary?.by_platform?.[platform]?.duration_seconds || 0
}

// 格式化函数
const formatNumber = (num: number): string => {
  if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`
  if (num >= 1000) return `${(num / 1000).toFixed(1)}K`
  return num.toString()
}

const formatDuration = (seconds: number): string => {
  if (seconds < 60) return `${seconds}s`
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m`
  const hours = Math.floor(seconds / 3600)
  const mins = Math.floor((seconds % 3600) / 60)
  return mins > 0 ? `${hours}h ${mins}m` : `${hours}h`
}

const formatAvgDuration = (totalSeconds: number, count: number): string => {
  const avg = Math.floor(totalSeconds / count)
  return `Avg ${formatDuration(avg)}`
}

const formatLastUpdated = (dateStr?: string): string => {
  if (!dateStr) return 'N/A'
  try {
    const date = new Date(dateStr)
    return date.toLocaleString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    })
  } catch {
    return dateStr
  }
}
</script>
