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
      ref="chartContainer"
      class="relative w-full bg-gradient-to-br from-gray-50/50 to-gray-100/50 dark:from-gray-900/30 dark:to-gray-800/30 rounded-xl p-6"
      style="height: 500px;"
    >
      <svg
        ref="chartSvg"
        class="w-full h-full"
        viewBox="0 0 1000 500"
        preserveAspectRatio="xMidYMid meet"
        @mousemove="handleMouseMove"
        @mouseleave="handleMouseLeave"
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
              style="stop-color:#3b82f6;stop-opacity:0.6"
            />
            <stop
              offset="100%"
              style="stop-color:#3b82f6;stop-opacity:0.05"
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
              style="stop-color:#10b981;stop-opacity:0.6"
            />
            <stop
              offset="100%"
              style="stop-color:#10b981;stop-opacity:0.05"
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
              style="stop-color:#f59e0b;stop-opacity:0.6"
            />
            <stop
              offset="100%"
              style="stop-color:#f59e0b;stop-opacity:0.05"
            />
          </linearGradient>
        </defs>

        <!-- Grid lines -->
        <g
          class="grid"
          opacity="0.2"
        >
          <line
            v-for="i in axisConfig.steps"
            :key="`h-${i}`"
            :x1="80"
            :y1="chartBottom - (i * yStepSize)"
            :x2="940"
            :y2="chartBottom - (i * yStepSize)"
            stroke="currentColor"
            class="text-gray-400 dark:text-gray-500"
            stroke-dasharray="5,5"
            stroke-width="1.5"
          />
          <!-- Zero line -->
          <line
            :x1="80"
            :y1="chartBottom"
            :x2="940"
            :y2="chartBottom"
            stroke="currentColor"
            class="text-gray-400 dark:text-gray-500"
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
            class="transition-all duration-300 ease-out"
          />

          <!-- Output Area (middle layer) -->
          <path
            v-if="activeSeriesSet.has('output')"
            :d="outputPath"
            fill="url(#outputGradient)"
            stroke="#10b981"
            stroke-width="2"
            class="transition-all duration-300 ease-out"
          />

          <!-- Input Area (top layer) -->
          <path
            v-if="activeSeriesSet.has('input')"
            :d="inputPath"
            fill="url(#inputGradient)"
            stroke="#3b82f6"
            stroke-width="2"
            class="transition-all duration-300 ease-out"
          />
        </g>

        <!-- X-axis labels -->
        <g
          class="x-axis text-gray-600 dark:text-gray-400"
          font-size="12"
          fill="currentColor"
          font-weight="500"
        >
          <text
            v-for="(label, i) in xLabels"
            :key="`x-${i}`"
            :x="getXPos(label.index)"
            y="485"
            text-anchor="middle"
          >
            {{ label.text }}
          </text>
        </g>

        <!-- Y-axis labels -->
        <g
          class="y-axis text-gray-600 dark:text-gray-400"
          font-size="12"
          fill="currentColor"
          font-weight="500"
        >
          <text
            v-for="i in axisConfig.steps + 1"
            :key="`y-${i-1}`"
            x="70"
            :y="chartBottom - ((i - 1) * yStepSize) + 4"
            text-anchor="end"
          >
            {{ formatValue((i - 1) * axisConfig.step) }}
          </text>
        </g>
        
        <!-- Hover Interaction Layer -->
        <g v-if="hoveredIndex !== -1 && chartData.length > 0">
          <!-- Vertical Line -->
          <line
            :x1="getXPos(hoveredIndex)"
            :y1="chartTop"
            :x2="getXPos(hoveredIndex)"
            :y2="chartBottom"
            stroke="currentColor"
            class="text-gray-400 dark:text-gray-500"
            stroke-width="1"
            stroke-dasharray="4,4"
            opacity="0.5"
          />
            
          <!-- Hover Points -->
          <circle
            v-if="activeSeriesSet.has('input') && hoveredData"
            :cx="getXPos(hoveredIndex)"
            :cy="getYPos(hoveredData.input)"
            r="4"
            fill="#3b82f6"
            stroke="white"
            stroke-width="2"
            class="dark:stroke-gray-800"
          />
          <circle
            v-if="activeSeriesSet.has('output') && hoveredData"
            :cx="getXPos(hoveredIndex)"
            :cy="getYPos(hoveredData.output)"
            r="4"
            fill="#10b981"
            stroke="white"
            stroke-width="2"
            class="dark:stroke-gray-800"
          />
          <circle
            v-if="activeSeriesSet.has('cache') && hoveredData"
            :cx="getXPos(hoveredIndex)"
            :cy="getYPos(hoveredData.cache)"
            r="4"
            fill="#f59e0b"
            stroke="white"
            stroke-width="2"
            class="dark:stroke-gray-800"
          />
        </g>
        
        <!-- Invisible overlay for mouse events -->
        <rect
          x="80"
          y="20"
          width="860"
          height="440"
          fill="transparent"
          class="cursor-crosshair"
        />
      </svg>
      
      <!-- Tooltip HTML Overlay -->
      <div
        v-if="hoveredIndex !== -1 && hoveredData"
        class="absolute pointer-events-none z-10 p-3 rounded-xl backdrop-blur-xl bg-white/90 dark:bg-gray-800/90 border border-gray-200 dark:border-gray-700 shadow-xl text-sm transition-all duration-100"
        :style="{
          left: `${tooltipPos.x}px`,
          top: `${tooltipPos.y}px`,
          transform: 'translate(-50%, -110%)'
        }"
      >
        <p class="font-bold text-gray-900 dark:text-white mb-2">
          {{ hoveredData.time }}
        </p>
        <div class="space-y-1">
          <div
            v-if="activeSeriesSet.has('input')"
            class="flex items-center gap-2 text-gray-600 dark:text-gray-300"
          >
            <div class="w-2 h-2 rounded-full bg-blue-500" />
            <span>输入: <span class="font-mono font-bold text-gray-900 dark:text-white">{{ formatNumber(hoveredData.input) }}</span></span>
          </div>
          <div
            v-if="activeSeriesSet.has('output')"
            class="flex items-center gap-2 text-gray-600 dark:text-gray-300"
          >
            <div class="w-2 h-2 rounded-full bg-green-500" />
            <span>输出: <span class="font-mono font-bold text-gray-900 dark:text-white">{{ formatNumber(hoveredData.output) }}</span></span>
          </div>
          <div
            v-if="activeSeriesSet.has('cache')"
            class="flex items-center gap-2 text-gray-600 dark:text-gray-300"
          >
            <div class="w-2 h-2 rounded-full bg-amber-500" />
            <span>缓存: <span class="font-mono font-bold text-gray-900 dark:text-white">{{ formatNumber(hoveredData.cache) }}</span></span>
          </div>
        </div>
      </div>
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
const chartContainer = ref<HTMLElement | null>(null)

// Chart layout constants
const chartLeft = 80
const chartRight = 940
const chartTop = 20
const chartBottom = 460
const chartWidth = chartRight - chartLeft
const chartHeight = chartBottom - chartTop

// Interaction state
const hoveredIndex = ref(-1)
const tooltipPos = ref({ x: 0, y: 0 })

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
  const parts = model.split('-')
  if (parts.length > 3) {
    return parts.slice(0, 3).join('-')
  }
  return model
}

// Helper to format date
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

// Format values for axis and tooltip
const formatValue = (val: number): string => {
  if (val >= 1000000) {
    const millions = val / 1000000
    return millions % 1 === 0 ? `${millions.toFixed(0)}M` : `${millions.toFixed(1)}M`
  }
  if (val >= 1000) {
    const thousands = val / 1000
    return thousands % 1 === 0 ? `${thousands.toFixed(0)}K` : `${thousands.toFixed(1)}K`
  }
  return val.toFixed(0)
}

const formatNumber = (num: number): string => {
  return new Intl.NumberFormat('en-US').format(num)
}

// Aggregate data
const chartData = computed(() => {
  const intervals: Record<string, { input: number; output: number; cache: number; time: string; timestamp: number }> = {}
  let format: string

  if (props.timeRange === 'all' || props.timeRange === 'month') {
    format = 'week'
  } else if (props.timeRange === '7d' || props.timeRange === 'week') {
    format = 'day'
  } else {
    format = 'hour'
  }

  props.records.forEach(record => {
    const time = new Date(record.timestamp)
    const key = formatDate(time, format)

    if (!intervals[key]) {
      intervals[key] = {
        input: 0,
        output: 0,
        cache: 0,
        time: key,
        timestamp: time.getTime()
      }
    } else {
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

  return Object.values(intervals)
    .sort((a, b) => a.timestamp - b.timestamp)
    .slice(0, 50)
})

const hoveredData = computed(() => {
    if (hoveredIndex.value === -1) return null
    return chartData.value[hoveredIndex.value]
})

// Adaptive Axis Calculation
const axisConfig = computed(() => {
  const data = chartData.value
  if (data.length === 0) return { max: 100, step: 20, steps: 5 }

  // Calculate max value in data
  const maxVal = Math.max(...data.map(d => {
    let m = 0
    if (activeSeriesSet.value.has('input')) m = Math.max(m, d.input)
    if (activeSeriesSet.value.has('output')) m = Math.max(m, d.output)
    if (activeSeriesSet.value.has('cache')) m = Math.max(m, d.cache)
    return m
  }), 0)

  if (maxVal === 0) return { max: 20, step: 4, steps: 5 }

  // Adaptive stepping logic
  const targetSteps = 5
  const rawStep = maxVal / targetSteps
  
  // Calculate nice step
  const exponent = Math.floor(Math.log10(rawStep))
  const fraction = rawStep / Math.pow(10, exponent)
  
  let niceFraction
  if (fraction <= 1) niceFraction = 1
  else if (fraction <= 2) niceFraction = 2
  else if (fraction <= 5) niceFraction = 5
  else niceFraction = 10
  
  const step = niceFraction * Math.pow(10, exponent)
  
  // Calculate steps needed to cover maxVal
  let steps = Math.ceil(maxVal / step)
  
  // Adjust if too few steps (prevent flat look)
  if (steps < 4) steps = 4
  
  return {
    max: steps * step,
    step: step,
    steps: steps
  }
})

// Scaling Helpers
const getXPos = (index: number) => {
    const count = Math.max(1, chartData.value.length - 1)
    const xScale = chartWidth / count
    return chartLeft + (index * xScale)
}

const getYPos = (value: number) => {
    if (axisConfig.value.max === 0) return chartBottom
    const yScale = chartHeight / axisConfig.value.max
    return chartBottom - (value * yScale)
}

const yStepSize = computed(() => {
  return chartHeight / axisConfig.value.steps
})

// Smooth Path Generator (Catmull-Rom spline or Bezier)
const generatePath = (getValue: (d: any) => number): string => {
  const data = chartData.value
  if (data.length === 0) return ''

  let d = `M ${getXPos(0)} ${chartBottom}`
  
  // First point
  d += ` L ${getXPos(0)} ${getYPos(getValue(data[0]))}`
  
  // Curve points
  for (let i = 0; i < data.length - 1; i++) {
    const x0 = getXPos(i)
    const y0 = getYPos(getValue(data[i]))
    const x1 = getXPos(i + 1)
    const y1 = getYPos(getValue(data[i + 1]))
    
    // Simple mid-point smoothing
    const midX = (x0 + x1) / 2
    
    // Using Cubic Bezier: C cp1x cp1y, cp2x cp2y, x y
    // Control points at x-midpoint, preserving y level (monotonic-ish)
    d += ` C ${midX} ${y0}, ${midX} ${y1}, ${x1} ${y1}`
  }
  
  // Close path
  d += ` L ${getXPos(data.length - 1)} ${chartBottom} Z`
  return d
}

const inputPath = computed(() => generatePath(d => d.input))
const outputPath = computed(() => generatePath(d => d.output))
const cachePath = computed(() => generatePath(d => d.cache))

// X-axis labels
const xLabels = computed(() => {
  if (chartData.value.length === 0) return []
  const count = chartData.value.length
  const maxLabels = 8 // Prevent overcrowding
  const step = Math.ceil(count / maxLabels)
  
  return chartData.value
    .map((d, i) => ({ text: d.time, index: i }))
    .filter((_, i) => i % step === 0)
})

// Interaction Handlers
const handleMouseMove = (e: MouseEvent) => {
    if (chartData.value.length === 0) return
    
    // Calculate relative X in SVG coordinate system
    // We need to get the bounding rect of the SVG
    const svgRect = (e.target as Element).closest('svg')?.getBoundingClientRect()
    if (!svgRect) return
    
    // Map mouseX to SVG 0-1000 coordinate
    const relativeX = (e.clientX - svgRect.left) / svgRect.width * 1000
    
    if (relativeX < chartLeft || relativeX > chartRight) {
        hoveredIndex.value = -1
        return
    }
    
    const count = Math.max(1, chartData.value.length - 1)
    const xScale = chartWidth / count
    
    let index = Math.round((relativeX - chartLeft) / xScale)
    index = Math.max(0, Math.min(index, chartData.value.length - 1))
    
    hoveredIndex.value = index
    
    // Update tooltip position relative to chart container
    if (chartContainer.value) {
        const containerRect = chartContainer.value.getBoundingClientRect()
        const xOffset = svgRect.left - containerRect.left
        const yOffset = svgRect.top - containerRect.top

        // Get X pos in SVG coords
        const svgX = getXPos(index)
        // Convert back to container coords
        const scaleX = svgRect.width / 1000
        const scaleY = svgRect.height / 500
        
        // Position tooltip near the top of the highest data point at this index
        let maxY = 0
        if (hoveredData.value) {
            let maxVal = 0
            if (activeSeriesSet.value.has('input')) maxVal = Math.max(maxVal, hoveredData.value.input)
            if (activeSeriesSet.value.has('output')) maxVal = Math.max(maxVal, hoveredData.value.output)
            if (activeSeriesSet.value.has('cache')) maxVal = Math.max(maxVal, hoveredData.value.cache)
            maxY = getYPos(maxVal)
        }
        
        tooltipPos.value = {
            x: xOffset + (svgX * scaleX),
            y: yOffset + (maxY * scaleY)
        }
    }
}

const handleMouseLeave = () => {
    hoveredIndex.value = -1
}
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
  transition: d 0.3s ease, opacity 0.3s ease, fill 0.3s ease, stroke 0.3s ease;
  filter: drop-shadow(0 2px 4px rgb(0 0 0 / 10%));
}

path:hover {
  filter: drop-shadow(0 4px 8px rgb(0 0 0 / 15%));
}

.glass-card {
  box-shadow: 0 8px 32px 0 rgb(31 38 135 / 15%);
}

/* Axis labels animation */
.x-axis text, .y-axis text {
  transition: all 0.2s ease;
}

/* Custom scrollbar for chart container if needed */
.chart-container {
    scrollbar-width: thin;
}
</style>
