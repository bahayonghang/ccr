<template>
  <div class="calendar-wrapper">
    <div class="grid grid-cols-7 gap-1.5 text-xs text-gray-500 dark:text-gray-400 mb-2">
      <div
        v-for="label in weekLabels"
        :key="label"
        class="text-center font-medium"
      >
        {{ label }}
      </div>
    </div>

    <div
      v-if="cells.length === 0"
      class="text-sm text-gray-400 dark:text-gray-500 text-center py-6"
    >
      暂无日历数据
    </div>
    <div
      v-else
      class="grid grid-cols-7 gap-1.5"
    >
      <div
        v-for="(cell, index) in cells"
        :key="cell ? cell.date : `empty-${index}`"
        class="calendar-cell"
        :class="cellClass(cell)"
        :title="cell ? buildTitle(cell) : ''"
      >
        <span
          v-if="cell"
          class="day-number"
        >{{ getDayNumber(cell.date) }}</span>
        <span
          v-if="cell && cell.income_increment"
          class="day-increment"
        >
          +{{ cell.income_increment.toFixed(1) }}
        </span>
      </div>
    </div>

    <div class="flex items-center gap-3 text-xs text-gray-500 dark:text-gray-400 mt-3">
      <div class="flex items-center gap-1">
        <span class="legend-dot checked" />
        已签到
      </div>
      <div class="flex items-center gap-1">
        <span class="legend-dot unchecked" />
        未签到
      </div>
      <div class="flex items-center gap-1">
        <span class="legend-dot today" />
        今天
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { CheckinDashboardCalendar, CheckinDashboardDay } from '@/types/checkin'

const props = defineProps<{
  calendar: CheckinDashboardCalendar | null
}>()

const weekLabels = ['日', '一', '二', '三', '四', '五', '六']

const todayString = computed(() => {
  const now = new Date()
  const year = now.getFullYear()
  const month = String(now.getMonth() + 1).padStart(2, '0')
  const day = String(now.getDate()).padStart(2, '0')
  return `${year}-${month}-${day}`
})

const firstWeekday = computed(() => {
  if (!props.calendar) return 0
  return new Date(props.calendar.year, props.calendar.month - 1, 1).getDay()
})

const cells = computed<(CheckinDashboardDay | null)[]>(() => {
  if (!props.calendar) return []
  const blanks: (CheckinDashboardDay | null)[] = Array.from(
    { length: firstWeekday.value },
    () => null
  )
  return blanks.concat(props.calendar.days)
})

const getDayNumber = (date: string) => Number(date.slice(8, 10))

const cellClass = (cell: CheckinDashboardDay | null) => {
  if (!cell) return 'cell-empty'
  const isToday = cell.date === todayString.value
  const classes = []
  if (cell.is_checked_in) {
    classes.push('cell-checked')
  } else {
    classes.push('cell-unchecked')
  }
  if (isToday) classes.push('cell-today')
  return classes.join(' ')
}

const buildTitle = (cell: CheckinDashboardDay) => {
  const status = cell.is_checked_in ? '已签到' : '未签到'
  const increment = cell.income_increment ? `+${cell.income_increment.toFixed(2)}` : '-'
  return `${cell.date} · ${status} · 增量 ${increment}`
}
</script>

<style scoped>
.calendar-cell {
  height: 2.25rem;
  border-radius: 0.375rem;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.1rem;
  border: 1px solid transparent;
  transition: all 0.15s ease;
}

.calendar-cell:hover {
  transform: scale(1.02);
}

.cell-empty {
  background: transparent;
}

.cell-checked {
  background: rgb(var(--color-success-rgb), 0.12);
  border-color: rgb(var(--color-success-rgb), 0.3);
}

.cell-unchecked {
  background: rgb(var(--color-gray-rgb), 0.08);
  border-color: rgb(var(--color-gray-rgb), 0.2);
}

.cell-today {
  border-color: var(--accent-info) !important;
  box-shadow: 0 0 0 2px rgb(var(--color-info-rgb), 0.3);
}

.day-number {
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--text-primary);
  line-height: 1;
}

.cell-checked .day-number {
  color: var(--accent-success);
}

.day-increment {
  font-size: 0.55rem;
  font-weight: 500;
  color: var(--accent-success);
  line-height: 1;
}

.legend-dot {
  width: 0.5rem;
  height: 0.5rem;
  border-radius: 50%;
}

.legend-dot.checked {
  background: var(--accent-success);
}

.legend-dot.unchecked {
  background: var(--border-color);
}

.legend-dot.today {
  border: 2px solid var(--accent-info);
}

:global(.dark) .day-number {
  color: var(--text-primary);
}

:global(.dark) .cell-checked .day-number {
  color: var(--accent-success);
}

:global(.dark) .cell-checked {
  background: rgb(var(--color-success-rgb), 0.2);
  border-color: rgb(var(--color-success-rgb), 0.4);
}

:global(.dark) .cell-unchecked {
  background: rgb(var(--color-slate-rgb), 0.4);
  border-color: rgb(var(--color-slate-rgb), 0.6);
}
</style>
