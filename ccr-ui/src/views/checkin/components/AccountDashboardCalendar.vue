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
            v-if="dayReward(cell) !== null"
            class="day-increment"
          >
            +{{ dayReward(cell)!.toFixed(2) }}
          </span>
          <span
            v-else-if="cell.is_checked_in"
            class="day-dot"
            aria-hidden="true"
          >·</span>
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

const dayReward = (cell: CheckinDashboardDay): number | null => {
  const amount = cell.reward_amount ?? cell.income_increment
  if (amount === undefined || amount === null || amount <= 0) return null
  return amount
}

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
  const reward = dayReward(cell)
  const rewardText = reward !== null ? `+${reward.toFixed(2)}` : '-'
  return `${cell.date} · ${status} · 奖励 ${rewardText}`
}
</script>

<style scoped>
.calendar-wrapper {
  display: flex;
  flex-direction: column;
  gap: 0.85rem;
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
  font-size: 0.7rem;
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
  font-size: 0.88rem;
}

.calendar-grid {
  gap: 0.5rem;
}

.calendar-cell {
  position: relative;
  min-height: 3.9rem;
  border-radius: 0.85rem;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.22rem;
  border: 1px solid transparent;
  transition:
    transform 0.18s ease,
    border-color 0.18s ease,
    background-color 0.18s ease;
}

.calendar-cell:hover:not(.cell-empty) {
  transform: translateY(-1px);
}

.cell-empty {
  background: transparent;
  border-color: transparent;
}

.cell-checked {
  background: rgb(var(--color-accent-primary-rgb) / 12%);
  border-color: rgb(var(--color-accent-primary-rgb) / 34%);
  box-shadow: var(--shadow-inner);
}

.cell-unchecked {
  background: rgb(var(--color-bg-elevated-rgb) / 62%);
  border-color: rgb(var(--color-border-default-rgb) / 56%);
}

.cell-today {
  border-color: rgb(var(--color-accent-primary-rgb) / 62%) !important;
  box-shadow:
    0 0 0 2px rgb(var(--color-accent-primary-rgb) / 14%),
    0 8px 18px rgb(var(--color-accent-primary-rgb) / 10%);
}

.day-number {
  color: var(--text-primary);
  font-size: 0.92rem;
  font-weight: 700;
  line-height: 1;
  font-family: var(--font-mono);
}

.cell-checked .day-number {
  color: var(--accent-primary);
}

.day-increment {
  color: var(--accent-primary);
  font-size: 0.6rem;
  font-weight: 700;
  line-height: 1;
  font-family: var(--font-mono);
}

.day-dot {
  color: rgb(var(--color-accent-primary-rgb) / 78%);
  font-size: 1rem;
  line-height: 0.4;
  font-weight: 900;
}

.calendar-legend {
  display: flex;
  flex-wrap: wrap;
  gap: 0.85rem;
  color: var(--text-secondary);
  font-size: 0.75rem;
  font-weight: 600;
}

.legend-item {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
}

.legend-dot {
  width: 0.6rem;
  height: 0.6rem;
  border-radius: 999px;
  flex-shrink: 0;
}

.legend-dot.checked {
  background: var(--accent-primary);
}

.legend-dot.unchecked {
  background: rgb(var(--color-border-default-rgb) / 82%);
}

.legend-dot.today {
  background: transparent;
  border: 2px solid var(--accent-primary);
  box-shadow: 0 0 0 2px rgb(var(--color-accent-primary-rgb) / 14%);
}

@media (width <= 768px) {
  .calendar-cell {
    min-height: 3.2rem;
    border-radius: 0.75rem;
  }

  .day-number {
    font-size: 0.84rem;
  }

  .day-increment {
    font-size: 0.56rem;
  }
}
</style>
