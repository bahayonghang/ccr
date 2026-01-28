<template>
  <button
    :class="buttonClasses"
    :style="buttonStyle"
    :disabled="disabled || loading"
    :aria-label="ariaLabel"
    :aria-busy="loading"
    :aria-disabled="disabled"
    @click="handleClick"
  >
    <!-- Leading icon slot -->
    <span
      v-if="$slots.leading && !loading"
      class="flex-shrink-0"
      :class="iconSizeClasses"
    >
      <slot name="leading" />
    </span>

    <!-- Loading spinner -->
    <span
      v-if="loading"
      class="flex-shrink-0"
      :class="iconSizeClasses"
    >
      <svg
        class="animate-spin"
        :class="iconSizeClasses"
        fill="none"
        viewBox="0 0 24 24"
      >
        <circle
          class="opacity-25"
          cx="12"
          cy="12"
          r="10"
          stroke="currentColor"
          stroke-width="4"
        />
        <path
          class="opacity-75"
          fill="currentColor"
          d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
        />
      </svg>
    </span>

    <!-- Content -->
    <span :class="{ 'opacity-0': loading && !$slots.leading }">
      <slot />
    </span>

    <!-- Trailing icon slot -->
    <span
      v-if="$slots.trailing && !loading"
      class="flex-shrink-0"
      :class="iconSizeClasses"
    >
      <slot name="trailing" />
    </span>
  </button>
</template>

<script setup lang="ts">
import { computed } from 'vue'

// Props interface
interface Props {
  /** 按钮变体 */
  variant?: 'primary' | 'secondary' | 'success' | 'warning' | 'danger' | 'ghost' | 'outline'
  /** 按钮尺寸 */
  size?: 'sm' | 'md' | 'lg'
  /** 是否禁用 */
  disabled?: boolean
  /** 是否全宽 */
  fullWidth?: boolean
  /** 是否加载中 */
  loading?: boolean
  /** ARIA 标签 */
  ariaLabel?: string
}

// Emits
const emit = defineEmits<{
  click: [event: MouseEvent]
}>()

// Props with defaults
const props = withDefaults(defineProps<Props>(), {
  variant: 'primary',
  size: 'md',
  disabled: false,
  fullWidth: false,
  loading: false,
  ariaLabel: undefined,
})

// 变体样式 - 使用 Neo-Terminal 设计令牌
const variantStyles = {
  primary: [
    'bg-accent-primary text-white',
    'hover:bg-accent-primary/90 hover:shadow-glow-primary',
    'focus:ring-accent-primary/30',
    'shadow-md',
  ],
  secondary: [
    'bg-bg-tertiary text-text-primary',
    'border border-border-color',
    'hover:bg-bg-hover hover:border-border-hover',
    'focus:ring-accent-primary/30',
  ],
  success: [
    'bg-accent-success text-white',
    'hover:bg-accent-success/90 hover:shadow-glow-success',
    'focus:ring-accent-success/30',
    'shadow-md',
  ],
  warning: [
    'bg-accent-warning text-white',
    'hover:bg-accent-warning/90 hover:shadow-glow-warning',
    'focus:ring-accent-warning/30',
    'shadow-md',
  ],
  danger: [
    'bg-accent-danger text-white',
    'hover:bg-accent-danger/90 hover:shadow-glow-danger',
    'focus:ring-accent-danger/30',
    'shadow-md',
  ],
  ghost: [
    'bg-transparent text-text-primary',
    'hover:bg-bg-hover',
    'focus:ring-accent-primary/30',
  ],
  outline: [
    'bg-transparent text-accent-primary',
    'border border-accent-primary/50',
    'hover:bg-accent-primary/10 hover:border-accent-primary',
    'focus:ring-accent-primary/30',
  ],
}

// 尺寸样式
const sizeStyles = {
  sm: 'text-sm px-3 py-1.5 gap-1.5 rounded-lg',
  md: 'text-base px-4 py-2.5 gap-2 rounded-xl',
  lg: 'text-lg px-6 py-3.5 gap-2.5 rounded-xl',
}

// 图标尺寸
const iconSizes = {
  sm: 'w-4 h-4',
  md: 'w-5 h-5',
  lg: 'w-6 h-6',
}

// 计算图标尺寸类
const iconSizeClasses = computed(() => iconSizes[props.size])

// 计算按钮类
const buttonClasses = computed(() => [
  // 基础样式
  'relative inline-flex items-center justify-center',
  'font-medium transition-all duration-200',
  'focus:outline-none focus:ring-2 focus:ring-offset-2',
  // 尺寸
  sizeStyles[props.size],
  // 变体
  ...variantStyles[props.variant],
  // 全宽
  props.fullWidth ? 'w-full' : '',
  // 禁用/加载状态
  (props.disabled || props.loading)
    ? 'opacity-50 cursor-not-allowed pointer-events-none'
    : 'cursor-pointer',
])

// 计算按钮样式
const buttonStyle = computed(() => ({
  '--tw-ring-offset-color': 'var(--bg-primary)',
}))

// 点击处理
function handleClick(event: MouseEvent) {
  if (!props.disabled && !props.loading) {
    emit('click', event)
  }
}
</script>
