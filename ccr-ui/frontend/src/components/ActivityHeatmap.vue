<template>
  <div class="activity-heatmap rounded-2xl p-6 bg-white/80 dark:bg-gray-800/80 border border-gray-200/50 dark:border-gray-700/50 shadow-lg backdrop-blur-sm">
    <!-- Header -->
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-lg font-bold text-gray-900 dark:text-white flex items-center gap-2">
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
            d="M17.657 18.657A8 8 0 016.343 7.343S7 9 9 10c0-2 .5-5 2.986-7C14 5 16.09 5.777 17.656 7.343A7.975 7.975 0 0120 13a7.975 7.975 0 01-2.343 5.657z"
          />
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M9.879 16.121A3 3 0 1012.015 11L11 14H9c0 .768.293 1.536.879 2.121z"
          />
        </svg>
        <span>{{ $t('activityHeatmap.title', '活动热力图') }}</span>
      </h3>
      <!-- Legend -->
      <div class="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-gray-100/80 dark:bg-gray-700/50 border border-gray-200/50 dark:border-gray-600/30">
        <span class="text-xs font-medium text-gray-500 dark:text-gray-400">{{ $t('activityHeatmap.less', '较少') }}</span>
        <div class="flex gap-0.5">
          <div
            v-for="level in [0, 1, 2, 3, 4]"
            :key="level"
            class="w-2.5 h-2.5 rounded-sm transition-all"
            :class="getLevelClass(level)"
          />
        </div>
        <span class="text-xs font-medium text-gray-500 dark:text-gray-400">{{ $t('activityHeatmap.more', '较多') }}</span>
      </div>
    </div>

    <!-- Loading -->
    <div
      v-if="loading"
      class="flex items-center justify-center py-12"
    >
      <div class="animate-spin rounded-full h-8 w-8 border-2 border-orange-200 border-t-orange-500" />
    </div>

    <!-- Heatmap Grid - Compact GitHub Style -->
    <div
      v-else
      class="rounded-xl bg-gray-50/50 dark:bg-gray-900/30 p-4"
    >
      <!-- Month Labels - Aligned with weeks -->
      <div class="flex mb-2">
        <div class="w-8 flex-shrink-0" />
        <div
          class="flex relative"
          style="width: calc(53 * 13px);"
        >
          <div
            v-for="(month, idx) in monthLabels"
            :key="idx"
            class="absolute text-xs font-medium text-gray-500 dark:text-gray-400"
            :style="{ left: `${month.weekOffset * 13}px` }"
          >
            {{ month.name }}
          </div>
        </div>
      </div>

      <!-- Grid with Day Labels -->
      <div class="flex gap-1">
        <!-- Day Labels (Mon, Wed, Fri) -->
        <div class="flex flex-col gap-[3px] text-xs font-medium text-gray-500 dark:text-gray-400 w-6 flex-shrink-0">
          <div class="h-[10px]" />
          <div class="h-[10px] flex items-center">
            一
          </div>
          <div class="h-[10px]" />
          <div class="h-[10px] flex items-center">
            三
          </div>
          <div class="h-[10px]" />
          <div class="h-[10px] flex items-center">
            五
          </div>
          <div class="h-[10px]" />
        </div>

        <!-- Weeks Grid -->
        <div class="flex gap-[3px]">
          <div
            v-for="(week, weekIndex) in weeks"
            :key="weekIndex"
            class="flex flex-col gap-[3px]"
          >
            <div
              v-for="(day, dayIndex) in week"
              :key="dayIndex"
              class="relative group"
            >
              <div
                v-if="day"
                class="w-[10px] h-[10px] rounded-sm transition-all hover:ring-1 hover:ring-orange-400/70 cursor-pointer"
                :class="getLevelClass(day.level)"
                @mouseenter="showTooltip(day, $event)"
                @mouseleave="hideTooltip"
              />
              <div
                v-else
                class="w-[10px] h-[10px]"
              />
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Tooltip -->
    <Teleport to="body">
      <div
        v-if="tooltip.visible"
        class="fixed z-[9999] px-3 py-2 text-sm backdrop-blur-xl bg-gray-900/95 text-white rounded-lg shadow-2xl pointer-events-none border border-gray-700/50"
        :style="{
          left: `${tooltip.x}px`,
          top: `${tooltip.y}px`,
          transform: 'translate(-50%, -120%)'
        }"
      >
        <div class="font-semibold">
          {{ tooltip.date }}
        </div>
        <div class="text-xs text-gray-300">
          {{ formatNumber(tooltip.count) }} tokens
        </div>
      </div>
    </Teleport>

    <!-- Summary Stats -->
    <div class="mt-4 pt-4 border-t border-gray-200/50 dark:border-gray-700/50">
      <div class="grid grid-cols-2 gap-4">
        <!-- Active Days -->
        <div class="flex items-center gap-3 px-4 py-3 rounded-xl bg-gradient-to-br from-orange-50/80 to-amber-50/80 dark:from-orange-900/20 dark:to-amber-900/20 border border-orange-200/50 dark:border-orange-800/50">
          <div class="p-2 rounded-lg bg-orange-500/20 dark:bg-orange-400/20">
            <svg
              class="w-5 h-5 text-orange-600 dark:text-orange-400"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"
              />
            </svg>
          </div>
          <div>
            <p class="text-xs font-medium text-gray-600 dark:text-gray-400">
              {{ $t('activityHeatmap.activeDays', '活跃天数') }}
            </p>
            <p class="text-xl font-bold text-gray-900 dark:text-white">
              {{ heatmapData?.active_days ?? 0 }}
            </p>
          </div>
        </div>
        
        <!-- Total Tokens -->
        <div class="flex items-center gap-3 px-4 py-3 rounded-xl bg-gradient-to-br from-red-50/80 to-rose-50/80 dark:from-red-900/20 dark:to-rose-900/20 border border-red-200/50 dark:border-red-800/50">
          <div class="p-2 rounded-lg bg-red-500/20 dark:bg-red-400/20">
            <svg
              class="w-5 h-5 text-red-600 dark:text-red-400"
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
          </div>
          <div>
            <p class="text-xs font-medium text-gray-600 dark:text-gray-400">
              {{ $t('activityHeatmap.totalTokens', '总 Token 数') }}
            </p>
            <p class="text-xl font-bold text-gray-900 dark:text-white">
              {{ formatNumber(heatmapData?.total_tokens ?? 0) }}
            </p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import { getHeatmapData, type HeatmapData } from '@/api/modules/stats'

interface DayData {
  date: string
  dateKey: string
  count: number
  level: number
}

interface MonthLabel {
  name: string
  weekOffset: number
}

// State
const loading = ref(true)
const heatmapData = ref<HeatmapData | null>(null)

// Tooltip
const tooltip = ref({
  visible: false,
  date: '',
  count: 0,
  x: 0,
  y: 0
})

// Start date calculation - aligned to Sunday before 364 days ago
const startDate = computed(() => {
  const today = new Date()
  const start = new Date(today)
  start.setDate(start.getDate() - 364)
  // Round to Sunday (start of week)
  const dayOfWeek = start.getDay()
  start.setDate(start.getDate() - dayOfWeek)
  return start
})

// Load data on mount
onMounted(async () => {
  try {
    heatmapData.value = await getHeatmapData()
  } catch (e) {
    console.error('Failed to load heatmap data:', e)
    heatmapData.value = { data: {}, max_value: 0, total_tokens: 0, active_days: 0 }
  } finally {
    loading.value = false
  }
})

// Generate weeks array (53 weeks, 7 days each)
const weeks = computed(() => {
  const result: (DayData | null)[][] = []
  const today = new Date()
  const maxValue = heatmapData.value?.max_value || 1
  const data = heatmapData.value?.data || {}
  const start = startDate.value

  // Generate 53 weeks
  for (let week = 0; week < 53; week++) {
    const weekDays: (DayData | null)[] = []
    
    for (let day = 0; day < 7; day++) {
      const currentDate = new Date(start)
      currentDate.setDate(currentDate.getDate() + (week * 7) + day)
      
      // Skip future dates
      if (currentDate > today) {
        weekDays.push(null)
        continue
      }

      const dateKey = currentDate.toISOString().split('T')[0]
      const count = data[dateKey] || 0
      
      // Calculate level
      let level = 0
      if (count > 0) {
        const ratio = count / maxValue
        level = ratio > 0.75 ? 4 : ratio > 0.5 ? 3 : ratio > 0.25 ? 2 : 1
      }

      weekDays.push({
        date: currentDate.toLocaleDateString('zh-CN', {
          year: 'numeric',
          month: 'short',
          day: 'numeric'
        }),
        dateKey,
        count,
        level
      })
    }
    
    result.push(weekDays)
  }

  return result
})

// Month labels - calculated based on actual week positions
const monthLabels = computed((): MonthLabel[] => {
  const labels: MonthLabel[] = []
  const start = startDate.value
  let lastMonth = -1

  for (let week = 0; week < 53; week++) {
    // Get the date for the first day of this week
    const weekStart = new Date(start)
    weekStart.setDate(weekStart.getDate() + (week * 7))
    
    const month = weekStart.getMonth()
    
    // Only add label when month changes
    if (month !== lastMonth) {
      labels.push({
        name: weekStart.toLocaleDateString('zh-CN', { month: 'short' }),
        weekOffset: week
      })
      lastMonth = month
    }
  }

  return labels
})

// Warm color gradient (orange to red)
const getLevelClass = (level: number): string => {
  const classes = [
    'bg-gray-200/80 dark:bg-gray-700/60',
    'bg-orange-200/90 dark:bg-orange-800/70',
    'bg-orange-400/90 dark:bg-orange-600/80',
    'bg-red-400/90 dark:bg-red-500/80',
    'bg-red-600/95 dark:bg-red-400/90',
  ]
  return classes[level] || classes[0]
}

// Tooltip handlers
const showTooltip = (day: DayData, event: MouseEvent) => {
  tooltip.value = {
    visible: true,
    date: day.date,
    count: day.count,
    x: event.clientX,
    y: event.clientY
  }
}

const hideTooltip = () => {
  tooltip.value.visible = false
}

// Format number
const formatNumber = (num: number): string => {
  if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`
  if (num >= 1000) return `${(num / 1000).toFixed(1)}K`
  return num.toString()
}
</script>

<style scoped>
.activity-heatmap {
  position: relative;
}
</style>
