<template>
  <div>
    <div class="grid grid-cols-7 gap-2 text-xs text-gray-500 dark:text-gray-400 mb-2">
      <div
        v-for="label in weekLabels"
        :key="label"
        class="text-center"
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
        class="h-16 rounded-lg flex flex-col items-center justify-center text-xs border border-transparent transition"
        :class="cellClass(cell)"
        :title="cell ? buildTitle(cell) : ''"
      >
        <span v-if="cell">{{ getDayNumber(cell.date) }}</span>
        <span
          v-if="cell && cell.income_increment"
          class="text-[10px] text-emerald-600 dark:text-emerald-400"
        >
          +{{ cell.income_increment.toFixed(2) }}
        </span>
      </div>
    </div>

    <div class="flex items-center gap-4 text-xs text-gray-500 dark:text-gray-400 mt-3">
      <div class="flex items-center gap-1">
        <span class="inline-block w-2 h-2 rounded-full bg-emerald-500" />
        已签到
      </div>
      <div class="flex items-center gap-1">
        <span class="inline-block w-2 h-2 rounded-full bg-gray-300 dark:bg-gray-600" />
        未签到
      </div>
      <div class="flex items-center gap-1">
        <span class="inline-block w-2 h-2 rounded-full border border-blue-500" />
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
  if (!cell) {
    return 'bg-transparent'
  }

  const isToday = cell.date === todayString.value
  const base = cell.is_checked_in
    ? 'bg-emerald-50 text-emerald-700 border-emerald-200 dark:bg-emerald-500/20 dark:text-emerald-200 dark:border-emerald-500/40'
    : 'bg-gray-50 text-gray-600 border-gray-200 dark:bg-gray-700/40 dark:text-gray-300 dark:border-gray-700'
  const ring = isToday ? 'ring-2 ring-blue-500/70' : ''
  return `${base} ${ring} hover:shadow-sm`
}

const buildTitle = (cell: CheckinDashboardDay) => {
  const status = cell.is_checked_in ? '已签到' : '未签到'
  const increment = cell.income_increment ? `+${cell.income_increment.toFixed(2)}` : '-'
  return `${cell.date} · ${status} · 增量 ${increment}`
}
</script>
