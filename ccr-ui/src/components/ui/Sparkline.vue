<!-- 统一 SVG Sparkline：合并原三份实现（profiles 统计条版 + usage 指标卡两版）为单一组件。
     两档视觉：默认裸折线（统计条点缀）；传入 fill 时叠加渐变面积 + 端点圆点（指标卡趋势）。 -->
<template>
  <svg
    class="ui-sparkline"
    :width="width"
    :height="height"
    :viewBox="`0 0 ${width} ${height}`"
    preserveAspectRatio="none"
    :role="label ? 'img' : 'presentation'"
    :aria-label="label || undefined"
    :aria-hidden="label ? undefined : 'true'"
  >
    <defs v-if="fill">
      <linearGradient
        :id="gradientId"
        x1="0"
        y1="0"
        x2="0"
        y2="1"
      >
        <stop
          offset="0%"
          :stop-color="fill"
          stop-opacity="0.28"
        />
        <stop
          offset="100%"
          :stop-color="fill"
          stop-opacity="0.02"
        />
      </linearGradient>
    </defs>

    <polyline
      v-if="fill && linePoints"
      class="ui-sparkline__area"
      :points="areaPoints"
      :fill="`url(#${gradientId})`"
    />
    <polyline
      v-if="linePoints"
      class="ui-sparkline__line"
      :points="linePoints"
      fill="none"
      :stroke="stroke"
      :stroke-width="strokeWidth"
    />
    <circle
      v-if="fill && lastPoint"
      class="ui-sparkline__dot"
      :cx="lastPoint.x"
      :cy="lastPoint.y"
      r="2"
      :fill="stroke"
    />
  </svg>
</template>

<script setup lang="ts">
import { computed, useId } from 'vue'

interface Props {
  /** 数据序列；空数组时不渲染任何图形 */
  values: number[]
  width?: number
  height?: number
  /** 折线颜色，默认继承 currentColor */
  stroke?: string
  strokeWidth?: number
  /** 传入即启用渐变面积 + 端点圆点（趋势档）；省略则为裸折线（点缀档） */
  fill?: string
  /** 提供则作为 role="img" 的无障碍标签，否则整图对读屏隐藏 */
  label?: string
}

const props = withDefaults(defineProps<Props>(), {
  width: 60,
  height: 18,
  stroke: 'currentColor',
  strokeWidth: 1.4,
  fill: undefined,
  label: undefined,
})

const gradientId = `ui-sparkline-${useId()}`

// 面积档给端点圆点留出上下留白；裸线档贴边（沿用原统计条版行为）
const padY = computed(() => (props.fill ? 4 : 0))
const baseline = computed(() => props.height - padY.value)

const points = computed(() => {
  const vs = props.values
  if (!vs.length) return [] as Array<{ x: number; y: number }>
  const min = Math.min(...vs)
  const max = Math.max(...vs)
  const range = Math.max(max - min, 1)
  const usableH = props.height - padY.value * 2
  const denom = Math.max(vs.length - 1, 1)
  return vs.map((v, i) => ({
    x: vs.length === 1 ? props.width / 2 : (i / denom) * props.width,
    y: baseline.value - ((v - min) / range) * usableH,
  }))
})

const linePoints = computed(() =>
  points.value.map((p) => `${p.x.toFixed(1)},${p.y.toFixed(1)}`).join(' '),
)

const areaPoints = computed(() =>
  linePoints.value ? `0,${baseline.value} ${linePoints.value} ${props.width},${baseline.value}` : '',
)

const lastPoint = computed(() => points.value.at(-1) ?? null)
</script>

<style scoped>
.ui-sparkline {
  display: block;
  overflow: visible;
}

.ui-sparkline__area {
  stroke: none;
}

.ui-sparkline__line {
  fill: none;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.ui-sparkline__dot {
  stroke: rgb(var(--color-bg-elevated-rgb) / 92%);
  stroke-width: 1.6;
}
</style>
