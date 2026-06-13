<template>
  <div class="trend-root">
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
          :x1="padding"
          :y1="baselineY"
          :x2="width - padding"
          :y2="baselineY"
          class="baseline"
        />

        <g
          v-for="(point, index) in chartData"
          :key="point.date"
          @mouseenter="hoveredIndex = index"
          @mouseleave="hoveredIndex = null"
        >
          <rect
            :x="barX(index)"
            :y="barY(point)"
            :width="barWidth"
            :height="barH(point)"
            :rx="Math.min(barWidth / 3, 3)"
            :class="barClass(point)"
          />
        </g>
      </svg>

      <div
        v-if="hoveredIndex !== null && chartData[hoveredIndex]"
        class="chart-tooltip"
        :style="tooltipStyle"
      >
        <div class="tooltip-date">
          {{ chartData[hoveredIndex].date }}
        </div>
        <div class="tooltip-row">
          <span>状态</span>
          <span
            class="tooltip-value"
            :class="chartData[hoveredIndex].is_checked_in ? 'tooltip-checked' : 'tooltip-missed'"
          >
            {{ chartData[hoveredIndex].is_checked_in ? '已签到' : '未签到' }}
          </span>
        </div>
        <div class="tooltip-row">
          <span>本日奖励</span>
          <span class="tooltip-value tooltip-checked">
            {{ formatReward(chartData[hoveredIndex].reward_amount) }}
          </span>
        </div>
        <div
          v-if="chartData[hoveredIndex].current_balance > 0"
          class="tooltip-row"
        >
          <span>当日余额</span>
          <span class="tooltip-value">${{ chartData[hoveredIndex].current_balance.toFixed(2) }}</span>
        </div>
      </div>

      <div class="chart-axis">
        <span>{{ trend?.start_date }}</span>
        <span
          v-for="tick in midTicks"
          :key="tick"
          class="chart-axis-mid"
        >
          {{ tick }}
        </span>
        <span>{{ trend?.end_date }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import type { CheckinDashboardTrend, CheckinDashboardTrendPoint } from '@/types/checkin'

const props = defineProps<{
  trend: CheckinDashboardTrend | null
}>()

const width = 800
const height = 220
const padding = 20
const chartHeight = height - padding * 2
const chartWidth = width - padding * 2
const baselineY = padding + chartHeight
const minBarRatio = 0.18

const hoveredIndex = ref<number | null>(null)

const chartData = computed<CheckinDashboardTrendPoint[]>(() => props.trend?.data_points ?? [])

const maxReward = computed(() => {
  const max = chartData.value.reduce((acc, p) => Math.max(acc, p.reward_amount ?? 0), 0)
  return max > 0 ? max : 1
})

const allZeroReward = computed(
  () => chartData.value.length === 0 || chartData.value.every((p) => !p.reward_amount)
)

const barGap = computed(() => {
  if (chartData.value.length === 0) return 2
  return chartData.value.length > 40 ? 2 : 4
})

const barWidth = computed(() => {
  if (chartData.value.length === 0) return 0
  const total = chartWidth
  const gap = barGap.value
  const raw = (total - gap * (chartData.value.length - 1)) / chartData.value.length
  return Math.max(raw, 2)
})

const barX = (index: number) => padding + index * (barWidth.value + barGap.value)

const barH = (point: CheckinDashboardTrendPoint) => {
  if (!point.is_checked_in) {
    return chartHeight * minBarRatio
  }
  if (allZeroReward.value) {
    return chartHeight * 0.45
  }
  const ratio = Math.max(point.reward_amount / maxReward.value, minBarRatio)
  return chartHeight * ratio
}

const barY = (point: CheckinDashboardTrendPoint) => baselineY - barH(point)

const barClass = (point: CheckinDashboardTrendPoint) => {
  if (!point.is_checked_in) return 'bar bar-missed'
  if (!point.reward_amount) return 'bar bar-checked-flat'
  return 'bar bar-checked'
}

const tooltipStyle = computed(() => {
  if (hoveredIndex.value === null) return {}
  const cx = barX(hoveredIndex.value) + barWidth.value / 2
  const leftPercent = (cx / width) * 100
  const left = leftPercent < 50 ? `${Math.max(leftPercent, 4)}%` : `${Math.min(leftPercent, 96)}%`
  const transform = leftPercent < 50 ? 'translate(0.6rem, -100%)' : 'translate(calc(-100% - 0.6rem), -100%)'
  return {
    left,
    top: `${padding + 6}px`,
    transform,
  }
})

const midTicks = computed<string[]>(() => {
  if (chartData.value.length < 30) return []
  const n = chartData.value.length
  const picks = [Math.floor(n * 0.25), Math.floor(n * 0.5), Math.floor(n * 0.75)]
  return picks.map((i) => chartData.value[i]?.date ?? '').filter(Boolean)
})

const formatReward = (amount: number) => {
  if (!amount || amount <= 0) return '—'
  return `+${amount.toFixed(2)}`
}
</script>

<style scoped>
.trend-root {
  width: 100%;
}

.trend-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 14rem;
  border-radius: 1rem;
  background: rgb(var(--color-bg-elevated-rgb) / 52%);
  border: 1px dashed rgb(var(--color-border-default-rgb) / 78%);
  color: var(--text-secondary);
  font-size: 0.88rem;
}

.trend-chart-container {
  position: relative;
  width: 100%;
  min-height: 13.75rem;
}

.trend-svg {
  width: 100%;
  height: 13.75rem;
  display: block;
}

.baseline {
  stroke: rgb(var(--color-border-default-rgb) / 72%);
  stroke-width: 1;
}

.bar {
  transition:
    fill 0.15s ease,
    stroke 0.15s ease,
    filter 0.15s ease;
}

.bar-checked {
  fill: var(--accent-primary);
  filter: drop-shadow(0 4px 8px rgb(var(--color-accent-primary-rgb) / 18%));
}

.bar-checked-flat {
  fill: rgb(var(--color-border-strong-rgb) / 42%);
}

.bar-missed {
  fill: transparent;
  stroke: rgb(var(--color-border-default-rgb) / 68%);
  stroke-width: 1;
  stroke-dasharray: 2 2;
}

.bar:hover {
  filter: brightness(1.08) drop-shadow(0 6px 14px rgb(var(--color-accent-primary-rgb) / 28%));
}

.chart-axis {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.55rem 0.5rem 0;
  color: var(--text-secondary);
  font-size: 0.72rem;
  font-weight: 600;
  font-family: var(--font-mono);
  gap: 0.5rem;
}

.chart-axis-mid {
  color: var(--text-muted);
}

.chart-tooltip {
  position: absolute;
  min-width: 11rem;
  padding: 0.75rem 0.85rem;
  border-radius: var(--radius-lg);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border-default);
  box-shadow: var(--shadow-lg);
  pointer-events: none;
  z-index: 5;
}

.tooltip-date {
  margin-bottom: 0.55rem;
  padding-bottom: 0.4rem;
  border-bottom: 1px solid rgb(var(--color-border-default-rgb) / 62%);
  color: var(--text-primary);
  font-size: 0.78rem;
  font-weight: 700;
  font-family: var(--font-mono);
}

.tooltip-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  margin-top: 0.3rem;
  color: var(--text-secondary);
  font-size: 0.75rem;
  font-weight: 600;
}

.tooltip-value {
  color: var(--text-primary);
  font-weight: 700;
  font-family: var(--font-mono);
}

.tooltip-value.tooltip-checked {
  color: var(--accent-primary);
}

.tooltip-value.tooltip-missed {
  color: var(--text-muted);
}

@media (width <= 768px) {
  .chart-axis {
    padding-inline: 0.25rem;
    font-size: 0.68rem;
  }

  .chart-axis-mid {
    display: none;
  }

  .chart-tooltip {
    min-width: 10rem;
  }
}
</style>
