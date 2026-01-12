<template>
  <div
    class="stat-card group relative overflow-hidden rounded-2xl p-5 transition-all duration-300"
    :class="[
      'backdrop-blur-xl border',
      'bg-gradient-to-br from-white/90 to-white/70 dark:from-gray-800/80 dark:to-gray-900/60',
      'border-gray-200/50 dark:border-gray-700/50',
      'hover:shadow-2xl hover:scale-[1.02] hover:border-opacity-80'
    ]"
    :style="{ '--accent': `var(--color-${colorVar})` }"
  >
    <!-- Animated gradient background on hover -->
    <div
      class="absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none"
      :style="{
        background: `radial-gradient(circle at 80% 20%, ${glowColor}15 0%, transparent 50%)`
      }"
    />

    <!-- Top accent line -->
    <div
      class="absolute top-0 left-0 right-0 h-1 opacity-0 group-hover:opacity-100 transition-all duration-300"
      :style="{ background: `linear-gradient(90deg, transparent, ${glowColor}, transparent)` }"
    />

    <!-- Content -->
    <div class="relative z-10 flex items-start justify-between gap-4">
      <!-- Left: Text content -->
      <div class="flex-1 min-w-0">
        <!-- Label -->
        <p class="text-xs font-semibold uppercase tracking-wider mb-2 text-gray-500 dark:text-gray-400">
          {{ label }}
        </p>

        <!-- Value with animation -->
        <div class="flex items-baseline gap-2 mb-2">
          <span
            class="text-3xl font-black text-gray-900 dark:text-white tracking-tight"
            :class="{ 'font-mono tabular-nums': isNumeric }"
          >
            <AnimatedCounter
              v-if="isNumeric"
              :value="numericValue"
              :format="format"
              :decimals="decimals"
              :duration="800"
            />
            <span v-else>{{ value }}</span>
          </span>
          <span
            v-if="suffix"
            class="text-sm font-medium text-gray-500 dark:text-gray-400"
          >
            {{ suffix }}
          </span>
        </div>

        <!-- Sparkline -->
        <div
          v-if="sparklineData && sparklineData.length > 1"
          class="mt-3"
        >
          <SparkLine
            :data="sparklineData"
            :width="100"
            :height="28"
            :color="glowColor"
            :show-area="true"
            :show-dot="true"
            :animated="true"
          />
        </div>

        <!-- Trend indicator -->
        <div
          v-if="trend !== undefined"
          class="flex items-center gap-1.5 mt-2"
        >
          <span
            class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-bold"
            :class="trendClasses"
          >
            <svg
              class="w-3 h-3"
              :class="{ 'rotate-180': trend < 0 }"
              fill="currentColor"
              viewBox="0 0 20 20"
            >
              <path
                fill-rule="evenodd"
                d="M5.293 9.707a1 1 0 010-1.414l4-4a1 1 0 011.414 0l4 4a1 1 0 01-1.414 1.414L11 7.414V15a1 1 0 11-2 0V7.414L6.707 9.707a1 1 0 01-1.414 0z"
                clip-rule="evenodd"
              />
            </svg>
            {{ Math.abs(trend).toFixed(1) }}%
          </span>
          <span class="text-xs text-gray-500 dark:text-gray-400">{{ trendLabel }}</span>
        </div>
      </div>

      <!-- Right: Icon -->
      <div
        class="flex-shrink-0 p-3 rounded-xl transition-all duration-300 group-hover:scale-110 group-hover:rotate-3"
        :class="iconBgClasses"
      >
        <slot name="icon">
          <component
            :is="iconComponent"
            v-if="iconComponent"
            class="w-7 h-7"
            :class="iconClasses"
          />
        </slot>
      </div>
    </div>

    <!-- Pulse effect for active state -->
    <div
      v-if="pulse"
      class="absolute top-3 right-3 w-2 h-2 rounded-full animate-pulse-subtle"
      :style="{ backgroundColor: glowColor }"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import AnimatedCounter from './AnimatedCounter.vue'
import SparkLine from './SparkLine.vue'

interface Props {
  label: string
  value: number | string
  suffix?: string
  format?: 'number' | 'compact' | 'percent'
  decimals?: number
  accentColor?: 'blue' | 'green' | 'amber' | 'purple' | 'red' | 'cyan' | 'teal'
  trend?: number
  trendLabel?: string
  sparklineData?: number[]
  pulse?: boolean
  iconComponent?: object
}

const props = withDefaults(defineProps<Props>(), {
  format: 'compact',
  decimals: 1,
  accentColor: 'blue',
  trendLabel: 'vs last period',
  pulse: false
})

const isNumeric = computed(() => typeof props.value === 'number')
const numericValue = computed(() => (typeof props.value === 'number' ? props.value : 0))

const colorVar = computed(() => {
  const colorMap: Record<string, string> = {
    blue: 'info',
    green: 'success',
    amber: 'warning',
    purple: 'accent-secondary',
    red: 'danger',
    cyan: 'accent-primary',
    teal: 'teal'
  }
  return colorMap[props.accentColor] || 'info'
})

const glowColor = computed(() => {
  const colorMap: Record<string, string> = {
    blue: '#3B82F6',
    green: '#10B981',
    amber: '#F59E0B',
    purple: '#8B5CF6',
    red: '#EF4444',
    cyan: '#06B6D4',
    teal: '#14B8A6'
  }
  return colorMap[props.accentColor] || '#3B82F6'
})

const iconBgClasses = computed(() => {
  const bgMap: Record<string, string> = {
    blue: 'bg-blue-500/10 dark:bg-blue-400/10',
    green: 'bg-emerald-500/10 dark:bg-emerald-400/10',
    amber: 'bg-amber-500/10 dark:bg-amber-400/10',
    purple: 'bg-purple-500/10 dark:bg-purple-400/10',
    red: 'bg-red-500/10 dark:bg-red-400/10',
    cyan: 'bg-cyan-500/10 dark:bg-cyan-400/10',
    teal: 'bg-teal-500/10 dark:bg-teal-400/10'
  }
  return bgMap[props.accentColor] || bgMap.blue
})

const iconClasses = computed(() => {
  const iconMap: Record<string, string> = {
    blue: 'text-blue-600 dark:text-blue-400',
    green: 'text-emerald-600 dark:text-emerald-400',
    amber: 'text-amber-600 dark:text-amber-400',
    purple: 'text-purple-600 dark:text-purple-400',
    red: 'text-red-600 dark:text-red-400',
    cyan: 'text-cyan-600 dark:text-cyan-400',
    teal: 'text-teal-600 dark:text-teal-400'
  }
  return iconMap[props.accentColor] || iconMap.blue
})

const trendClasses = computed(() => {
  if (props.trend === undefined) return ''
  if (props.trend > 0) {
    return 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400'
  } else if (props.trend < 0) {
    return 'bg-red-500/10 text-red-600 dark:text-red-400'
  }
  return 'bg-gray-500/10 text-gray-600 dark:text-gray-400'
})
</script>

<style scoped>
.stat-card {
  box-shadow:
    0 4px 6px -1px rgba(0, 0, 0, 0.1),
    0 2px 4px -1px rgba(0, 0, 0, 0.06),
    inset 0 1px 0 rgba(255, 255, 255, 0.1);
}

.stat-card:hover {
  box-shadow:
    0 20px 25px -5px rgba(0, 0, 0, 0.1),
    0 10px 10px -5px rgba(0, 0, 0, 0.04),
    inset 0 1px 0 rgba(255, 255, 255, 0.15);
}

:deep(.dark) .stat-card {
  box-shadow:
    0 4px 6px -1px rgba(0, 0, 0, 0.3),
    0 2px 4px -1px rgba(0, 0, 0, 0.2),
    inset 0 1px 0 rgba(255, 255, 255, 0.05);
}

:deep(.dark) .stat-card:hover {
  box-shadow:
    0 20px 25px -5px rgba(0, 0, 0, 0.4),
    0 10px 10px -5px rgba(0, 0, 0, 0.2),
    inset 0 1px 0 rgba(255, 255, 255, 0.08);
}
</style>
