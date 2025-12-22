<template>
  <GuofengCard
    variant="glass"
    class="relative overflow-hidden"
  >
    <!-- 背景装饰 -->
    <div class="absolute top-0 right-0 w-64 h-64 bg-guofeng-jade/5 rounded-full blur-3xl -mr-16 -mt-16" />
    
    <div class="relative z-10 space-y-6">
      <!-- 标题区域 -->
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 class="text-xl font-bold text-guofeng-text-primary flex items-center gap-2">
            <BarChart3 class="w-5 h-5 text-guofeng-jade" />
            {{ $t('usageStats.title') }}
          </h2>
          <p class="text-sm text-guofeng-text-muted mt-1">
            {{ $t('usageStats.updated') }} {{ formatLastUpdated(dailyStats?.last_updated) }}
            · {{ $t('usageStats.cacheEntries', { count: dailyStats?.summary.total_sessions || 0 }) }}
          </p>
        </div>
        
        <div class="flex items-center gap-3">
          <!-- 日期范围选择器 -->
          <div class="flex items-center gap-1 bg-guofeng-bg-tertiary/50 p-1 rounded-lg">
            <button
              v-for="range in dateRanges"
              :key="range.days"
              class="px-3 py-1 text-xs font-medium rounded-md transition-all duration-200"
              :class="selectedDays === range.days 
                ? 'bg-guofeng-indigo text-white shadow-sm' 
                : 'text-guofeng-text-secondary hover:text-guofeng-text-primary hover:bg-guofeng-bg-secondary/50'"
              @click="setDateRange(range.days)"
            >
              {{ range.label }}
            </button>
          </div>
          
          <!-- 视图切换按钮 -->
          <div class="flex items-center gap-1 bg-guofeng-bg-tertiary/50 p-1 rounded-lg">
            <button
              v-for="mode in viewModes"
              :key="mode.value"
              class="px-4 py-1.5 text-sm font-medium rounded-md transition-all duration-200"
              :class="viewMode === mode.value 
                ? 'bg-guofeng-jade text-white shadow-sm' 
                : 'text-guofeng-text-secondary hover:text-guofeng-text-primary hover:bg-guofeng-bg-secondary/50'"
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
          :data="dailyStats.daily_stats"
          :view-mode="viewMode"
        />
      </div>
      <div
        v-else-if="loading"
        class="h-64 flex items-center justify-center"
      >
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-guofeng-jade" />
      </div>
      <div
        v-else
        class="h-64 flex items-center justify-center text-guofeng-text-muted"
      >
        {{ $t('usageStats.noData') }}
      </div>
      
      <!-- 汇总卡片网格 -->
      <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <!-- Sessions 卡片 -->
        <div class="p-4 rounded-xl bg-guofeng-bg-secondary/30 border border-guofeng-border/30">
          <p class="text-xs font-medium text-guofeng-text-muted uppercase tracking-wide mb-1">
            {{ $t('usageStats.sessions') }}
          </p>
          <p class="text-2xl font-bold text-guofeng-text-primary">
            {{ dailyStats?.summary.total_sessions || 0 }}
          </p>
          <p class="text-xs text-guofeng-text-secondary mt-1">
            {{ $t('usageStats.inSelectedRange') }}
          </p>
        </div>
        
        <!-- Messages 卡片 -->
        <div class="p-4 rounded-xl bg-guofeng-bg-secondary/30 border border-guofeng-border/30">
          <p class="text-xs font-medium text-guofeng-text-muted uppercase tracking-wide mb-1">
            {{ $t('usageStats.messages') }}
          </p>
          <p class="text-2xl font-bold text-guofeng-text-primary">
            {{ formatNumber(dailyStats?.summary.total_messages || 0) }}
          </p>
          <p class="text-xs text-guofeng-text-secondary mt-1">
            {{ $t('usageStats.userAssistant') }}
          </p>
        </div>
        
        <!-- Active Time 卡片 -->
        <div class="p-4 rounded-xl bg-guofeng-bg-secondary/30 border border-guofeng-border/30">
          <p class="text-xs font-medium text-guofeng-text-muted uppercase tracking-wide mb-1">
            {{ $t('usageStats.activeTime') }}
          </p>
          <p class="text-2xl font-bold text-guofeng-text-primary">
            {{ formatDuration(dailyStats?.summary.total_duration_seconds || 0) }}
          </p>
          <p class="text-xs text-guofeng-text-secondary mt-1">
            {{ $t('usageStats.totalTime') }}
          </p>
        </div>
        
        <!-- Projects 卡片 -->
        <div class="p-4 rounded-xl bg-guofeng-bg-secondary/30 border border-guofeng-border/30">
          <p class="text-xs font-medium text-guofeng-text-muted uppercase tracking-wide mb-1">
            {{ $t('usageStats.platforms') }}
          </p>
          <p class="text-2xl font-bold text-guofeng-text-primary">
            {{ Object.keys(dailyStats?.summary.by_platform || {}).length }}
          </p>
          <p class="text-xs text-guofeng-text-secondary mt-1">
            {{ $t('usageStats.trackedPlatforms') }}
          </p>
        </div>
      </div>
      
      <!-- 平台分项统计 -->
      <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <!-- All 卡片 -->
        <div class="p-4 rounded-xl bg-guofeng-bg-secondary/40 border border-guofeng-border/50">
          <div class="flex items-center justify-between mb-2">
            <span class="font-semibold text-guofeng-text-primary">All</span>
            <span class="text-sm text-guofeng-text-muted">
              {{ dailyStats?.summary.total_sessions || 0 }} sessions
            </span>
          </div>
          <div class="space-y-1 text-xs text-guofeng-text-secondary">
            <div class="flex items-center gap-1">
              <Hash class="w-3 h-3" />
              <span>{{ formatNumber(dailyStats?.summary.total_messages || 0) }} messages</span>
            </div>
            <div class="flex items-center gap-1">
              <Clock class="w-3 h-3" />
              <span>{{ formatAvgDuration(dailyStats?.summary.total_duration_seconds || 0, dailyStats?.summary.total_sessions || 1) }}</span>
            </div>
          </div>
        </div>
        
        <!-- Codex 卡片 -->
        <div class="p-4 rounded-xl bg-orange-500/5 border border-orange-500/20">
          <div class="flex items-center justify-between mb-2">
            <span class="font-semibold text-orange-500">Codex</span>
            <span class="text-sm text-guofeng-text-muted">
              {{ getPlatformSessions('codex') }} sessions
            </span>
          </div>
          <div class="space-y-1 text-xs text-guofeng-text-secondary">
            <div class="flex items-center gap-1">
              <Hash class="w-3 h-3" />
              <span>{{ formatNumber(getPlatformMessages('codex')) }} messages</span>
            </div>
            <div class="flex items-center gap-1">
              <Clock class="w-3 h-3" />
              <span>{{ formatAvgDuration(getPlatformDuration('codex'), getPlatformSessions('codex') || 1) }}</span>
            </div>
          </div>
        </div>
        
        <!-- Claude 卡片 -->
        <div class="p-4 rounded-xl bg-pink-500/5 border border-pink-500/20">
          <div class="flex items-center justify-between mb-2">
            <span class="font-semibold text-pink-400">Claude</span>
            <span class="text-sm text-guofeng-text-muted">
              {{ getPlatformSessions('claude') }} sessions
            </span>
          </div>
          <div class="space-y-1 text-xs text-guofeng-text-secondary">
            <div class="flex items-center gap-1">
              <Hash class="w-3 h-3" />
              <span>{{ formatNumber(getPlatformMessages('claude')) }} messages</span>
            </div>
            <div class="flex items-center gap-1">
              <Clock class="w-3 h-3" />
              <span>{{ formatAvgDuration(getPlatformDuration('claude'), getPlatformSessions('claude') || 1) }}</span>
            </div>
          </div>
        </div>
        
        <!-- Gemini 卡片 -->
        <div class="p-4 rounded-xl bg-blue-500/5 border border-blue-500/20">
          <div class="flex items-center justify-between mb-2">
            <span class="font-semibold text-blue-500">Gemini</span>
            <span class="text-sm text-guofeng-text-muted">
              {{ getPlatformSessions('gemini') }} sessions
            </span>
          </div>
          <div class="space-y-1 text-xs text-guofeng-text-secondary">
            <div class="flex items-center gap-1">
              <Hash class="w-3 h-3" />
              <span>{{ formatNumber(getPlatformMessages('gemini')) }} messages</span>
            </div>
            <div class="flex items-center gap-1">
              <Clock class="w-3 h-3" />
              <span>{{ formatAvgDuration(getPlatformDuration('gemini'), getPlatformSessions('gemini') || 1) }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </GuofengCard>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { BarChart3, Hash, Clock } from 'lucide-vue-next'
import GuofengCard from '@/components/common/GuofengCard.vue'
import UsageStatsChart from '@/components/UsageStatsChart.vue'
import { getDailyStats } from '@/api/client'
import type { DailyStatsResponse, StatsViewMode } from '@/types'

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

// 设置日期范围
const setDateRange = async (days: number) => {
  selectedDays.value = days
  loading.value = true
  try {
    dailyStats.value = await getDailyStats(days)
  } catch (error) {
    console.error('Failed to load daily stats:', error)
  } finally {
    loading.value = false
  }
}

// 加载数据
onMounted(async () => {
  try {
    dailyStats.value = await getDailyStats(selectedDays.value)
  } catch (error) {
    console.error('Failed to load daily stats:', error)
  } finally {
    loading.value = false
  }
})

// 获取平台统计
const getPlatformSessions = (platform: string): number => {
  return dailyStats.value?.summary.by_platform[platform]?.sessions || 0
}

const getPlatformMessages = (platform: string): number => {
  return dailyStats.value?.summary.by_platform[platform]?.messages || 0
}

const getPlatformDuration = (platform: string): number => {
  return dailyStats.value?.summary.by_platform[platform]?.duration_seconds || 0
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
