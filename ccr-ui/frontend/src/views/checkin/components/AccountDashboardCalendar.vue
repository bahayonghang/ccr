<template>
  <div>
    <div class="grid grid-cols-7 gap-2 text-xs text-gray-500 dark:text-gray-400 mb-3">
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
      class="text-sm text-gray-400 dark:text-gray-500 text-center py-8"
    >
      暂无日历数据
    </div>
    <div
      v-else
      class="grid grid-cols-7 gap-2"
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

    <div class="flex items-center gap-4 text-xs text-gray-500 dark:text-gray-400 mt-4">
      <div class="flex items-center gap-1.5">
        <span class="legend-dot checked" />
        已签到
      </div>
      <div class="flex items-center gap-1.5">
        <span class="legend-dot unchecked" />
        未签到
      </div>
      <div class="flex items-center gap-1.5">
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
  aspect-ratio: 1;
  min-height: 2.5rem;
  border-radius: 0.5rem;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.15rem;
  border: 1px solid transparent;
  transition: all 0.2s ease;
}

.calendar-cell:hover {
  transform: scale(1.05);
}

.cell-empty {
  background: transparent;
}

.cell-checked {
  background: rgba(16, 185, 129, 0.12);
  border-color: rgba(16, 185, 129, 0.3);
}

.cell-unchecked {
  background: rgba(148, 163, 184, 0.08);
  border-color: rgba(148, 163, 184, 0.2);
}

.cell-today {
  border-color: #3b82f6 !important;
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.3);
}

.day-number {
  font-size: 1rem;
  font-weight: 600;
  color: #334155;
}

.cell-checked .day-number {
  color: #059669;
}

.day-increment {
  font-size: 0.6rem;
  font-weight: 500;
  color: #10b981;
}

.legend-dot {
  width: 0.5rem;
  height: 0.5rem;
  border-radius: 50%;
}

.legend-dot.checked {
  background: #10b981;
}

.legend-dot.unchecked {
  background: #cbd5e1;
}

.legend-dot.today {
  border: 2px solid #3b82f6;
}

:global(.dark) .day-number {
  color: #e2e8f0;
}

:global(.dark) .cell-checked .day-number {
  color: #34d399;
}

:global(.dark) .cell-checked {
  background: rgba(16, 185, 129, 0.2);
  border-color: rgba(16, 185, 129, 0.4);
}

:global(.dark) .cell-unchecked {
  background: rgba(51, 65, 85, 0.4);
  border-color: rgba(51, 65, 85, 0.6);
}
</style>
