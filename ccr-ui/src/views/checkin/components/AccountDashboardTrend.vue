<template>
  <div>
    <div
      v-if="chartData.length === 0"
      class="trend-empty"
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
        <line
          v-for="(_, index) in 5"
          :key="`grid-${index}`"
          :x1="padding"
          :y1="padding + (chartHeight / 4) * index"
          :x2="width - padding"
          :y2="padding + (chartHeight / 4) * index"
          class="grid-line"
        />

        <text
          v-for="(_, index) in 5"
          :key="`label-${index}`"
          :x="padding - 8"
          :y="padding + (chartHeight / 4) * index + 4"
          class="axis-label"
        >
          {{ formatAxisValue(maxValue - ((maxValue - minValue) / 4) * index) }}
        </text>

        <defs>
          <linearGradient
            id="checkin-trend-area-gradient"
            x1="0"
            y1="0"
            x2="0"
            y2="1"
          >
            <stop
              offset="0%"
              stop-color="rgba(var(--color-platform-gemini-rgb), 0.30)"
            />
            <stop
              offset="100%"
              stop-color="rgba(var(--color-platform-gemini-rgb), 0.04)"
            />
          </linearGradient>
        </defs>

        <path
          :d="areaPath"
          fill="url(#checkin-trend-area-gradient)"
        />

        <path
          :d="linePath"
          class="trend-line"
        />

        <circle
          v-for="(point, index) in chartData"
          :key="`point-${index}`"
          :cx="getX(index)"
          :cy="getY(point.total_quota)"
          r="4.5"
          class="data-point"
          @mouseenter="hoveredIndex = index"
          @mouseleave="hoveredIndex = null"
        />
      </svg>

      <div
        v-if="hoveredIndex !== null"
        class="chart-tooltip"
        :style="tooltipStyle"
      >
        <div class="tooltip-date">
          {{ chartData[hoveredIndex].date }}
        </div>
        <div class="tooltip-row">
          <span>总额度</span>
          <span class="tooltip-value">${{ chartData[hoveredIndex].total_quota.toFixed(2) }}</span>
        </div>
        <div class="tooltip-row">
          <span>当日余额</span>
          <span class="tooltip-value">${{ chartData[hoveredIndex].current_balance.toFixed(2) }}</span>
        </div>
        <div
          v-if="chartData[hoveredIndex].income_increment > 0"
          class="tooltip-row"
        >
          <span>增量</span>
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
  return Math.min(...chartData.value.map((point) => point.total_quota)) * 0.95
})

const maxValue = computed(() => {
  if (chartData.value.length === 0) return 1
  return Math.max(...chartData.value.map((point) => point.total_quota)) * 1.05
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
    .map((point, index) => `${index === 0 ? 'M' : 'L'} ${getX(index)} ${getY(point.total_quota)}`)
    .join(' ')
})

const areaPath = computed(() => {
  if (chartData.value.length === 0) return ''

  const start = `M ${getX(0)} ${padding + chartHeight}`
  const line = chartData.value
    .map((point, index) => `L ${getX(index)} ${getY(point.total_quota)}`)
    .join(' ')
  const end = `L ${getX(chartData.value.length - 1)} ${padding + chartHeight} Z`

  return `${start} ${line} ${end}`
})

const tooltipStyle = computed(() => {
  if (hoveredIndex.value === null) return {}

  const x = getX(hoveredIndex.value)
  const left = x < width / 2 ? `${(x / width) * 100 + 2}%` : `${(x / width) * 100 - 22}%`

  return {
    left,
    top: '28px',
  }
})
</script>

<style scoped>
.trend-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 14rem;
  border-radius: 1.1rem;
  background: rgb(var(--color-bg-elevated-rgb) / 52%);
  border: 1px dashed rgb(var(--color-border-default-rgb) / 78%);
  color: var(--text-secondary);
  font-size: 0.92rem;
}

.trend-chart-container {
  position: relative;
  width: 100%;
  min-height: 13.75rem;
}

.trend-svg {
  width: 100%;
  height: 13.75rem;
}

.grid-line {
  stroke: rgb(var(--color-border-default-rgb) / 88%);
  stroke-width: 1;
}

.axis-label {
  fill: var(--text-secondary);
  font-size: 10px;
  font-weight: 700;
  text-anchor: end;
}

.trend-line {
  fill: none;
  stroke: var(--platform-gemini);
  stroke-width: 3;
  stroke-linecap: round;
  stroke-linejoin: round;
  filter: drop-shadow(0 8px 16px rgb(var(--color-platform-gemini-rgb) / 22%));
}

.data-point {
  fill: var(--platform-gemini);
  stroke: white;
  stroke-width: 2.5;
  cursor: pointer;
  transform-box: fill-box;
  transform-origin: center;
  transition:
    transform 0.18s ease,
    filter 0.18s ease;
}

.data-point:hover {
  transform: scale(1.3);
  filter: drop-shadow(0 6px 14px rgb(var(--color-platform-gemini-rgb) / 30%));
}

.chart-axis {
  display: flex;
  justify-content: space-between;
  padding: 0.6rem 45px 0;
  color: var(--text-secondary);
  font-size: 0.76rem;
  font-weight: 600;
}

.chart-tooltip {
  position: absolute;
  min-width: 11rem;
  padding: 0.9rem 0.95rem;
  border-radius: 0.95rem;
  background: rgb(var(--color-bg-base-rgb) / 96%);
  border: 1px solid rgb(var(--color-border-default-rgb) / 90%);
  box-shadow: 0 20px 40px rgb(45 27 48 / 18%);
  backdrop-filter: blur(18px) saturate(145%);
  pointer-events: none;
  z-index: 5;
}

.tooltip-date {
  margin-bottom: 0.65rem;
  padding-bottom: 0.45rem;
  border-bottom: 1px solid rgb(var(--color-border-default-rgb) / 72%);
  color: var(--text-primary);
  font-size: 0.82rem;
  font-weight: 700;
}

.tooltip-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  margin-top: 0.35rem;
  color: var(--text-secondary);
  font-size: 0.78rem;
  font-weight: 600;
}

.tooltip-value {
  color: var(--platform-gemini);
  font-weight: 700;
}

.tooltip-increment {
  color: var(--accent-success);
  font-weight: 700;
}

:global(.dark) .chart-tooltip {
  background: rgb(var(--color-bg-surface-rgb) / 96%);
  border-color: rgb(var(--color-border-default-rgb) / 90%);
  box-shadow: 0 24px 48px rgb(0 0 0 / 34%);
}

@media (width <= 768px) {
  .chart-axis {
    padding-inline: 0.25rem;
  }

  .chart-tooltip {
    min-width: 10rem;
  }
}
</style>
