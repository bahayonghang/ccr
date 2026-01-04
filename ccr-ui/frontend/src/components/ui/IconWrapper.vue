<template>
  <div
    :class="wrapperClasses"
    :style="{ '--icon-color': colorValue }"
  >
    <slot>
      <component
        :is="icon"
        v-if="icon"
        :class="iconClasses"
      />
    </slot>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { Component } from 'vue'

interface Props {
  /** 图标组件 */
  icon?: Component
  /** 图标大小 */
  size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl'
  /** 颜色变体 */
  variant?: 'default' | 'primary' | 'secondary' | 'success' | 'warning' | 'danger' | 'info'
  /** 自定义颜色 (CSS 颜色值) */
  color?: string
  /** 是否显示背景 */
  background?: boolean
  /** 背景形状 */
  shape?: 'circle' | 'rounded' | 'square'
}

const props = withDefaults(defineProps<Props>(), {
  icon: undefined,
  size: 'md',
  variant: 'default',
  color: undefined,
  background: false,
  shape: 'rounded',
})

// 尺寸映射
const sizes = {
  xs: { wrapper: 'w-6 h-6', icon: 'w-3 h-3' },
  sm: { wrapper: 'w-8 h-8', icon: 'w-4 h-4' },
  md: { wrapper: 'w-10 h-10', icon: 'w-5 h-5' },
  lg: { wrapper: 'w-12 h-12', icon: 'w-6 h-6' },
  xl: { wrapper: 'w-16 h-16', icon: 'w-8 h-8' },
}

// 颜色映射
const colors = {
  default: 'var(--color-text-secondary)',
  primary: 'var(--color-accent-primary)',
  secondary: 'var(--color-accent-secondary)',
  success: 'var(--color-success)',
  warning: 'var(--color-warning)',
  danger: 'var(--color-danger)',
  info: 'var(--color-info)',
}

// 背景颜色映射
const bgColors = {
  default: 'bg-bg-surface',
  primary: 'bg-accent-primary/10',
  secondary: 'bg-accent-secondary/10',
  success: 'bg-accent-success/10',
  warning: 'bg-accent-warning/10',
  danger: 'bg-accent-danger/10',
  info: 'bg-accent-info/10',
}

// 形状映射
const shapes = {
  circle: 'rounded-full',
  rounded: 'rounded-xl',
  square: 'rounded-none',
}

// 计算颜色值
const colorValue = computed(() => props.color || colors[props.variant])

// 包装器类
const wrapperClasses = computed(() => [
  'inline-flex items-center justify-center flex-shrink-0',
  'transition-colors duration-200',
  sizes[props.size].wrapper,
  props.background ? [bgColors[props.variant], shapes[props.shape]] : '',
])

// 图标类
const iconClasses = computed(() => [
  sizes[props.size].icon,
  'text-[var(--icon-color)]',
])
</script>
