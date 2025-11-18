<template>
  <div class="activity-heatmap glass-card rounded-2xl p-6 backdrop-blur-xl bg-white/70 dark:bg-gray-800/70 border border-white/20 dark:border-gray-700/50 shadow-xl">
    <!-- Header -->
    <div class="flex items-center justify-between mb-6">
      <h3 class="text-xl font-bold text-gray-900 dark:text-white flex items-center gap-2">
        <span class="text-2xl">📅</span>
        <span>活动热力图</span>
      </h3>
      <div class="flex items-center gap-3 px-4 py-2 rounded-lg bg-gradient-to-r from-gray-50/80 to-gray-100/80 dark:from-gray-900/50 dark:to-gray-800/50 border border-gray-200/50 dark:border-gray-700/50">
        <span class="text-xs font-medium text-gray-600 dark:text-gray-400">较少</span>
        <div class="flex gap-1.5">
          <div
            v-for="level in [0, 1, 2, 3, 4]"
            :key="level"
            class="w-3.5 h-3.5 rounded transition-all hover:scale-125"
            :class="getLevelClass(level)"
          />
        </div>
        <span class="text-xs font-medium text-gray-600 dark:text-gray-400">较多</span>
      </div>
    </div>

    <!-- Heatmap Grid -->
    <div class="overflow-x-auto bg-gradient-to-br from-gray-50/50 to-gray-100/50 dark:from-gray-900/30 dark:to-gray-800/30 rounded-xl p-4">
      <div class="inline-flex flex-col gap-1">
        <!-- Month labels -->
        <div class="flex ml-8 mb-2">
          <div
            v-for="(month, index) in monthLabels"
            :key="index"
            class="text-xs font-semibold text-gray-700 dark:text-gray-300"
            :style="{ width: `${month.weeks * 14}px`, marginLeft: index > 0 ? '2px' : '0' }"
          >
            {{ month.name }}
          </div>
        </div>

        <!-- Weekday labels + cells -->
        <div class="flex gap-1.5">
          <!-- Weekday labels -->
          <div class="flex flex-col gap-1.5 text-xs font-medium text-gray-700 dark:text-gray-300 justify-around pr-2">
            <div style="height: 12px;" />
            <div style="height: 12px;">
              周一
            </div>
            <div style="height: 12px;" />
            <div style="height: 12px;">
              周三
            </div>
            <div style="height: 12px;" />
            <div style="height: 12px;">
              周五
            </div>
            <div style="height: 12px;" />
          </div>

          <!-- Heatmap cells -->
          <div class="flex gap-1.5">
            <div
              v-for="(week, weekIndex) in weeks"
              :key="weekIndex"
              class="flex flex-col gap-1.5"
            >
              <div
                v-for="(day, dayIndex) in week"
                :key="dayIndex"
                class="relative group"
              >
                <div
                  v-if="day"
                  class="w-[12px] h-[12px] rounded transition-all duration-200 hover:ring-2 hover:ring-blue-500/70 dark:hover:ring-blue-400/70 hover:scale-125 cursor-pointer shadow-sm"
                  :class="getLevelClass(day.level)"
                  @mouseenter="showTooltip(day, $event)"
                  @mouseleave="hideTooltip"
                />
                <div
                  v-else
                  class="w-[12px] h-[12px]"
                />
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Tooltip -->
    <div
      v-if="tooltip.visible"
      class="fixed z-50 px-4 py-3 text-sm backdrop-blur-xl bg-gray-900/90 dark:bg-gray-800/90 text-white rounded-xl shadow-2xl pointer-events-none border border-gray-700/50"
      :style="{
        left: `${tooltip.x}px`,
        top: `${tooltip.y}px`,
        transform: 'translate(-50%, -120%)'
      }"
    >
      <div class="font-bold text-base mb-1">
        {{ tooltip.date }}
      </div>
      <div class="text-xs text-gray-300 flex items-center gap-1">
        <span class="inline-block w-2 h-2 rounded-full bg-green-400" />
        <span>{{ formatNumber(tooltip.count) }} tokens</span>
      </div>
    </div>

    <!-- Summary -->
    <div class="mt-6 pt-4 border-t border-gray-200/50 dark:border-gray-700/50">
      <div class="grid grid-cols-2 gap-4">
        <div class="flex items-center gap-3 px-4 py-3 rounded-xl bg-gradient-to-br from-blue-50/80 to-indigo-50/80 dark:from-blue-900/20 dark:to-indigo-900/20 border border-blue-200/50 dark:border-blue-800/50">
          <div class="p-2 rounded-lg bg-blue-500/20 dark:bg-blue-400/20">
            <svg
              class="w-5 h-5 text-blue-600 dark:text-blue-400"
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
              活跃天数
            </p>
            <p class="text-xl font-bold text-gray-900 dark:text-white">
              {{ totalDays }}
            </p>
          </div>
        </div>
        <div class="flex items-center gap-3 px-4 py-3 rounded-xl bg-gradient-to-br from-green-50/80 to-emerald-50/80 dark:from-green-900/20 dark:to-emerald-900/20 border border-green-200/50 dark:border-green-800/50">
          <div class="p-2 rounded-lg bg-green-500/20 dark:bg-green-400/20">
            <svg
              class="w-5 h-5 text-green-600 dark:text-green-400"
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
              总 Token 数
            </p>
            <p class="text-xl font-bold text-gray-900 dark:text-white">
              {{ formatNumber(totalActivity) }}
            </p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import type { UsageRecord } from '@/types'

interface Props {
  records: UsageRecord[]
}

const props = defineProps<Props>()

interface DayData {
  date: string
  count: number
  level: number
}

// Tooltip state
const tooltip = ref({
  visible: false,
  date: '',
  count: 0,
  x: 0,
  y: 0
})

// Generate last 365 days grid data
const weeks = computed(() => {
  const result: (DayData | null)[][] = []
  const today = new Date()
  const startDate = new Date(today)
  startDate.setDate(startDate.getDate() - 364) // Last 365 days

  // Round to Sunday
  const dayOfWeek = startDate.getDay()
  startDate.setDate(startDate.getDate() - dayOfWeek)

  // Aggregate usage by date
  const usageByDate = new Map<string, number>()
  props.records.forEach(record => {
    const date = new Date(record.timestamp)
    const dateKey = date.toISOString().split('T')[0]
    const tokens = (record.usage?.input_tokens || 0) +
                   (record.usage?.output_tokens || 0) +
                   (record.usage?.cache_read_input_tokens || 0)
    usageByDate.set(dateKey, (usageByDate.get(dateKey) || 0) + tokens)
  })

  // Find max for level calculation
  const maxCount = Math.max(...Array.from(usageByDate.values()), 1)

  // Generate weeks
  let currentWeek: (DayData | null)[] = []
  const currentDate = new Date(startDate)
  const endDate = new Date(today)
  endDate.setHours(23, 59, 59, 999)

  while (currentDate <= endDate) {
    const dateKey = currentDate.toISOString().split('T')[0]
    const count = usageByDate.get(dateKey) || 0

    // Calculate level (0-4)
    let level = 0
    if (count > 0) {
      const ratio = count / maxCount
      if (ratio > 0.75) level = 4
      else if (ratio > 0.5) level = 3
      else if (ratio > 0.25) level = 2
      else level = 1
    }

    currentWeek.push({
      date: currentDate.toLocaleDateString('en-US', {
        year: 'numeric',
        month: 'short',
        day: 'numeric'
      }),
      count,
      level
    })

    // Start new week on Sunday
    if (currentDate.getDay() === 6) {
      result.push(currentWeek)
      currentWeek = []
    }

    currentDate.setDate(currentDate.getDate() + 1)
  }

  // Push remaining days
  if (currentWeek.length > 0) {
    // Fill remaining days with null
    while (currentWeek.length < 7) {
      currentWeek.push(null)
    }
    result.push(currentWeek)
  }

  return result
})

// Month labels
const monthLabels = computed(() => {
  const labels: { name: string; weeks: number }[] = []
  const today = new Date()
  const startDate = new Date(today)
  startDate.setDate(startDate.getDate() - 364)

  let currentMonth = startDate.getMonth()
  let weekCount = 0

  for (let i = 0; i < weeks.value.length; i++) {
    const firstDay = weeks.value[i].find(day => day !== null)
    if (firstDay) {
      const date = new Date(firstDay.date)
      const month = date.getMonth()

      if (month !== currentMonth) {
        if (weekCount > 0) {
          labels.push({
            name: new Date(2000, currentMonth).toLocaleDateString('en-US', { month: 'short' }),
            weeks: weekCount
          })
        }
        currentMonth = month
        weekCount = 1
      } else {
        weekCount++
      }
    }
  }

  // Push last month
  if (weekCount > 0) {
    labels.push({
      name: new Date(2000, currentMonth).toLocaleDateString('en-US', { month: 'short' }),
      weeks: weekCount
    })
  }

  return labels
})

// Get CSS class for activity level
const getLevelClass = (level: number): string => {
  const classes = [
    'bg-gray-200/80 dark:bg-gray-700/60 border border-gray-300/50 dark:border-gray-600/30',           // Level 0 (no activity)
    'bg-emerald-200/90 dark:bg-emerald-800/70 border border-emerald-300/60 dark:border-emerald-700/40',         // Level 1
    'bg-emerald-400/90 dark:bg-emerald-600/80 border border-emerald-500/60 dark:border-emerald-500/50',         // Level 2
    'bg-emerald-600/90 dark:bg-emerald-400/80 border border-emerald-700/60 dark:border-emerald-300/50',         // Level 3
    'bg-emerald-700/95 dark:bg-emerald-300/90 border border-emerald-800/70 dark:border-emerald-200/60',         // Level 4
  ]
  return classes[level] || classes[0]
}

// Show tooltip
const showTooltip = (day: DayData, event: MouseEvent) => {
  tooltip.value = {
    visible: true,
    date: day.date,
    count: day.count,
    x: event.clientX,
    y: event.clientY
  }
}

// Hide tooltip
const hideTooltip = () => {
  tooltip.value.visible = false
}

// Total days with activity
const totalDays = computed(() => {
  return weeks.value.flat().filter(day => day && day.count > 0).length
})

// Total activity
const totalActivity = computed(() => {
  return weeks.value.flat().reduce((sum, day) => {
    return sum + (day?.count || 0)
  }, 0)
})

// Format number with K/M suffix
const formatNumber = (num: number): string => {
  if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`
  if (num >= 1000) return `${(num / 1000).toFixed(1)}K`
  return num.toString()
}
</script>

<style scoped>
.activity-heatmap {
  position: relative;
  transition: all 0.3s ease;
}

.glass-card {
  box-shadow: 0 8px 32px 0 rgba(31, 38, 135, 0.15);
}
</style>
