<template>
  <div
    v-if="loading"
    class="loading-container"
  >
    <div class="loading-spinner">
      <div class="spinner-ring" />
      <div class="spinner-ring" />
      <div class="spinner-ring" />
    </div>
    <span class="loading-text">{{ loadingLabel }}</span>
  </div>

  <div
    v-else
    class="heatmap-container"
  >
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

    <div class="grid-wrapper">
      <div class="day-labels">
        <span class="day-label" />
        <span class="day-label">{{ $t('activityHeatmap.mon', 'Mon') }}</span>
        <span class="day-label" />
        <span class="day-label">{{ $t('activityHeatmap.wed', 'Wed') }}</span>
        <span class="day-label" />
        <span class="day-label">{{ $t('activityHeatmap.fri', 'Fri') }}</span>
        <span class="day-label" />
      </div>

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
              @mouseenter="emit('hover-day', { day, event: $event })"
              @mouseleave="emit('leave-day')"
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
</template>

<script setup lang="ts">
import type { ActivityHeatmapDayData, ActivityHeatmapMonthLabel } from '@/types/activityHeatmap'

defineProps<{
  loading: boolean
  loadingLabel: string
  monthLabels: ActivityHeatmapMonthLabel[]
  weeks: Array<Array<ActivityHeatmapDayData | null>>
}>()

const emit = defineEmits<{
  (e: 'hover-day', payload: { day: ActivityHeatmapDayData; event: MouseEvent }): void
  (e: 'leave-day'): void
}>()
</script>

<style scoped>
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

.loading-text {
  font-size: var(--text-sm);
  color: var(--color-text-muted);
}

.heatmap-container {
  background: var(--color-bg-surface);
  border-radius: var(--radius-lg);
  padding: var(--space-4);
  border: 1px solid var(--color-border-subtle);
}

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

.grid-wrapper {
  display: flex;
  gap: var(--space-1);
  justify-content: center;
}

.day-labels {
  display: flex;
  flex-direction: column;
  gap: 3px;
  width: 24px;
  flex-shrink: 0;
}

.day-label {
  height: 10px;
  font-size: 9px;
  font-weight: var(--font-medium);
  color: var(--color-text-muted);
  display: flex;
  align-items: center;
}

.weeks-grid {
  display: flex;
  gap: 3px;
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
  gap: 3px;
}

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
  width: 10px;
  height: 10px;
  border-radius: 2px;
  cursor: pointer;
  position: relative;
  transition: all var(--duration-fast) var(--ease-out);
}

.day-cell:hover {
  transform: scale(1.3);
  z-index: 10;
}

.day-cell[data-level='0'] { background: var(--color-bg-overlay); }
.day-cell[data-level='1'] { background: rgb(6 182 212 / 30%); }
.day-cell[data-level='2'] { background: rgb(6 182 212 / 50%); }
.day-cell[data-level='3'] { background: rgb(6 182 212 / 75%); }
.day-cell[data-level='4'] { background: var(--color-accent-primary); }

.day-cell[data-level='3']:hover,
.day-cell[data-level='4']:hover {
  box-shadow: 0 0 12px var(--color-accent-primary);
}

.day-cell.is-today {
  outline: 2px solid var(--color-accent-secondary);
  outline-offset: 1px;
}

.day-cell-empty {
  width: 10px;
  height: 10px;
}

.cell-glow {
  position: absolute;
  inset: -2px;
  background: var(--color-accent-primary);
  border-radius: 2px;
  opacity: 0.3;
  filter: blur(4px);
  animation: glow-pulse 2s ease-in-out infinite;
}

@keyframes glow-pulse {
  0%, 100% { opacity: 0.2; }
  50% { opacity: 0.4; }
}
</style>
