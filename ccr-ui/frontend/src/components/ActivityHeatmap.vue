<template>
  <div class="activity-heatmap">
    <!-- Header -->
    <div class="heatmap-header">
      <div class="header-left">
        <div class="icon-wrapper">
          <svg
            class="icon"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
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
        </div>
        <h3 class="title">
          {{ $t('activityHeatmap.title', '活动热力图') }}
        </h3>
      </div>

      <!-- Legend -->
      <div class="legend">
        <span class="legend-label">{{ $t('activityHeatmap.less', '较少') }}</span>
        <div class="legend-cells">
          <div
            v-for="level in 5"
            :key="level"
            class="legend-cell"
            :data-level="level - 1"
          />
        </div>
        <span class="legend-label">{{ $t('activityHeatmap.more', '较多') }}</span>
      </div>
    </div>

    <!-- Loading State -->
    <div
      v-if="loading"
      class="loading-container"
    >
      <div class="loading-spinner">
        <div class="spinner-ring" />
        <div class="spinner-ring" />
        <div class="spinner-ring" />
      </div>
      <span class="loading-text">{{ $t('common.loading', '加载中...') }}</span>
    </div>

    <!-- Heatmap Grid -->
    <div
      v-else
      class="heatmap-container"
    >
      <!-- Month Labels -->
      <div class="month-row">
        <div class="day-label-spacer" />
        <div class="months-container">
          <span
            v-for="(month, idx) in monthLabels"
            :key="idx"
            class="month-label"
            :style="{ left: `${month.weekOffset * 12}px` }"
          >
            {{ month.name }}
          </span>
        </div>
      </div>

      <!-- Grid with Day Labels -->
      <div class="grid-wrapper">
        <!-- Day Labels -->
        <div class="day-labels">
          <span class="day-label" />
          <span class="day-label">{{ $t('activityHeatmap.mon', 'Mon') }}</span>
          <span class="day-label" />
          <span class="day-label">{{ $t('activityHeatmap.wed', 'Wed') }}</span>
          <span class="day-label" />
          <span class="day-label">{{ $t('activityHeatmap.fri', 'Fri') }}</span>
          <span class="day-label" />
        </div>

        <!-- Weeks Grid -->
        <div class="weeks-grid">
          <div
            v-for="(week, weekIndex) in weeks"
            :key="weekIndex"
            class="week-column"
            :style="{ '--week-index': weekIndex }"
          >
            <div
              v-for="(day, dayIndex) in week"
              :key="dayIndex"
              class="day-cell-wrapper"
              :style="{ '--day-index': dayIndex, '--cell-delay': weekIndex * 7 + dayIndex }"
            >
              <div
                v-if="day"
                class="day-cell"
                :data-level="day.level"
                :class="{ 'is-today': day.isToday }"
                @mouseenter="showTooltip(day, $event)"
                @mouseleave="hideTooltip"
              >
                <div
                  v-if="day.level >= 3"
                  class="cell-glow"
                />
              </div>
              <div
                v-else
                class="day-cell-empty"
              />
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Tooltip -->
    <Teleport to="body">
      <Transition name="tooltip">
        <div
          v-if="tooltip.visible"
          class="heatmap-tooltip"
          :style="{
            left: `${tooltip.x}px`,
            top: `${tooltip.y}px`
          }"
        >
          <div class="tooltip-date">
            {{ tooltip.date }}
          </div>
          <div class="tooltip-value">
            <span class="tooltip-count">{{ formatNumber(tooltip.count) }}</span>
            <span class="tooltip-unit">tokens</span>
          </div>
          <div class="tooltip-arrow" />
        </div>
      </Transition>
    </Teleport>

    <!-- Summary Stats -->
    <div class="stats-row">
      <div class="stat-card stat-card-primary">
        <div class="stat-icon">
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"
            />
          </svg>
        </div>
        <div class="stat-content">
          <span class="stat-label">{{ $t('activityHeatmap.activeDays', '活跃天数') }}</span>
          <span class="stat-value">{{ heatmapData?.active_days ?? 0 }}</span>
        </div>
      </div>

      <div class="stat-card stat-card-secondary">
        <div class="stat-icon">
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M13 7h8m0 0v8m0-8l-8 8-4-4-6 6"
            />
          </svg>
        </div>
        <div class="stat-content">
          <span class="stat-label">{{ $t('activityHeatmap.totalTokens', '总 Token 数') }}</span>
          <span class="stat-value">{{ formatNumber(heatmapData?.total_tokens ?? 0) }}</span>
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
  isToday: boolean
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
  const todayKey = today.toISOString().split('T')[0]
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

      // Calculate level (0-4)
      let level = 0
      if (count > 0) {
        const ratio = count / maxValue
        level = ratio > 0.75 ? 4 : ratio > 0.5 ? 3 : ratio > 0.25 ? 2 : 1
      }

      weekDays.push({
        date: currentDate.toLocaleDateString('zh-CN', {
          year: 'numeric',
          month: 'short',
          day: 'numeric',
          weekday: 'short'
        }),
        dateKey,
        count,
        level,
        isToday: dateKey === todayKey
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
    const weekStart = new Date(start)
    weekStart.setDate(weekStart.getDate() + (week * 7))

    const month = weekStart.getMonth()

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

// Tooltip handlers
const showTooltip = (day: DayData, event: MouseEvent) => {
  const rect = (event.target as HTMLElement).getBoundingClientRect()
  tooltip.value = {
    visible: true,
    date: day.date,
    count: day.count,
    x: rect.left + rect.width / 2,
    y: rect.top - 8
  }
}

const hideTooltip = () => {
  tooltip.value.visible = false
}

// Format number
const formatNumber = (num: number): string => {
  if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`
  if (num >= 1000) return `${(num / 1000).toFixed(1)}K`
  return num.toLocaleString()
}
</script>

<style scoped>
.activity-heatmap {
  --cell-size: 10px;
  --cell-gap: 3px;
  --cell-radius: 2px;

  /* Heat colors - warm gradient */
  --heat-0: var(--color-bg-overlay);
  --heat-1: rgba(6, 182, 212, 0.3);
  --heat-2: rgba(6, 182, 212, 0.5);
  --heat-3: rgba(6, 182, 212, 0.75);
  --heat-4: var(--color-accent-primary);

  background: var(--glass-bg-light);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-xl);
  padding: var(--space-5);
  backdrop-filter: var(--glass-blur-sm);
  box-shadow: var(--glass-shadow);
}

/* Header */
.heatmap-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--space-4);
}

.header-left {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.icon-wrapper {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, var(--color-accent-primary), var(--color-accent-secondary));
  border-radius: var(--radius-lg);
  box-shadow: var(--glow-primary);
}

.icon {
  width: 18px;
  height: 18px;
  color: white;
}

.title {
  font-size: var(--text-base);
  font-weight: var(--font-semibold);
  color: var(--color-text-primary);
  margin: 0;
}

/* Legend */
.legend {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-1-5) var(--space-3);
  background: var(--color-bg-surface);
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--radius-full);
}

.legend-label {
  font-size: var(--text-xs);
  color: var(--color-text-muted);
  font-weight: var(--font-medium);
}

.legend-cells {
  display: flex;
  gap: 3px;
}

.legend-cell {
  width: 10px;
  height: 10px;
  border-radius: var(--cell-radius);
  transition: transform var(--duration-fast) var(--ease-out);
}

.legend-cell:hover {
  transform: scale(1.2);
}

.legend-cell[data-level="0"] { background: var(--heat-0); }
.legend-cell[data-level="1"] { background: var(--heat-1); }
.legend-cell[data-level="2"] { background: var(--heat-2); }
.legend-cell[data-level="3"] { background: var(--heat-3); }
.legend-cell[data-level="4"] { background: var(--heat-4); }

/* Loading */
.loading-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--space-12) 0;
  gap: var(--space-3);
}

.loading-spinner {
  position: relative;
  width: 40px;
  height: 40px;
}

.spinner-ring {
  position: absolute;
  inset: 0;
  border: 2px solid transparent;
  border-top-color: var(--color-accent-primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

.spinner-ring:nth-child(2) {
  inset: 4px;
  border-top-color: var(--color-accent-secondary);
  animation-duration: 0.8s;
  animation-direction: reverse;
}

.spinner-ring:nth-child(3) {
  inset: 8px;
  border-top-color: var(--color-accent-primary);
  animation-duration: 0.6s;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.loading-text {
  font-size: var(--text-sm);
  color: var(--color-text-muted);
}

/* Heatmap Container */
.heatmap-container {
  background: var(--color-bg-surface);
  border-radius: var(--radius-lg);
  padding: var(--space-4);
  border: 1px solid var(--color-border-subtle);
}

/* Month Row */
.month-row {
  display: flex;
  margin-bottom: var(--space-1);
}

.day-label-spacer {
  width: 28px;
  flex-shrink: 0;
}

.months-container {
  position: relative;
  height: 16px;
  flex: 1;
}

.month-label {
  position: absolute;
  font-size: 10px;
  font-weight: var(--font-medium);
  color: var(--color-text-muted);
  white-space: nowrap;
}

/* Grid Wrapper */
.grid-wrapper {
  display: flex;
  gap: var(--space-1);
  justify-content: center;
}

/* Day Labels */
.day-labels {
  display: flex;
  flex-direction: column;
  gap: var(--cell-gap);
  width: 24px;
  flex-shrink: 0;
}

.day-label {
  height: var(--cell-size);
  font-size: 9px;
  font-weight: var(--font-medium);
  color: var(--color-text-muted);
  display: flex;
  align-items: center;
}

/* Weeks Grid */
.weeks-grid {
  display: flex;
  gap: var(--cell-gap);
  overflow-x: auto;
  padding-bottom: var(--space-1);
  justify-content: center;
}

.weeks-grid::-webkit-scrollbar {
  height: 4px;
}

.weeks-grid::-webkit-scrollbar-track {
  background: var(--color-bg-overlay);
  border-radius: 2px;
}

.weeks-grid::-webkit-scrollbar-thumb {
  background: var(--color-border-strong);
  border-radius: 2px;
}

.week-column {
  display: flex;
  flex-direction: column;
  gap: var(--cell-gap);
}

/* Day Cells */
.day-cell-wrapper {
  animation: cell-appear 0.4s var(--ease-out-back) backwards;
  animation-delay: calc(var(--cell-delay) * 3ms);
}

@keyframes cell-appear {
  from {
    opacity: 0;
    transform: scale(0);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.day-cell {
  width: var(--cell-size);
  height: var(--cell-size);
  border-radius: var(--cell-radius);
  cursor: pointer;
  position: relative;
  transition: all var(--duration-fast) var(--ease-out);
}

.day-cell:hover {
  transform: scale(1.3);
  z-index: 10;
}

.day-cell[data-level="0"] { background: var(--heat-0); }
.day-cell[data-level="1"] { background: var(--heat-1); }
.day-cell[data-level="2"] { background: var(--heat-2); }
.day-cell[data-level="3"] { background: var(--heat-3); }
.day-cell[data-level="4"] { background: var(--heat-4); }

.day-cell[data-level="3"]:hover,
.day-cell[data-level="4"]:hover {
  box-shadow: 0 0 12px var(--color-accent-primary);
}

.day-cell.is-today {
  outline: 2px solid var(--color-accent-secondary);
  outline-offset: 1px;
}

.day-cell-empty {
  width: var(--cell-size);
  height: var(--cell-size);
}

/* Cell Glow Effect */
.cell-glow {
  position: absolute;
  inset: -2px;
  background: var(--color-accent-primary);
  border-radius: var(--cell-radius);
  opacity: 0.3;
  filter: blur(4px);
  animation: glow-pulse 2s ease-in-out infinite;
}

@keyframes glow-pulse {
  0%, 100% { opacity: 0.2; }
  50% { opacity: 0.4; }
}

/* Tooltip */
.heatmap-tooltip {
  position: fixed;
  z-index: var(--z-tooltip);
  padding: var(--space-2) var(--space-3);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-xl);
  backdrop-filter: var(--glass-blur-md);
  transform: translate(-50%, -100%);
  pointer-events: none;
}

.tooltip-date {
  font-size: var(--text-xs);
  font-weight: var(--font-semibold);
  color: var(--color-text-primary);
  margin-bottom: var(--space-1);
}

.tooltip-value {
  display: flex;
  align-items: baseline;
  gap: var(--space-1);
}

.tooltip-count {
  font-size: var(--text-sm);
  font-weight: var(--font-bold);
  color: var(--color-accent-primary);
  font-family: var(--font-mono);
}

.tooltip-unit {
  font-size: var(--text-xs);
  color: var(--color-text-muted);
}

.tooltip-arrow {
  position: absolute;
  bottom: -6px;
  left: 50%;
  transform: translateX(-50%);
  width: 0;
  height: 0;
  border-left: 6px solid transparent;
  border-right: 6px solid transparent;
  border-top: 6px solid var(--color-bg-elevated);
}

/* Tooltip Transition */
.tooltip-enter-active,
.tooltip-leave-active {
  transition: all var(--duration-fast) var(--ease-out);
}

.tooltip-enter-from,
.tooltip-leave-to {
  opacity: 0;
  transform: translate(-50%, -90%);
}

/* Stats Row */
.stats-row {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: var(--space-3);
  margin-top: var(--space-4);
  padding-top: var(--space-4);
  border-top: 1px solid var(--color-border-subtle);
}

.stat-card {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-4);
  border-radius: var(--radius-lg);
  border: 1px solid var(--color-border-subtle);
  transition: all var(--duration-normal) var(--ease-out);
}

.stat-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
}

.stat-card-primary {
  background: linear-gradient(135deg, rgba(6, 182, 212, 0.1), rgba(6, 182, 212, 0.05));
  border-color: rgba(6, 182, 212, 0.2);
}

.stat-card-primary:hover {
  border-color: rgba(6, 182, 212, 0.4);
  box-shadow: var(--glow-primary);
}

.stat-card-secondary {
  background: linear-gradient(135deg, rgba(139, 92, 246, 0.1), rgba(139, 92, 246, 0.05));
  border-color: rgba(139, 92, 246, 0.2);
}

.stat-card-secondary:hover {
  border-color: rgba(139, 92, 246, 0.4);
  box-shadow: var(--glow-secondary);
}

.stat-icon {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-md);
}

.stat-card-primary .stat-icon {
  background: rgba(6, 182, 212, 0.15);
  color: var(--color-accent-primary);
}

.stat-card-secondary .stat-icon {
  background: rgba(139, 92, 246, 0.15);
  color: var(--color-accent-secondary);
}

.stat-icon svg {
  width: 18px;
  height: 18px;
}

.stat-content {
  display: flex;
  flex-direction: column;
  gap: var(--space-0-5);
}

.stat-label {
  font-size: var(--text-xs);
  font-weight: var(--font-medium);
  color: var(--color-text-muted);
}

.stat-value {
  font-size: var(--text-xl);
  font-weight: var(--font-bold);
  color: var(--color-text-primary);
  font-family: var(--font-mono);
}

/* Responsive */
@media (max-width: 640px) {
  .heatmap-header {
    flex-direction: column;
    align-items: flex-start;
    gap: var(--space-3);
  }

  .stats-row {
    grid-template-columns: 1fr;
  }
}
</style>
