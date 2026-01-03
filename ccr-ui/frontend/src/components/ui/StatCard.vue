<template>
  <div :class="cardClasses">
    <!-- 图标区域 -->
    <div
      class="flex-shrink-0 w-12 h-12 rounded-xl flex items-center justify-center"
      :class="iconContainerClasses"
    >
      <slot name="icon">
        <component
          :is="icon"
          v-if="icon"
          class="w-6 h-6"
        />
      </slot>
    </div>

    <!-- 内容区域 -->
    <div class="flex-1 min-w-0">
      <!-- 标签 -->
      <p class="text-sm text-text-muted truncate">
        <slot name="label">
          {{ label }}
        </slot>
      </p>
      
      <!-- 数值 -->
      <div class="flex items-baseline gap-2 mt-1">
        <span class="text-2xl font-bold text-text-primary tabular-nums">
          <slot name="value">{{ formattedValue }}</slot>
        </span>
        
        <!-- 变化趋势 -->
        <span
          v-if="change !== undefined && change !== 0"
          class="text-xs font-medium flex items-center gap-0.5"
          :class="changeClasses"
        >
          <svg
            v-if="change > 0"
            class="w-3 h-3"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M5 15l7-7 7 7"
            />
          </svg>
          <svg
            v-else
            class="w-3 h-3"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M19 9l-7 7-7-7"
            />
          </svg>
          {{ Math.abs(change) }}%
        </span>
      </div>
      
      <!-- 附加描述 -->
      <p
        v-if="description || $slots.description"
        class="text-xs text-text-muted mt-1 truncate"
      >
        <slot name="description">
          {{ description }}
        </slot>
      </p>
    </div>

    <!-- 右侧操作区 -->
    <div
      v-if="$slots.action"
      class="flex-shrink-0"
    >
      <slot name="action" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { Component } from 'vue'

interface Props {
  /** 图标组件 */
  icon?: Component
  /** 标签文字 */
  label?: string
  /** 数值 */
  value?: number | string
  /** 数值格式化函数 */
  formatter?: (value: number | string) => string
  /** 变化百分比 */
  change?: number
  /** 附加描述 */
  description?: string
  /** 颜色变体 */
  variant?: 'default' | 'primary' | 'secondary' | 'success' | 'warning' | 'danger'
  /** 是否显示边框 */
  bordered?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  icon: undefined,
  label: undefined,
  value: undefined,
  formatter: undefined,
  change: undefined,
  description: undefined,
  variant: 'default',
  bordered: true,
})

// 格式化数值
const formattedValue = computed(() => {
  if (props.value === undefined) return '-'
  if (props.formatter) return props.formatter(props.value)
  if (typeof props.value === 'number') {
    return props.value.toLocaleString()
  }
  return props.value
})

// 变体颜色映射
const variantColors = {
  default: {
    container: 'bg-bg-surface',
    icon: 'text-text-secondary',
  },
  primary: {
    container: 'bg-accent-primary/10',
    icon: 'text-accent-primary',
  },
  secondary: {
    container: 'bg-accent-secondary/10',
    icon: 'text-accent-secondary',
  },
  success: {
    container: 'bg-accent-success/10',
    icon: 'text-accent-success',
  },
  warning: {
    container: 'bg-accent-warning/10',
    icon: 'text-accent-warning',
  },
  danger: {
    container: 'bg-accent-danger/10',
    icon: 'text-accent-danger',
  },
}

// 卡片类
const cardClasses = computed(() => [
  'flex items-start gap-4 p-4 rounded-xl bg-bg-elevated',
  'transition-all duration-200 ease-out',
  props.bordered ? 'border border-border-subtle hover:border-border-default' : '',
])

// 图标容器类
const iconContainerClasses = computed(() => [
  variantColors[props.variant].container,
  variantColors[props.variant].icon,
])

// 变化趋势类
const changeClasses = computed(() => {
  if (props.change === undefined) return ''
  return props.change > 0
    ? 'text-accent-success'
    : props.change < 0
      ? 'text-accent-danger'
      : 'text-text-muted'
})
</script>
