<template>
  <div class="activity-heatmap">
    <ActivityHeatmapHeader />

    <ActivityHeatmapGrid
      :loading="loading"
      :loading-label="$t('common.loading', '加载中...')"
      :month-labels="monthLabels"
      :weeks="weeks"
      @hover-day="handleTooltipShow"
      @leave-day="hideTooltip"
    />

    <ActivityHeatmapTooltip
      :tooltip="tooltip"
      :formatted-count="formatNumber(tooltip.count)"
    />

    <ActivityHeatmapStats :items="statItems" />
  </div>
</template>

<script setup lang="ts">
import ActivityHeatmapGrid from '@/components/activity/ActivityHeatmapGrid.vue'
import ActivityHeatmapHeader from '@/components/activity/ActivityHeatmapHeader.vue'
import ActivityHeatmapStats from '@/components/activity/ActivityHeatmapStats.vue'
import ActivityHeatmapTooltip from '@/components/activity/ActivityHeatmapTooltip.vue'
import { getHeatmapData } from '@/api'
import type {
  ActivityHeatmapData,
  ActivityHeatmapDayData,
  ActivityHeatmapMonthLabel,
  ActivityHeatmapStatItem,
  ActivityHeatmapTooltipState,
} from '@/types/activityHeatmap'
import { logger } from '@/utils/logger'
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const loading = ref(true)
const heatmapData = ref<ActivityHeatmapData | null>(null)
const tooltip = ref<ActivityHeatmapTooltipState>({
  visible: false,
  date: '',
  count: 0,
  x: 0,
  y: 0,
})

const startDate = computed(() => {
  const today = new Date()
  const start = new Date(today)
  start.setDate(start.getDate() - 364)
  start.setDate(start.getDate() - start.getDay())
  return start
})

onMounted(async () => {
  try {
    heatmapData.value = await getHeatmapData()
  } catch (error) {
    logger.error('Failed to load heatmap data:', error)
    heatmapData.value = { data: {}, max_value: 0, total_tokens: 0, active_days: 0 }
  } finally {
    loading.value = false
  }
})

const weeks = computed<Array<Array<ActivityHeatmapDayData | null>>>(() => {
  const result: Array<Array<ActivityHeatmapDayData | null>> = []
  const today = new Date()
  const todayKey = today.toISOString().split('T')[0]
  const maxValue = heatmapData.value?.max_value || 1
  const data = heatmapData.value?.data || {}
  const start = startDate.value

  for (let week = 0; week < 53; week++) {
    const weekDays: Array<ActivityHeatmapDayData | null> = []

    for (let day = 0; day < 7; day++) {
      const currentDate = new Date(start)
      currentDate.setDate(currentDate.getDate() + (week * 7) + day)

      if (currentDate > today) {
        weekDays.push(null)
        continue
      }

      const dateKey = currentDate.toISOString().split('T')[0]
      const count = data[dateKey] || 0

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
          weekday: 'short',
        }),
        dateKey,
        count,
        level,
        isToday: dateKey === todayKey,
      })
    }

    result.push(weekDays)
  }

  return result
})

const monthLabels = computed<ActivityHeatmapMonthLabel[]>(() => {
  const labels: ActivityHeatmapMonthLabel[] = []
  const start = startDate.value
  let lastMonth = -1

  for (let week = 0; week < 53; week++) {
    const weekStart = new Date(start)
    weekStart.setDate(weekStart.getDate() + (week * 7))
    const month = weekStart.getMonth()

    if (month !== lastMonth) {
      labels.push({
        name: weekStart.toLocaleDateString('zh-CN', { month: 'short' }),
        weekOffset: week,
      })
      lastMonth = month
    }
  }

  return labels
})

const statItems = computed<ActivityHeatmapStatItem[]>(() => [
  {
    id: 'activeDays',
    label: t('activityHeatmap.activeDays', '活跃天数'),
    value: String(heatmapData.value?.active_days ?? 0),
  },
  {
    id: 'totalTokens',
    label: t('activityHeatmap.totalTokens', '总 Token 数'),
    value: formatNumber(heatmapData.value?.total_tokens ?? 0),
  },
])

function handleTooltipShow(payload: { day: ActivityHeatmapDayData; event: MouseEvent }) {
  const rect = (payload.event.target as HTMLElement).getBoundingClientRect()
  tooltip.value = {
    visible: true,
    date: payload.day.date,
    count: payload.day.count,
    x: rect.left + rect.width / 2,
    y: rect.top - 8,
  }
}

function hideTooltip() {
  tooltip.value.visible = false
}

function formatNumber(num: number): string {
  if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`
  if (num >= 1000) return `${(num / 1000).toFixed(1)}K`
  return num.toLocaleString()
}
</script>

<style scoped>
.activity-heatmap {
  background: var(--glass-bg-light);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-xl);
  padding: var(--space-5);
  backdrop-filter: var(--glass-blur-sm);
  box-shadow: var(--glass-shadow);
}
</style>
