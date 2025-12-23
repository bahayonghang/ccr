<template>
  <div>
    <div
      v-if="chartData.length === 0"
      class="text-sm text-gray-400 dark:text-gray-500 text-center py-12"
    >
      暂无趋势数据
    </div>
    <div v-else>
      <div class="trend-chart h-44 flex items-end gap-1">
        <div
          v-for="(point, index) in chartData"
          :key="`${point.date}-${index}`"
          class="flex-1 flex flex-col items-center"
        >
          <div
            class="w-full rounded-sm trend-bar"
            :class="point.is_checked_in ? 'is-checked' : 'is-empty'"
            :style="{ height: `${getBarHeight(point.income_increment)}%` }"
            :title="buildTitle(point)"
          />
        </div>
      </div>
      <div class="mt-2 flex justify-between text-[10px] text-gray-400 dark:text-gray-500">
        <span>{{ trend?.start_date }}</span>
        <span>{{ trend?.end_date }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { CheckinDashboardTrend } from '@/types/checkin'

const props = defineProps<{
  trend: CheckinDashboardTrend | null
}>()

const chartData = computed(() => props.trend?.data_points ?? [])

const maxValue = computed(() => {
  if (chartData.value.length === 0) return 1
  return Math.max(...chartData.value.map(point => point.income_increment), 1)
})

const getBarHeight = (value: number) => {
  const ratio = value / maxValue.value
  const percent = ratio * 100
  return Math.max(percent, 4)
}

const buildTitle = (point: { date: string; income_increment: number; current_balance: number }) => {
  return `${point.date} · 增量 ${point.income_increment.toFixed(2)} · 余额 ${point.current_balance.toFixed(2)}`
}
</script>

<style scoped>
.trend-chart {
  position: relative;
  padding-top: 0.5rem;
  border-radius: 0.75rem;
  background: linear-gradient(180deg, rgba(148, 163, 184, 0.08), transparent);
}

.trend-bar {
  transition: transform 0.2s ease;
}

.trend-bar.is-checked {
  background: linear-gradient(180deg, rgba(59, 130, 246, 0.85), rgba(59, 130, 246, 0.35));
}

.trend-bar.is-empty {
  background: linear-gradient(180deg, rgba(148, 163, 184, 0.5), rgba(148, 163, 184, 0.2));
}

.trend-bar:hover {
  transform: translateY(-2px);
}

:global(.dark) .trend-chart {
  background: linear-gradient(180deg, rgba(30, 41, 59, 0.6), transparent);
}
</style>
