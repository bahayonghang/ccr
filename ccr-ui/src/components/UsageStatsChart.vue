<template>
  <div class="usage-stats-chart h-72 relative select-none">
    <div class="absolute top-0 right-0 flex items-center gap-4 text-xs z-10">
      <div class="flex items-center gap-1.5">
        <span class="w-3 h-3 rounded-[2px] bg-orange-500" />
        <span class="text-text-secondary">Codex</span>
      </div>
      <div class="flex items-center gap-1.5">
        <span class="w-3 h-3 rounded-[2px] bg-pink-400" />
        <span class="text-text-secondary">Claude</span>
      </div>
      <div class="flex items-center gap-1.5">
        <span class="w-3 h-3 rounded-[2px] bg-blue-500" />
        <span class="text-text-secondary">Gemini</span>
      </div>
    </div>

    <div class="flex flex-col h-full pt-8 pb-6">
      <div class="flex-1 flex items-end justify-between relative gap-[2px]">
        <div
          class="absolute bottom-[25%] left-0 right-0 h-px bg-border-default/15 pointer-events-none"
        />
        <div
          class="absolute bottom-[50%] left-0 right-0 h-px bg-border-default/20 pointer-events-none"
        />
        <div
          class="absolute bottom-[75%] left-0 right-0 h-px bg-border-default/15 pointer-events-none"
        />
        <div class="absolute bottom-0 left-0 right-0 h-px bg-border-default/50" />

        <div
          v-for="(row, index) in chartRows"
          :key="row.date"
          class="flex-1 h-full flex flex-col justify-end items-center cursor-pointer group relative"
          @mouseenter="setHoveredIndex(index)"
          @mouseleave="setHoveredIndex(null)"
        >
          <div
            class="absolute inset-x-0 top-0 bottom-0 bg-text-primary/5 rounded opacity-0 transition-opacity duration-200"
            :class="{ 'opacity-100': hoveredIndex === index }"
          />

          <div
            class="w-full max-w-[80%] flex flex-col-reverse items-center relative z-10 transition-transform duration-300"
          >
            <div
              v-if="row.gemini > 0"
              class="w-full bg-blue-500"
              :style="{ height: `${row.geminiHeight}px` }"
              :class="[
                { 'opacity-90': hoveredIndex !== null && hoveredIndex !== index },
                row.geminiCorner,
              ]"
            />
            <div
              v-if="row.claude > 0"
              class="w-full bg-pink-400"
              :style="{ height: `${row.claudeHeight}px` }"
              :class="[
                { 'opacity-90': hoveredIndex !== null && hoveredIndex !== index },
                row.claudeCorner,
              ]"
            />
            <div
              v-if="row.codex > 0"
              class="w-full bg-orange-500"
              :style="{ height: `${row.codexHeight}px` }"
              :class="[
                { 'opacity-90': hoveredIndex !== null && hoveredIndex !== index },
                row.codexCorner,
              ]"
            />
          </div>

          <div
            class="w-px bg-border-default/50 mt-[1px]"
            :class="row.showDate ? 'h-1.5' : 'h-1'"
          />

          <span
            v-if="row.showDate"
            class="absolute top-[100%] mt-1 text-[10px] text-text-muted font-mono"
          >
            {{ row.dayLabel }}
          </span>

          <div
            v-if="row.isMonthStart"
            class="absolute top-[100%] mt-5 left-0 pl-1 border-l border-accent-secondary/50 h-3 flex items-center"
          >
            <span
              class="text-[10px] font-bold text-accent-secondary ml-1 uppercase tracking-wider whitespace-nowrap"
            >
              {{ row.monthLabel }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <Transition name="fade">
      <div
        v-if="hoveredRow"
        class="absolute z-20 px-3 py-2 rounded-lg bg-bg-base/95 border border-border-default shadow-xl backdrop-blur-md text-xs pointer-events-none transform -translate-x-1/2 transition-all duration-75"
        :style="tooltipStyle"
      >
        <div class="font-bold text-text-primary mb-1.5 border-b border-border-default/50 pb-1">
          {{ formatDateFull(hoveredRow.date) }}
        </div>
        <div class="space-y-1">
          <div class="flex items-center justify-between gap-6">
            <div class="flex items-center gap-1.5">
              <span class="w-2 h-2 rounded-[1px] bg-orange-500" />
              <span class="text-text-secondary">Codex</span>
            </div>
            <span class="text-text-primary font-mono font-medium">
              {{ formatValue(hoveredRow.codex) }}
            </span>
          </div>
          <div class="flex items-center justify-between gap-6">
            <div class="flex items-center gap-1.5">
              <span class="w-2 h-2 rounded-[1px] bg-pink-400" />
              <span class="text-text-secondary">Claude</span>
            </div>
            <span class="text-text-primary font-mono font-medium">
              {{ formatValue(hoveredRow.claude) }}
            </span>
          </div>
          <div class="flex items-center justify-between gap-6">
            <div class="flex items-center gap-1.5">
              <span class="w-2 h-2 rounded-[1px] bg-blue-500" />
              <span class="text-text-secondary">Gemini</span>
            </div>
            <span class="text-text-primary font-mono font-medium">
              {{ formatValue(hoveredRow.gemini) }}
            </span>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from 'vue'
import type { HomeOverviewSeriesItem, HomeOverviewViewMode } from '@/types/usage'

const props = withDefaults(
  defineProps<{
    data?: HomeOverviewSeriesItem[]
    viewMode?: HomeOverviewViewMode
  }>(),
  {
    data: () => [],
    viewMode: 'sessions',
  }
)

const hoveredIndex = ref<number | null>(null)
const pendingHoverIndex = ref<number | null>(null)
let hoverRafId: number | null = null

const getValue = (
  item: HomeOverviewSeriesItem,
  platform: 'claude' | 'codex' | 'gemini'
): number => {
  const stats = item?.[platform]
  if (!stats) return 0
  switch (props.viewMode) {
    case 'sessions':
      return stats.sessions ?? 0
    case 'requests':
      return stats.requests ?? 0
    case 'tokens':
      return stats.tokens ?? 0
    default:
      return stats.sessions ?? 0
  }
}

const chartData = computed(() => props.data ?? [])

const maxValue = computed(() => {
  let max = 0
  for (const item of chartData.value) {
    const total = getValue(item, 'claude') + getValue(item, 'codex') + getValue(item, 'gemini')
    if (total > max) max = total
  }
  return max || 1
})

const getBarHeight = (value: number): number => {
  const maxHeight = 150
  return Math.max(0, (value / maxValue.value) * maxHeight)
}

const chartRows = computed(() => {
  const rows = chartData.value
  const labelInterval = rows.length > 60 ? 7 : rows.length > 30 ? 5 : 2
  return rows.map((item, index) => {
    const codex = getValue(item, 'codex')
    const claude = getValue(item, 'claude')
    const gemini = getValue(item, 'gemini')

    const codexCorner = gemini === 0 && claude === 0 && codex > 0 ? 'rounded-t-[2px]' : ''
    const claudeCorner = gemini === 0 && claude > 0 ? 'rounded-t-[2px]' : ''
    const geminiCorner = gemini > 0 ? 'rounded-t-[2px]' : ''

    const date = new Date(item.date)
    const prevDate = index > 0 ? new Date(rows[index - 1].date) : null
    const isMonthStart = index === 0 || (prevDate ? date.getMonth() !== prevDate.getMonth() : false)
    const isLast = index === rows.length - 1
    const showDate = isMonthStart || isLast || index % labelInterval === 0

    return {
      date: item.date,
      codex,
      claude,
      gemini,
      codexHeight: getBarHeight(codex),
      claudeHeight: getBarHeight(claude),
      geminiHeight: getBarHeight(gemini),
      codexCorner,
      claudeCorner,
      geminiCorner,
      isMonthStart,
      showDate,
      dayLabel: date.getDate().toString(),
      monthLabel: date.toLocaleString('en-US', { month: 'short' }),
    }
  })
})

const hoveredRow = computed(() => {
  if (hoveredIndex.value === null) return null
  return chartRows.value[hoveredIndex.value] ?? null
})

const setHoveredIndex = (index: number | null) => {
  pendingHoverIndex.value = index
  if (hoverRafId !== null) return
  hoverRafId = requestAnimationFrame(() => {
    hoveredIndex.value = pendingHoverIndex.value
    hoverRafId = null
  })
}

onBeforeUnmount(() => {
  if (hoverRafId !== null) {
    cancelAnimationFrame(hoverRafId)
    hoverRafId = null
  }
})

const formatDateFull = (dateStr: string): string => {
  const date = new Date(dateStr)
  return date.toLocaleDateString('en-US', { weekday: 'short', month: 'short', day: 'numeric' })
}

const formatValue = (value: number): string => {
  return value.toLocaleString()
}

const tooltipStyle = computed(() => {
  if (hoveredIndex.value === null) return {}
  const count = chartRows.value.length
  if (count <= 1) {
    return { left: '50%', transform: 'translateX(-50%)', bottom: '80%' }
  }

  const percent = (hoveredIndex.value / (count - 1)) * 100
  let left = `${percent}%`
  let transform = 'translateX(-50%)'

  if (hoveredIndex.value < 2) {
    left = '0%'
    transform = 'translateX(0)'
  } else if (hoveredIndex.value > count - 3) {
    left = '100%'
    transform = 'translateX(-100%)'
  }

  return {
    left,
    transform,
    bottom: '80%',
  }
})
</script>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition:
    opacity 0.15s ease,
    transform 0.15s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: translateY(4px) translateX(-50%);
}
</style>
