<template>
  <ChartPreparingState
    v-if="degraded"
    :label="label"
  />
  <slot
    v-else
    :reload-key="reloadKey"
  />
</template>

<script setup lang="ts">
import { onErrorCaptured, ref } from 'vue'
import ChartPreparingState from './ChartPreparingState.vue'
import { logger } from '@/utils/logger'

/*
 * ========================================================================
 * 图表自愈式错误边界
 * ========================================================================
 * 背景：
 *   ApexCharts(vue3-apexcharts) 在「异步 init + 动画」的生命周期窗口里，可能解引用
 *   已被移除的 SVG 元素，抛出 `Cannot read properties of undefined (reading 'node')`。
 *   该异常经子组件 watcher / 生命周期同步抛出，会冒泡到 main.ts 全局 errorHandler，
 *   弹出“应用错误”吐司并留下空白图。
 * 策略：
 *   1) onErrorCaptured 就近接住 → return false 阻断冒泡（消除全局吐司）。
 *   2) 有限次「干净重挂」（换 reloadKey）：易变窗口过去后图表通常能正常渲染。
 *   3) 超过重试上限则降级为准备态，避免无限重挂闪烁。
 */

const props = withDefaults(defineProps<{
  // 降级态文案，透传给 ChartPreparingState（缺省回落到 observer.chart.preparing）
  label?: string
  // 最多自愈重挂次数
  maxRetries?: number
}>(), {
  label: undefined,
  maxRetries: 2,
})

// 作为子图表的 :key，自增即触发一次干净重挂
const reloadKey = ref(0)
// 是否处于降级态（重挂间隙或永久降级时显示准备态）
const degraded = ref(false)
let retries = 0

// 优先在下一帧重挂（等布局/动画稳定），兼容无 requestAnimationFrame 的环境
const scheduleReload = (fn: () => void) => {
  if (typeof requestAnimationFrame === 'function') {
    requestAnimationFrame(() => fn())
  } else {
    setTimeout(fn, 0)
  }
}

onErrorCaptured((err) => {
  logger.warn('[claudeObserver] chart render error contained', err)
  if (retries < props.maxRetries) {
    retries += 1
    // 先移除坏子树，下一帧再以新 key 干净重挂
    degraded.value = true
    scheduleReload(() => {
      reloadKey.value += 1
      degraded.value = false
    })
  } else {
    // 重试用尽：永久降级，不再尝试
    degraded.value = true
  }
  // 阻断异常继续向上冒泡到全局 errorHandler
  return false
})
</script>
