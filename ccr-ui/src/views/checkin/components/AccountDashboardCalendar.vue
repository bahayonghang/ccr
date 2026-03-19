<template>
  <div class="calendar-wrapper">
    <div class="calendar-weekdays">
      <div
        v-for="label in weekLabels"
        :key="label"
        class="weekday-label"
      >
        {{ label }}
      </div>
    </div>

    <div
      v-if="cells.length === 0"
      class="calendar-empty"
    >
      暂无日历数据
    </div>

    <div
      v-else
      class="calendar-grid"
    >
      <div
        v-for="(cell, index) in cells"
        :key="cell ? cell.date : `empty-${index}`"
        class="calendar-cell"
        :class="cellClass(cell)"
        :title="cell ? buildTitle(cell) : ''"
      >
        <template v-if="cell">
          <span class="day-number">{{ getDayNumber(cell.date) }}</span>
          <span
            v-if="cell.income_increment"
            class="day-increment"
          >
            +{{ cell.income_increment.toFixed(1) }}
          </span>
        </template>
      </div>
    </div>

    <div class="calendar-legend">
      <div class="legend-item">
        <span class="legend-dot checked" />
        已签到
      </div>
      <div class="legend-item">
        <span class="legend-dot unchecked" />
        未签到
      </div>
      <div class="legend-item">
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

  const classes = [cell.is_checked_in ? 'cell-checked' : 'cell-unchecked']

  if (cell.date === todayString.value) {
    classes.push('cell-today')
  }

  return classes.join(' ')
}

const buildTitle = (cell: CheckinDashboardDay) => {
  const status = cell.is_checked_in ? '已签到' : '未签到'
  const increment = cell.income_increment ? `+${cell.income_increment.toFixed(2)}` : '-'
  return `${cell.date} · ${status} · 增量 ${increment}`
}
</script>

<style scoped>
.calendar-wrapper {
  display: flex;
  flex-direction: column;
  gap: 0.9rem;
}

.calendar-weekdays,
.calendar-grid {
  display: grid;
  grid-template-columns: repeat(7, minmax(0, 1fr));
}

.calendar-weekdays {
  gap: 0.45rem;
}

.weekday-label {
  color: var(--text-muted);
  font-size: 0.72rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-align: center;
}

.calendar-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 11rem;
  border-radius: 1rem;
  background: rgb(var(--color-bg-elevated-rgb) / 56%);
  border: 1px dashed rgb(var(--color-border-default-rgb) / 78%);
  color: var(--text-secondary);
  font-size: 0.92rem;
}

.calendar-grid {
  gap: 0.55rem;
}

.calendar-cell {
  min-height: 4.2rem;
  border-radius: 1rem;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.28rem;
  border: 1px solid transparent;
  transition:
    transform 0.18s ease,
    border-color 0.18s ease,
    background-color 0.18s ease,
    box-shadow 0.18s ease;
}

.calendar-cell:hover:not(.cell-empty) {
  transform: translateY(-1px);
}

.cell-empty {
  background: transparent;
  border-color: transparent;
}

.cell-checked {
  background:
    linear-gradient(180deg, rgb(var(--color-success-rgb) / 16%), rgb(var(--color-success-rgb) / 10%)),
    rgb(var(--color-bg-elevated-rgb) / 50%);
  border-color: rgb(var(--color-success-rgb) / 38%);
  box-shadow:
    inset 0 1px 0 rgb(255 255 255 / 30%),
    0 10px 18px rgb(var(--color-success-rgb) / 8%);
}

.cell-unchecked {
  background: rgb(var(--color-bg-elevated-rgb) / 48%);
  border-color: rgb(var(--color-border-default-rgb) / 72%);
  box-shadow: inset 0 1px 0 rgb(255 255 255 / 22%);
}

.cell-today {
  border-color: rgb(var(--color-platform-gemini-rgb) / 58%) !important;
  box-shadow:
    0 0 0 3px rgb(var(--color-platform-gemini-rgb) / 12%),
    0 14px 28px rgb(var(--color-platform-gemini-rgb) / 12%);
}

.day-number {
  color: var(--text-primary);
  font-size: 0.96rem;
  font-weight: 700;
  line-height: 1;
}

.cell-checked .day-number {
  color: var(--accent-success);
}

.day-increment {
  color: var(--accent-success);
  font-size: 0.62rem;
  font-weight: 700;
  line-height: 1;
}

.calendar-legend {
  display: flex;
  flex-wrap: wrap;
  gap: 0.9rem;
  color: var(--text-secondary);
  font-size: 0.76rem;
  font-weight: 600;
}

.legend-item {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
}

.legend-dot {
  width: 0.65rem;
  height: 0.65rem;
  border-radius: 999px;
  flex-shrink: 0;
}

.legend-dot.checked {
  background: var(--accent-success);
}

.legend-dot.unchecked {
  background: rgb(var(--color-border-default-rgb) / 94%);
}

.legend-dot.today {
  background: white;
  border: 2px solid var(--platform-gemini);
  box-shadow: 0 0 0 2px rgb(var(--color-platform-gemini-rgb) / 14%);
}

@media (width <= 768px) {
  .calendar-cell {
    min-height: 3.35rem;
    border-radius: 0.9rem;
  }

  .day-number {
    font-size: 0.88rem;
  }

  .day-increment {
    font-size: 0.56rem;
  }
}
</style>
