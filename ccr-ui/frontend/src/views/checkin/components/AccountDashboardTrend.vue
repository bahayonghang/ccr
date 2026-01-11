<template>
  <div>
    <div
      v-if="chartData.length === 0"
      class="text-sm text-gray-400 dark:text-gray-500 text-center py-16"
    >
      暂无趋势数据
    </div>
    <div
      v-else
      class="trend-chart-container"
    >
      <svg
        :viewBox="`0 0 ${width} ${height}`"
        class="trend-svg"
        preserveAspectRatio="xMidYMid meet"
      >
        <!-- 网格线 -->
        <line
          v-for="(_, i) in 5"
          :key="`grid-${i}`"
          :x1="padding"
          :y1="padding + (chartHeight / 4) * i"
          :x2="width - padding"
          :y2="padding + (chartHeight / 4) * i"
          class="grid-line"
        />
        <!-- Y轴刻度 -->
        <text
          v-for="(_, i) in 5"
          :key="`label-${i}`"
          :x="padding - 8"
          :y="padding + (chartHeight / 4) * i + 4"
          class="axis-label"
        >
          {{ formatAxisValue(maxValue - (maxValue - minValue) / 4 * i) }}
        </text>
        <!-- 面积填充 -->
        <defs>
          <linearGradient
            id="areaGradient"
            x1="0"
            y1="0"
            x2="0"
            y2="1"
          >
            <stop
              offset="0%"
              stop-color="rgba(var(--color-info-rgb), 0.25)"
            />
            <stop
              offset="100%"
              stop-color="rgba(var(--color-info-rgb), 0.02)"
            />
          </linearGradient>
        </defs>
        <path
          :d="areaPath"
          fill="url(#areaGradient)"
        />
        <!-- 折线 -->
        <path
          :d="linePath"
          class="trend-line"
        />
        <!-- 数据点 -->
        <circle
          v-for="(point, index) in chartData"
          :key="`point-${index}`"
          :cx="getX(index)"
          :cy="getY(point.total_quota)"
          r="5"
          class="data-point"
          @mouseenter="hoveredIndex = index"
          @mouseleave="hoveredIndex = null"
        />
      </svg>
      <!-- Tooltip -->
      <div
        v-if="hoveredIndex !== null"
        class="chart-tooltip"
        :style="tooltipStyle"
      >
        <div class="tooltip-date">
          {{ chartData[hoveredIndex].date }}
        </div>
        <div class="tooltip-row">
          <span>总额度:</span>
          <span class="tooltip-value">${{ chartData[hoveredIndex].total_quota.toFixed(2) }}</span>
        </div>
        <div class="tooltip-row">
          <span>当日余额:</span>
          <span class="tooltip-value">${{ chartData[hoveredIndex].current_balance.toFixed(2) }}</span>
        </div>
        <div
          v-if="chartData[hoveredIndex].income_increment > 0"
          class="tooltip-row"
        >
          <span>增量:</span>
          <span class="tooltip-increment">+${{ chartData[hoveredIndex].income_increment.toFixed(2) }}</span>
        </div>
      </div>
      <div class="chart-axis">
        <span>{{ trend?.start_date }}</span>
        <span>{{ trend?.end_date }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import type { CheckinDashboardTrend } from '@/types/checkin'

const props = defineProps<{
  trend: CheckinDashboardTrend | null
}>()

const width = 800
const height = 220
const padding = 45
const chartWidth = width - padding * 2
const chartHeight = height - padding * 2

const hoveredIndex = ref<number | null>(null)

const chartData = computed(() => props.trend?.data_points ?? [])

const minValue = computed(() => {
  if (chartData.value.length === 0) return 0
  return Math.min(...chartData.value.map(p => p.total_quota)) * 0.95
})

const maxValue = computed(() => {
  if (chartData.value.length === 0) return 1
  return Math.max(...chartData.value.map(p => p.total_quota)) * 1.05
})

const getX = (index: number) => {
  if (chartData.value.length <= 1) return padding + chartWidth / 2
  return padding + (index / (chartData.value.length - 1)) * chartWidth
}

const getY = (value: number) => {
  const range = maxValue.value - minValue.value
  if (range === 0) return padding + chartHeight / 2
  return padding + chartHeight - ((value - minValue.value) / range) * chartHeight
}

const formatAxisValue = (value: number) => {
  if (value >= 1000) return `$${(value / 1000).toFixed(1)}k`
  return `$${value.toFixed(0)}`
}

const linePath = computed(() => {
  if (chartData.value.length === 0) return ''
  return chartData.value
    .map((point, i) => `${i === 0 ? 'M' : 'L'} ${getX(i)} ${getY(point.total_quota)}`)
    .join(' ')
})

const areaPath = computed(() => {
  if (chartData.value.length === 0) return ''
  const start = `M ${getX(0)} ${padding + chartHeight}`
  const line = chartData.value.map((point, i) => `L ${getX(i)} ${getY(point.total_quota)}`).join(' ')
  const end = `L ${getX(chartData.value.length - 1)} ${padding + chartHeight} Z`
  return `${start} ${line} ${end}`
})

const tooltipStyle = computed(() => {
  if (hoveredIndex.value === null) return {}
  const x = getX(hoveredIndex.value)
  const left = x < width / 2 ? `${(x / width) * 100 + 2}%` : `${(x / width) * 100 - 22}%`
  return { left, top: '30px' }
})
</script>

<style scoped>
.trend-chart-container {
  position: relative;
  width: 100%;
  min-height: 220px;
}

.trend-svg {
  width: 100%;
  height: 220px;
}

.grid-line {
  stroke: rgba(var(--color-gray-rgb), 0.15);
  stroke-width: 1;
}

.axis-label {
  font-size: 10px;
  fill: var(--text-muted);
  text-anchor: end;
}

.trend-line {
  fill: none;
  stroke: var(--accent-info);
  stroke-width: 3;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.data-point {
  fill: var(--accent-info);
  stroke: white;
  stroke-width: 2;
  cursor: pointer;
  transition: r 0.15s ease;
}

.data-point:hover {
  r: 7;
}

.chart-axis {
  display: flex;
  justify-content: space-between;
  font-size: 0.75rem;
  color: var(--text-muted);
  padding: 0.5rem 45px 0;
}

.chart-tooltip {
  position: absolute;
  background: var(--glass-bg-heavy);
  border: 1px solid var(--border-color);
  border-radius: 0.6rem;
  padding: 0.85rem;
  box-shadow: var(--shadow-medium);
  font-size: 0.8rem;
  min-width: 160px;
  pointer-events: none;
  z-index: 10;
}

.tooltip-date {
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 0.6rem;
  padding-bottom: 0.4rem;
  border-bottom: 1px solid var(--border-color);
}

.tooltip-row {
  display: flex;
  justify-content: space-between;
  margin-top: 0.3rem;
  color: var(--text-secondary);
}

.tooltip-value {
  font-weight: 600;
  color: var(--accent-info);
}

.tooltip-increment {
  font-weight: 600;
  color: var(--accent-success);
}

:global(.dark) .chart-tooltip {
  background: rgba(var(--color-slate-dark-rgb), 0.98);
  border-color: rgba(var(--color-slate-rgb), 0.8);
}

:global(.dark) .tooltip-date {
  color: var(--text-primary);
}

:global(.dark) .tooltip-row {
  color: var(--text-muted);
}

:global(.dark) .grid-line {
  stroke: rgba(var(--color-slate-rgb), 0.4);
}

:global(.dark) .axis-label {
  fill: var(--text-secondary);
}
</style>
