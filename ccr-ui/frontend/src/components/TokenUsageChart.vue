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
      class="relative w-full bg-gradient-to-br from-gray-50/50 to-gray-100/50 dark:from-gray-900/30 dark:to-gray-800/30 rounded-xl p-4"
      style="height: 400px;"
    >
      <svg
        ref="chartSvg"
        class="w-full h-full"
        viewBox="0 0 800 400"
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
          opacity="0.15"
        >
          <line
            v-for="i in 6"
            :key="`h-${i}`"
            :x1="60"
            :y1="370 - (i * 60)"
            :x2="750"
            :y2="370 - (i * 60)"
            stroke="currentColor"
            class="text-gray-400 dark:text-gray-500"
            stroke-dasharray="4,4"
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
          font-size="11"
          fill="currentColor"
          font-weight="500"
        >
          <text
            v-for="(label, i) in xLabels"
            :key="`x-${i}`"
            :x="60 + (i * (690 / Math.max(1, xLabels.length - 1)))"
            y="388"
            text-anchor="middle"
          >
            {{ label }}
          </text>
        </g>

        <!-- Y-axis labels -->
        <g
          class="y-axis text-gray-700 dark:text-gray-300"
          font-size="11"
          fill="currentColor"
          font-weight="500"
        >
          <text
            v-for="(label, i) in yLabels"
            :key="`y-${i}`"
            x="50"
            :y="374 - (i * 60)"
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
      class="absolute inset-0 flex items-center justify-center bg-gradient-to-br from-gray-50/50 to-gray-100/50 dark:from-gray-900/30 dark:to-gray-800/30 rounded-xl"
    >
      <div class="text-center">
        <svg
          class="w-16 h-16 mx-auto mb-3 text-gray-400 dark:text-gray-500"
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
        <p class="text-sm font-medium text-gray-600 dark:text-gray-400">
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
  const intervals: Record<string, { input: number; output: number; cache: number; time: string }> = {}

  // Determine interval format based on time range
  let format: string

  if (props.timeRange === 'all' || props.timeRange === 'month') {
    format = 'week' // Weekly or daily intervals
  } else if (props.timeRange === '7d' || props.timeRange === 'week') {
    format = 'day' // Daily intervals
  } else {
    format = 'hour' // Hourly intervals
  }

  // Aggregate records
  props.records.forEach(record => {
    const time = new Date(record.timestamp)
    const key = formatDate(time, format)

    if (!intervals[key]) {
      intervals[key] = { input: 0, output: 0, cache: 0, time: key }
    }

    if (record.usage) {
      intervals[key].input += record.usage.input_tokens || 0
      intervals[key].output += record.usage.output_tokens || 0
      intervals[key].cache += record.usage.cache_read_input_tokens || 0
    }
  })

  // Convert to array and sort by time
  return Object.values(intervals).slice(0, 50) // Limit to 50 points for performance
})

// Calculate max value for Y-axis scaling
const maxValue = computed(() => {
  if (chartData.value.length === 0) return 1000

  const max = Math.max(
    ...chartData.value.map(d => Math.max(d.input, d.output, d.cache))
  )

  // Add 10% padding to prevent data touching the top
  const maxWithPadding = max * 1.1

  // Round up to nice number
  const magnitude = Math.pow(10, Math.floor(Math.log10(maxWithPadding)))
  return Math.ceil(maxWithPadding / magnitude) * magnitude
})

// Generate path data for SVG
const generatePath = (getValue: (d: any) => number): string => {
  if (chartData.value.length === 0) return ''

  // Chart area coordinates
  const chartLeft = 60
  const chartRight = 750
  const chartTop = 10
  const chartBottom = 370

  const width = chartRight - chartLeft  // 690px
  const height = chartBottom - chartTop  // 360px

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

// Y-axis labels (token counts)
const yLabels = computed(() => {
  const max = maxValue.value
  const step = max / 5

  return [0, 1, 2, 3, 4, 5].map(i => {
    const value = i * step
    if (value >= 1000000) return `${(value / 1000000).toFixed(1)}M`
    if (value >= 1000) return `${(value / 1000).toFixed(1)}K`
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
}

.glass-card {
  box-shadow: 0 8px 32px 0 rgba(31, 38, 135, 0.15);
}
</style>
