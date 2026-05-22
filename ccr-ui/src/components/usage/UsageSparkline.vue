<template>
  <svg
    class="usage-sparkline"
    :class="`usage-sparkline--${tone}`"
    :viewBox="viewBox"
    role="img"
    :aria-label="label"
    preserveAspectRatio="none"
  >
    <defs>
      <linearGradient
        :id="gradientId"
        x1="0"
        y1="0"
        x2="0"
        y2="1"
      >
        <stop
          offset="0%"
          stop-color="currentColor"
          stop-opacity="0.28"
        />
        <stop
          offset="100%"
          stop-color="currentColor"
          stop-opacity="0.02"
        />
      </linearGradient>
    </defs>

    <polyline
      v-if="linePoints"
      class="usage-sparkline__area"
      :points="areaPoints"
      :fill="`url(#${gradientId})`"
    />
    <polyline
      v-if="linePoints"
      class="usage-sparkline__line"
      :points="linePoints"
    />
    <circle
      v-if="lastPoint"
      class="usage-sparkline__dot"
      :cx="lastPoint.x"
      :cy="lastPoint.y"
      r="2"
    />
  </svg>
</template>

<script setup lang="ts">
import { computed } from 'vue'

export type UsageSparklineTone = 'rose' | 'sand' | 'sky' | 'amber'

export interface UsageSparklinePoint {
  label: string
  value: number
}

interface Props {
  points: UsageSparklinePoint[]
  tone: UsageSparklineTone
  label: string
}

const props = defineProps<Props>()

const width = 120
const height = 38
const padding = 4
const baseline = height - padding
const viewBox = `0 0 ${width} ${height}`
const gradientId = `usage-sparkline-${Math.random().toString(36).slice(2)}`

const normalizedPoints = computed(() => {
  const values = props.points.map((point) => point.value)
  if (values.length === 0) return []

  const min = Math.min(...values)
  const max = Math.max(...values)
  const range = Math.max(max - min, 1)
  const denominator = Math.max(values.length - 1, 1)

  return values.map((value, index) => {
    const x = values.length === 1 ? width / 2 : (index / denominator) * width
    const y = baseline - ((value - min) / range) * (height - padding * 2)
    return { x, y }
  })
})

const linePoints = computed(() =>
  normalizedPoints.value.map((point) => `${point.x.toFixed(2)},${point.y.toFixed(2)}`).join(' '),
)

const areaPoints = computed(() => {
  if (!linePoints.value) return ''
  return `0,${baseline} ${linePoints.value} ${width},${baseline}`
})

const lastPoint = computed(() => normalizedPoints.value.at(-1) ?? null)
</script>

<style scoped>
.usage-sparkline {
  --usage-sparkline-rgb: var(--color-accent-primary-rgb);

  display: block;
  width: 100%;
  height: 2.45rem;
  color: rgb(var(--usage-sparkline-rgb) / 92%);
  overflow: visible;
}

.usage-sparkline--rose {
  --usage-sparkline-rgb: var(--color-accent-primary-rgb);
}

.usage-sparkline--sand {
  --usage-sparkline-rgb: var(--color-accent-secondary-rgb);
}

.usage-sparkline--sky {
  --usage-sparkline-rgb: var(--color-info-rgb);
}

.usage-sparkline--amber {
  --usage-sparkline-rgb: var(--color-warning-rgb);
}

.usage-sparkline__area {
  stroke: none;
}

.usage-sparkline__line {
  fill: none;
  stroke: currentcolor;
  stroke-width: 2.4;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.usage-sparkline__dot {
  fill: currentcolor;
  stroke: rgb(var(--color-bg-elevated-rgb) / 92%);
  stroke-width: 1.6;
}
</style>
