<template>
  <div
    class="sparkline-container"
    :style="{ width: `${width}px`, height: `${height}px` }"
  >
    <svg
      class="sparkline-svg"
      :width="width"
      :height="height"
      :viewBox="`0 0 ${width} ${height}`"
      preserveAspectRatio="none"
    >
      <!-- Gradient fill -->
      <defs>
        <linearGradient
          :id="gradientId"
          x1="0%"
          y1="0%"
          x2="0%"
          y2="100%"
        >
          <stop
            offset="0%"
            :stop-color="color"
            stop-opacity="0.3"
          />
          <stop
            offset="100%"
            :stop-color="color"
            stop-opacity="0"
          />
        </linearGradient>
      </defs>

      <!-- Area fill -->
      <path
        v-if="showArea && areaPath"
        :d="areaPath"
        :fill="`url(#${gradientId})`"
        class="sparkline-area"
      />

      <!-- Line -->
      <path
        v-if="linePath"
        ref="lineRef"
        :d="linePath"
        fill="none"
        :stroke="color"
        :stroke-width="strokeWidth"
        stroke-linecap="round"
        stroke-linejoin="round"
        class="sparkline-line"
        :style="{ strokeDasharray: pathLength, strokeDashoffset: animated ? pathLength : 0 }"
      />

      <!-- End dot -->
      <circle
        v-if="showDot && points.length > 0"
        :cx="points[points.length - 1]?.x"
        :cy="points[points.length - 1]?.y"
        :r="dotRadius"
        :fill="color"
        class="sparkline-dot"
      />

      <!-- Glow effect for end dot -->
      <circle
        v-if="showDot && points.length > 0"
        :cx="points[points.length - 1]?.x"
        :cy="points[points.length - 1]?.y"
        :r="dotRadius * 2"
        :fill="color"
        opacity="0.3"
        class="sparkline-dot-glow animate-pulse-subtle"
      />
    </svg>

    <!-- Trend indicator -->
    <div
      v-if="showTrend && trend !== 0"
      class="trend-indicator absolute -right-1 -top-1 flex items-center gap-0.5 text-xs font-bold"
      :class="trend > 0 ? 'text-emerald-400' : 'text-rose-400'"
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
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, nextTick } from 'vue'

interface Props {
  data: number[]
  width?: number
  height?: number
  color?: string
  strokeWidth?: number
  showArea?: boolean
  showDot?: boolean
  showTrend?: boolean
  dotRadius?: number
  padding?: number
  animated?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  width: 80,
  height: 32,
  color: 'var(--color-accent-primary)',
  strokeWidth: 2,
  showArea: true,
  showDot: true,
  showTrend: false,
  dotRadius: 3,
  padding: 4,
  animated: true
})

const gradientId = computed(() => `sparkline-gradient-${Math.random().toString(36).substr(2, 9)}`)
const pathLength = ref(0)
const lineRef = ref<SVGPathElement | null>(null)

interface Point {
  x: number
  y: number
}

const points = computed((): Point[] => {
  if (!props.data || props.data.length < 2) return []

  const data = props.data
  const min = Math.min(...data)
  const max = Math.max(...data)
  const range = max - min || 1

  const effectiveWidth = props.width - props.padding * 2
  const effectiveHeight = props.height - props.padding * 2

  return data.map((value, index) => ({
    x: props.padding + (index / (data.length - 1)) * effectiveWidth,
    y: props.padding + effectiveHeight - ((value - min) / range) * effectiveHeight
  }))
})

const linePath = computed(() => {
  if (points.value.length < 2) return ''

  // Create smooth curve using Catmull-Rom spline
  const pts = points.value
  let d = `M ${pts[0].x} ${pts[0].y}`

  for (let i = 0; i < pts.length - 1; i++) {
    const p0 = pts[Math.max(0, i - 1)]
    const p1 = pts[i]
    const p2 = pts[i + 1]
    const p3 = pts[Math.min(pts.length - 1, i + 2)]

    // Catmull-Rom to Bezier conversion
    const cp1x = p1.x + (p2.x - p0.x) / 6
    const cp1y = p1.y + (p2.y - p0.y) / 6
    const cp2x = p2.x - (p3.x - p1.x) / 6
    const cp2y = p2.y - (p3.y - p1.y) / 6

    d += ` C ${cp1x} ${cp1y}, ${cp2x} ${cp2y}, ${p2.x} ${p2.y}`
  }

  return d
})

const areaPath = computed(() => {
  if (!linePath.value) return ''

  const pts = points.value
  const bottomY = props.height - props.padding

  return `${linePath.value} L ${pts[pts.length - 1].x} ${bottomY} L ${pts[0].x} ${bottomY} Z`
})

const trend = computed(() => {
  if (props.data.length < 2) return 0
  const first = props.data[0]
  const last = props.data[props.data.length - 1]
  if (first === 0) return last > 0 ? 1 : 0
  return ((last - first) / first) * 100
})

onMounted(() => {
  // Calculate path length for animation using component's own ref
  nextTick(() => {
    if (lineRef.value) {
      pathLength.value = lineRef.value.getTotalLength()
    }
  })
})
</script>

<style scoped>
.sparkline-container {
  position: relative;
  display: inline-block;
}

.sparkline-svg {
  overflow: visible;
}

.sparkline-line {
  transition: stroke-dashoffset 1.5s var(--ease-out);
}

.sparkline-area {
  opacity: 0;
  animation: area-fade-in 0.8s var(--ease-out) 0.5s forwards;
}

.sparkline-dot {
  filter: drop-shadow(0 0 4px currentcolor);
}

.sparkline-dot-glow {
  animation: dot-pulse 2s var(--ease-in-out) infinite;
}

.trend-indicator {
  font-size: 10px;
  text-shadow: 0 0 8px currentcolor;
}

@keyframes area-fade-in {
  from {
    opacity: 0;
  }

  to {
    opacity: 1;
  }
}

@keyframes dot-pulse {
  0%, 100% {
    opacity: 0.3;
    transform: scale(1);
  }

  50% {
    opacity: 0.1;
    transform: scale(1.5);
  }
}
</style>
