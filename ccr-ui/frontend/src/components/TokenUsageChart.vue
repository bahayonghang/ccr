<template>
  <div class="token-usage-chart glass-card rounded-2xl p-6 backdrop-blur-xl bg-white/70 dark:bg-gray-800/70 border border-white/20 dark:border-gray-700/50 shadow-xl">
    <!-- Header -->
    <div class="flex items-center justify-between mb-6 flex-wrap gap-3">
      <div class="flex items-center gap-4">
        <h3 class="text-xl font-bold text-gray-900 dark:text-white flex items-center gap-2">
          <span class="text-2xl">📈</span>
          <span>Token 使用趋势</span>
        </h3>
        <!-- Filter Status Badge -->
        <div class="flex items-center gap-2">
          <span class="px-3 py-1.5 rounded-lg bg-blue-500/10 dark:bg-blue-400/10 border border-blue-500/30 dark:border-blue-400/30 text-xs font-medium text-blue-700 dark:text-blue-300">
            {{ getTimeRangeLabel(timeRange) }}
          </span>
          <span
            v-if="selectedModel !== 'all'"
            class="px-3 py-1.5 rounded-lg bg-purple-500/10 dark:bg-purple-400/10 border border-purple-500/30 dark:border-purple-400/30 text-xs font-medium text-purple-700 dark:text-purple-300"
          >
            {{ shortenModelName(selectedModel) }}
          </span>
        </div>
      </div>

      <!-- Legend -->
      <div class="flex items-center space-x-3">
        <button
          class="flex items-center space-x-2 px-3 py-2 rounded-lg transition-all hover:scale-105 border-2"
          :class="activeSeriesSet.has('input') ? 'bg-blue-500/20 dark:bg-blue-400/20 ring-2 ring-blue-500/50 border-blue-500/50' : 'opacity-50 hover:opacity-70 border-transparent'"
          @click="toggleSeries('input')"
        >
          <div class="w-4 h-4 rounded-full bg-blue-500 dark:bg-blue-400 shadow-lg" />
          <span class="text-sm font-bold text-gray-900 dark:text-gray-100">输入</span>
        </button>

        <button
          class="flex items-center space-x-2 px-3 py-2 rounded-lg transition-all hover:scale-105 border-2"
          :class="activeSeriesSet.has('output') ? 'bg-green-500/20 dark:bg-green-400/20 ring-2 ring-green-500/50 border-green-500/50' : 'opacity-50 hover:opacity-70 border-transparent'"
          @click="toggleSeries('output')"
        >
          <div class="w-4 h-4 rounded-full bg-green-500 dark:bg-green-400 shadow-lg" />
          <span class="text-sm font-bold text-gray-900 dark:text-gray-100">输出</span>
        </button>

        <button
          class="flex items-center space-x-2 px-3 py-2 rounded-lg transition-all hover:scale-105 border-2"
          :class="activeSeriesSet.has('cache') ? 'bg-amber-500/20 dark:bg-amber-400/20 ring-2 ring-amber-500/50 border-amber-500/50' : 'opacity-50 hover:opacity-70 border-transparent'"
          @click="toggleSeries('cache')"
        >
          <div class="w-4 h-4 rounded-full bg-amber-500 dark:bg-amber-400 shadow-lg" />
          <span class="text-sm font-bold text-gray-900 dark:text-gray-100">缓存</span>
        </button>
      </div>
    </div>

    <!-- Chart Container -->
    <div
      class="relative w-full bg-gradient-to-br from-gray-50/50 to-gray-100/50 dark:from-gray-900/30 dark:to-gray-800/30 rounded-xl p-6"
      style="height: 500px;"
    >
      <svg
        ref="chartSvg"
        class="w-full h-full"
        viewBox="0 0 1000 500"
        preserveAspectRatio="xMidYMid meet"
      >
        <!-- Gradients -->
        <defs>
          <linearGradient
            id="inputGradient"
            x1="0%"
            y1="0%"
            x2="0%"
            y2="100%"
          >
            <stop
              offset="0%"
              style="stop-color:#3b82f6;stop-opacity:0.8"
            />
            <stop
              offset="100%"
              style="stop-color:#3b82f6;stop-opacity:0.1"
            />
          </linearGradient>
          <linearGradient
            id="outputGradient"
            x1="0%"
            y1="0%"
            x2="0%"
            y2="100%"
          >
            <stop
              offset="0%"
              style="stop-color:#10b981;stop-opacity:0.8"
            />
            <stop
              offset="100%"
              style="stop-color:#10b981;stop-opacity:0.1"
            />
          </linearGradient>
          <linearGradient
            id="cacheGradient"
            x1="0%"
            y1="0%"
            x2="0%"
            y2="100%"
          >
            <stop
              offset="0%"
              style="stop-color:#f59e0b;stop-opacity:0.8"
            />
            <stop
              offset="100%"
              style="stop-color:#f59e0b;stop-opacity:0.1"
            />
          </linearGradient>
        </defs>

        <!-- Grid lines -->
        <g
          class="grid"
          opacity="0.2"
        >
          <line
            v-for="i in yAxisSteps"
            :key="`h-${i}`"
            :x1="80"
            :y1="460 - (i * yStepSize)"
            :x2="940"
            :y2="460 - (i * yStepSize)"
            stroke="currentColor"
            class="text-gray-400 dark:text-gray-500"
            stroke-dasharray="5,5"
            stroke-width="1.5"
          />
        </g>

        <!-- Area charts -->
        <g class="charts">
          <!-- Cache Read Area (bottom layer) -->
          <path
            v-if="activeSeriesSet.has('cache')"
            :d="cachePath"
            fill="url(#cacheGradient)"
            stroke="#f59e0b"
            stroke-width="2"
            class="transition-all duration-300"
          />

          <!-- Output Area (middle layer) -->
          <path
            v-if="activeSeriesSet.has('output')"
            :d="outputPath"
            fill="url(#outputGradient)"
            stroke="#10b981"
            stroke-width="2"
            class="transition-all duration-300"
          />

          <!-- Input Area (top layer) -->
          <path
            v-if="activeSeriesSet.has('input')"
            :d="inputPath"
            fill="url(#inputGradient)"
            stroke="#3b82f6"
            stroke-width="2"
            class="transition-all duration-300"
          />
        </g>

        <!-- X-axis labels -->
        <g
          class="x-axis text-gray-700 dark:text-gray-300"
          font-size="13"
          fill="currentColor"
          font-weight="600"
        >
          <text
            v-for="(label, i) in xLabels"
            :key="`x-${i}`"
            :x="80 + (i * (860 / Math.max(1, xLabels.length - 1)))"
            y="485"
            text-anchor="middle"
          >
            {{ label }}
          </text>
        </g>

        <!-- Y-axis labels -->
        <g
          class="y-axis text-gray-700 dark:text-gray-300"
          font-size="13"
          fill="currentColor"
          font-weight="600"
        >
          <text
            v-for="(label, i) in yLabels"
            :key="`y-${i}`"
            x="70"
            :y="465 - (i * yStepSize)"
            text-anchor="end"
          >
            {{ label }}
          </text>
        </g>
      </svg>
    </div>

    <!-- Empty state -->
    <div
      v-if="chartData.length === 0"
      class="absolute inset-6 flex items-center justify-center bg-gradient-to-br from-gray-50/80 to-gray-100/80 dark:from-gray-900/50 dark:to-gray-800/50 rounded-xl backdrop-blur-sm"
    >
      <div class="text-center">
        <svg
          class="w-20 h-20 mx-auto mb-4 text-gray-400 dark:text-gray-500"
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
        <p class="text-base font-semibold text-gray-600 dark:text-gray-400">
          所选筛选条件下无可用数据
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import type { UsageRecord, TimeRange } from '@/types'

interface Props {
  records: UsageRecord[]
  timeRange: TimeRange
  selectedModel: string
}

const props = defineProps<Props>()

const activeSeries = ref<Set<string>>(new Set(['input', 'output']))
const activeSeriesSet = computed(() => activeSeries.value)

const toggleSeries = (series: string) => {
  const newSet = new Set(activeSeries.value)
  if (newSet.has(series)) {
    newSet.delete(series)
  } else {
    newSet.add(series)
  }
  activeSeries.value = newSet
}

// Get time range label for display
const getTimeRangeLabel = (range: TimeRange): string => {
  const labels: Record<TimeRange, string> = {
    '5h': '最近 5 小时',
    'today': '今天',
    '7d': '最近 7 天',
    'week': '本周',
    'month': '本月',
    'all': '全部时间'
  }
  return labels[range] || range
}

// Shorten model name for display
const shortenModelName = (model: string): string => {
  if (model === 'all') return '全部模型'
  // Extract key parts of model name
  const parts = model.split('-')
  if (parts.length > 3) {
    // For names like "claude-sonnet-4-5-20250929"
    return parts.slice(0, 3).join('-')
  }
  return model
}

// Helper to format date based on time range
const formatDate = (date: Date, format: string): string => {
  const month = date.toLocaleDateString('en-US', { month: 'short' })
  const day = date.getDate().toString().padStart(2, '0')
  const hours = date.getHours().toString().padStart(2, '0')
  const minutes = date.getMinutes().toString().padStart(2, '0')

  if (format === 'week') {
    return `${month} ${day}`
  } else if (format === 'day') {
    return `${month} ${day}`
  } else {
    return `${hours}:${minutes}`
  }
}

// Aggregate data by time intervals
const chartData = computed(() => {
  const intervals: Record<string, { input: number; output: number; cache: number; time: string; timestamp: number }> = {}

  // Determine interval format based on time range
  let format: string

  if (props.timeRange === 'all' || props.timeRange === 'month') {
    format = 'week' // Weekly or daily intervals
  } else if (props.timeRange === '7d' || props.timeRange === 'week') {
    format = 'day' // Daily intervals
  } else {
    format = 'hour' // Hourly intervals
  }

  // Aggregate records with timestamp tracking
  props.records.forEach(record => {
    const time = new Date(record.timestamp)
    const key = formatDate(time, format)

    if (!intervals[key]) {
      intervals[key] = {
        input: 0,
        output: 0,
        cache: 0,
        time: key,
        timestamp: time.getTime() // Add timestamp for sorting
      }
    } else {
      // Update timestamp to the latest record in this interval
      const currentTime = time.getTime()
      if (currentTime > intervals[key].timestamp) {
        intervals[key].timestamp = currentTime
      }
    }

    if (record.usage) {
      intervals[key].input += record.usage.input_tokens || 0
      intervals[key].output += record.usage.output_tokens || 0
      intervals[key].cache += record.usage.cache_read_input_tokens || 0
    }
  })

  // Convert to array, sort by timestamp (ascending), and limit
  return Object.values(intervals)
    .sort((a, b) => a.timestamp - b.timestamp) // Sort from old to new
    .slice(0, 50) // Limit to 50 points for performance
})

// Calculate nice step size for the given range
const calculateNiceStep = (rawStep: number): number => {
  const exponent = Math.floor(Math.log10(rawStep))
  const fraction = rawStep / Math.pow(10, exponent)

  let niceFraction: number
  if (fraction <= 1) niceFraction = 1
  else if (fraction <= 2) niceFraction = 2
  else if (fraction <= 5) niceFraction = 5
  else niceFraction = 10

  return niceFraction * Math.pow(10, exponent)
}

// Calculate max value for Y-axis with proper adaptation
const maxValue = computed(() => {
  if (chartData.value.length === 0) return 100

  // Find actual maximum value across all data series
  const actualMax = Math.max(
    ...chartData.value.map(d => Math.max(d.input, d.output, d.cache)),
    0 // Ensure at least 0
  )

  // If all data is 0, return a small default
  if (actualMax === 0) return 100

  // Target number of steps (we want 5-6 steps)
  const targetSteps = 5

  // Calculate raw step size
  const rawStep = actualMax / targetSteps

  // Get nice step size
  const niceStep = calculateNiceStep(rawStep)

  // Calculate max value by rounding up to nearest step
  // Add some padding to ensure data doesn't touch the top
  const stepsNeeded = Math.ceil(actualMax / niceStep) + 1

  return niceStep * stepsNeeded
})

// Calculate actual step value for Y-axis
const yStepValue = computed(() => {
  if (chartData.value.length === 0) return 20

  const actualMax = Math.max(
    ...chartData.value.map(d => Math.max(d.input, d.output, d.cache)),
    0
  )

  if (actualMax === 0) return 20

  const targetSteps = 5
  const rawStep = actualMax / targetSteps
  return calculateNiceStep(rawStep)
})

// Calculate optimal number of Y-axis steps based on actual max and step value
const yAxisSteps = computed(() => {
  const steps = Math.ceil(maxValue.value / yStepValue.value)
  return Math.max(4, Math.min(6, steps)) // Keep between 4-6 steps
})

// Calculate pixel height for each step
const yStepSize = computed(() => {
  const chartHeight = 440 // 460 - 20 (top margin)
  return chartHeight / yAxisSteps.value
})

// Generate path data for SVG with smooth curves
const generatePath = (getValue: (d: any) => number): string => {
  if (chartData.value.length === 0) return ''

  // Chart area coordinates (updated for new viewBox)
  const chartLeft = 80
  const chartRight = 940
  const chartTop = 20
  const chartBottom = 460

  const width = chartRight - chartLeft  // 860px
  const height = chartBottom - chartTop  // 440px

  const xScale = width / Math.max(1, chartData.value.length - 1)
  const yScale = height / maxValue.value

  let path = `M ${chartLeft} ${chartBottom}`

  chartData.value.forEach((d, i) => {
    const x = chartLeft + (i * xScale)
    const y = chartBottom - (getValue(d) * yScale)

    if (i === 0) {
      path += ` L ${x} ${y}`
    } else {
      path += ` L ${x} ${y}`
    }
  })

  // Close the path to create area
  const lastX = chartLeft + ((chartData.value.length - 1) * xScale)
  path += ` L ${lastX} ${chartBottom} Z`

  return path
}

const inputPath = computed(() => generatePath(d => d.input))
const outputPath = computed(() => generatePath(d => d.output))
const cachePath = computed(() => generatePath(d => d.cache))

// X-axis labels (time labels)
const xLabels = computed(() => {
  if (chartData.value.length === 0) return []

  // Show max 10 labels
  const step = Math.ceil(chartData.value.length / 10)
  return chartData.value
    .filter((_, i) => i % step === 0)
    .map(d => d.time)
})

// Y-axis labels (token counts) with smart formatting
const yLabels = computed(() => {
  const stepValue = yStepValue.value
  const steps = yAxisSteps.value

  return Array.from({ length: steps + 1 }, (_, i) => {
    const value = i * stepValue

    // Format large numbers nicely
    if (value >= 1000000) {
      const millions = value / 1000000
      return millions % 1 === 0 ? `${millions.toFixed(0)}M` : `${millions.toFixed(1)}M`
    }
    if (value >= 1000) {
      const thousands = value / 1000
      return thousands % 1 === 0 ? `${thousands.toFixed(0)}K` : `${thousands.toFixed(1)}K`
    }
    return value.toFixed(0)
  })
})
</script>

<style scoped>
.token-usage-chart {
  position: relative;
  transition: all 0.3s ease;
}

button {
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

button:hover {
  transform: scale(1.05);
}

path {
  transition: opacity 0.3s ease, fill 0.3s ease, stroke 0.3s ease;
  filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.1));
}

path:hover {
  filter: drop-shadow(0 4px 8px rgba(0, 0, 0, 0.15));
}

.glass-card {
  box-shadow: 0 8px 32px 0 rgba(31, 38, 135, 0.15);
}

/* Chart container enhancements */
.token-usage-chart > div:last-of-type {
  box-shadow: inset 0 2px 8px rgba(0, 0, 0, 0.05);
}

/* Axis labels animation */
.x-axis text, .y-axis text {
  transition: all 0.2s ease;
}

svg:hover .x-axis text,
svg:hover .y-axis text {
  opacity: 1;
}
</style>
