<!-- 极简 SVG Sparkline：只服务统计条视觉点缀，不承担真实图表能力 -->
<template>
  <svg
    :width="width"
    :height="height"
    :viewBox="`0 0 ${width} ${height}`"
    aria-hidden="true"
    role="presentation"
  >
    <polyline
      :points="points"
      fill="none"
      :stroke="color"
      stroke-width="1.4"
      stroke-linejoin="round"
      stroke-linecap="round"
    />
  </svg>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  values: number[]
  color?: string
  width?: number
  height?: number
}

const props = withDefaults(defineProps<Props>(), {
  color: 'currentColor',
  width: 60,
  height: 18,
})

const points = computed(() => {
  const vs = props.values
  if (!vs.length) return ''
  const max = Math.max(...vs, 1)
  const min = Math.min(...vs, 0)
  const range = max - min || 1
  const w = props.width
  const h = props.height
  const denom = vs.length === 1 ? 1 : vs.length - 1
  return vs
    .map((v, i) => {
      const x = (i / denom) * w
      const y = h - ((v - min) / range) * h
      return `${x.toFixed(1)},${y.toFixed(1)}`
    })
    .join(' ')
})
</script>
