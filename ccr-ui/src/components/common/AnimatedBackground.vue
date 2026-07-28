<template>
  <div
    :class="backgroundLayerClass"
    aria-hidden="true"
  />
</template>

<script setup lang="ts">
import { computed } from 'vue'

type BackgroundVariant = 'default' | 'complex' | 'aurora' | 'spotlight' | 'mesh' | 'orbs' | 'minimal'
type SpotlightColor = 'primary' | 'secondary' | 'success' | 'warning' | 'danger' | 'info'

// 氛围层收敛：组件壳保留（3 处挂载点不动），props 契约不变；
// halo / grain / spotlight 等动画层已随配色系统重构移除，统一渲染静态不透明基底。
const props = withDefaults(defineProps<{
  variant?: BackgroundVariant
  spotlightColor?: SpotlightColor
  contained?: boolean
  complex?: boolean
}>(), {
  variant: 'default',
  spotlightColor: 'primary',
  contained: false,
  complex: false,
})

const backgroundLayerClass = computed(() => (
  props.contained
    ? 'background-layer background-layer--contained absolute inset-0 overflow-hidden pointer-events-none -z-10 transition-colors duration-500'
    : 'background-layer fixed inset-0 overflow-hidden pointer-events-none -z-10 transition-colors duration-500'
))
</script>

<style scoped>
.background-layer {
  isolation: isolate;
  background: var(--color-bg-base);
}
</style>
